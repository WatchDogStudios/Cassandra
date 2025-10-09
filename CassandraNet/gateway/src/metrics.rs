use crate::auth::{validate_api_key, validate_jwt, AuthStatus};
use axum::{
    body::Body,
    http::{HeaderValue, Request},
};
use once_cell::sync::Lazy;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, Encoder, HistogramVec,
    IntCounterVec, IntGauge, TextEncoder,
};
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use uuid::Uuid;

static REQ_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "gateway_http_requests_total",
        "HTTP request count",
        &["method", "path", "status"]
    )
    .unwrap()
});
static REQ_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "gateway_http_request_duration_seconds",
        "HTTP request latency",
        &["method", "path", "status"]
    )
    .unwrap()
});
static ERROR_5XX_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "gateway_http_errors_total",
        "HTTP 5xx error count",
        &["method", "path", "status"]
    )
    .unwrap()
});
static BUILD_INFO: Lazy<IntGauge> = Lazy::new(|| {
    let g = register_int_gauge!(
        "gateway_build_info",
        "Build info as a constant 1 gauge with labels"
    )
    .unwrap();
    g.set(1);
    g
});
static PROCESS_CPU: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("process_cpu_percent", "Process CPU percent * 100").unwrap());
static PROCESS_MEM: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("process_memory_bytes", "Resident memory bytes").unwrap());

pub struct MetricsLayer;

impl Clone for MetricsLayer {
    fn clone(&self) -> Self {
        MetricsLayer
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner }
    }
}

#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for MetricsService<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut req = req;
        let req_id = req
            .headers()
            .get("x-request-id")
            .and_then(|id| id.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let rid = if req_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            req_id
        };
        let method = req.method().as_str().to_owned();
        let path = normalize_path(req.uri().path()).into_owned();
        let is_public = matches!(
            path.as_str(),
            "/health" | "/version" | "/metrics" | "/docs" | "/api-docs/openapi.json"
        );
        if !is_public {
            match (validate_api_key(req.headers()), validate_jwt(req.headers())) {
                (AuthStatus::Allow, _) | (_, AuthStatus::Allow) => {}
                _ => {
                    let resp = axum::response::Response::builder()
                        .status(axum::http::StatusCode::UNAUTHORIZED)
                        .header("x-request-id", &rid)
                        .body(axum::body::Body::from("unauthorized"))
                        .unwrap();
                    REQ_COUNTER
                        .with_label_values(&[&method, &path, "401"])
                        .inc();
                    ERROR_5XX_COUNTER
                        .with_label_values(&[&method, &path, "401"])
                        .inc();
                    return Box::pin(async move { Ok(resp) });
                }
            }
        }
        req.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_str(&rid).unwrap_or(HeaderValue::from_static("invalid")),
        );
        let start = Instant::now();
        let mut inner = self.inner.clone();
        let fut = inner.call(req);
        Box::pin(async move {
            match fut.await {
                Ok(resp) => {
                    let status = resp.status().as_u16().to_string();
                    REQ_COUNTER
                        .with_label_values(&[&method, &path, &status])
                        .inc();
                    let dur = start.elapsed().as_secs_f64();
                    REQ_LATENCY
                        .with_label_values(&[&method, &path, &status])
                        .observe(dur);
                    if status.starts_with('5') {
                        ERROR_5XX_COUNTER
                            .with_label_values(&[&method, &path, &status])
                            .inc();
                    }
                    let mut resp = resp;
                    resp.headers_mut()
                        .insert("x-request-id", HeaderValue::from_str(&rid).unwrap());
                    Ok(resp)
                }
                Err(e) => {
                    REQ_COUNTER
                        .with_label_values(&[&method, &path, "error"])
                        .inc();
                    let dur = start.elapsed().as_secs_f64();
                    REQ_LATENCY
                        .with_label_values(&[&method, &path, "error"])
                        .observe(dur);
                    ERROR_5XX_COUNTER
                        .with_label_values(&[&method, &path, "error"])
                        .inc();
                    Err(e)
                }
            }
        })
    }
}

pub fn gather_metrics() -> (axum::http::StatusCode, String) {
    update_process_metrics();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buf = Vec::new();
    match encoder.encode(&metric_families, &mut buf) {
        Ok(_) => (
            axum::http::StatusCode::OK,
            String::from_utf8_lossy(&buf).into_owned(),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("encode error: {e}"),
        ),
    }
}

pub fn normalize_path(path: &str) -> std::borrow::Cow<'_, str> {
    if matches!(path, "/health" | "/version" | "/metrics") {
        return std::borrow::Cow::Borrowed(path);
    }
    let mut changed = false;
    let norm: Vec<_> = path
        .split('/')
        .map(|seg| {
            if seg.is_empty() {
                return seg;
            }
            if seg.chars().all(|c| c.is_ascii_digit()) {
                changed = true;
                return ":id";
            }
            if seg.len() >= 8
                && seg.chars().filter(|c| *c == '-').count() >= 2
                && seg.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
            {
                changed = true;
                return ":id";
            }
            seg
        })
        .collect();
    if changed {
        std::borrow::Cow::Owned(norm.join("/"))
    } else {
        std::borrow::Cow::Borrowed(path)
    }
}

fn update_process_metrics() {
    use sysinfo::System;
    static SYS: Lazy<std::sync::Mutex<System>> =
        Lazy::new(|| std::sync::Mutex::new(System::new_all()));
    if let Ok(mut sys) = SYS.lock() {
        sys.refresh_processes();
        if let Ok(pid) = sysinfo::get_current_pid() {
            if let Some(proc_) = sys.process(pid) {
                let cpu = (proc_.cpu_usage() * 100.0) as i64;
                PROCESS_CPU.set(cpu);
                PROCESS_MEM.set(proc_.memory() as i64 * 1024);
            }
        }
    }
    let _ = BUILD_INFO.get();
}
