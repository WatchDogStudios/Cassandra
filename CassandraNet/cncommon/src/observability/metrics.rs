use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricPoint {
    pub name: String,
    pub kind: MetricKind,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

impl MetricPoint {
    pub fn counter(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Counter,
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        }
    }

    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Gauge,
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        }
    }

    pub fn histogram(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Histogram,
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        }
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }
}

#[derive(Clone, Default)]
pub struct InMemoryMetricsRegistry {
    inner: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
}

impl InMemoryMetricsRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record(&self, point: MetricPoint) {
        let mut guard = self.inner.write().expect("metrics write lock poisoned");
        guard.entry(point.name.clone()).or_default().push(point);
    }

    pub fn increment_counter(
        &self,
        name: impl Into<String>,
        value: f64,
        labels: Option<HashMap<String, String>>,
    ) {
        let mut point = MetricPoint::counter(name, value);
        if let Some(labels) = labels {
            point = point.with_labels(labels);
        }
        self.record(point);
    }

    pub fn set_gauge(
        &self,
        name: impl Into<String>,
        value: f64,
        labels: Option<HashMap<String, String>>,
    ) {
        let mut point = MetricPoint::gauge(name, value);
        if let Some(labels) = labels {
            point = point.with_labels(labels);
        }
        self.record(point);
    }

    pub fn observe_histogram(
        &self,
        name: impl Into<String>,
        value: f64,
        labels: Option<HashMap<String, String>>,
    ) {
        let mut point = MetricPoint::histogram(name, value);
        if let Some(labels) = labels {
            point = point.with_labels(labels);
        }
        self.record(point);
    }

    pub fn snapshot(&self, name: &str) -> Vec<MetricPoint> {
        self.inner
            .read()
            .expect("metrics read lock poisoned")
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn snapshot_all(&self) -> HashMap<String, Vec<MetricPoint>> {
        self.inner
            .read()
            .expect("metrics read lock poisoned")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_reads_counter() {
        let registry = InMemoryMetricsRegistry::new();
        registry.increment_counter("ugc_uploads", 1.0, None);
        let snapshot = registry.snapshot("ugc_uploads");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].kind, MetricKind::Counter);
        assert_eq!(snapshot[0].value, 1.0);
    }

    #[test]
    fn supports_histogram_observations() {
        let registry = InMemoryMetricsRegistry::new();
        registry.observe_histogram("upload_latency", 42.0, None);
        let snapshot = registry.snapshot("upload_latency");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].kind, MetricKind::Histogram);
    }
}
