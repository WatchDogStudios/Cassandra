# Peripheral Services (Go)

This module contains supporting microservices that complement the CassandraNet Rust core. Each service is intentionally lightweight, relies only on the Go standard library, and can run independently for local development or staging deployments.

## Services Overview

| Service | Binary | Default Port | Responsibilities |
|---------|--------|--------------|------------------|
| Metrics Collector | `cmd/metrics-collector` | `8081` | Accepts custom metric samples and exposes aggregate summaries. |
| Log Pipeline | `cmd/log-pipeline` | `8082` | Buffers structured logs, applies severity filtering, and forwards to registered sinks. |
| UGC Worker | `cmd/ugc-worker` | `8083` | Moderates user-generated content using a keyword policy and exposes moderation results. |
| Notification Service | `cmd/notification` | `8084` | Renders templates and dispatches notifications across channels. |

## Shared Conventions

- **Configuration**: Environment variables prefixed with `METRICS_`, `LOG_PIPELINE_`, `UGC_`, or `NOTIFY_` respectively. Defaults target local development without any configuration.
- **Observability**: Every service exposes `GET /healthz` and standard logs via the shared logger in `internal/logging`.
- **Graceful Shutdown**: Each binary traps `SIGINT/SIGTERM` and performs cleanup (draining queues, stopping workers).

## Quickstart

From the repository root:

```cmd
cd peripherals
# Run the metrics collector
go run ./cmd/metrics-collector
```

Each service can be launched in a similar way (`./cmd/log-pipeline`, `./cmd/ugc-worker`, `./cmd/notification`).

### Example API Calls

- **Metrics Collector**
  - `POST /metrics/ingest`: `{ "namespace": "api", "name": "latency", "value": 120, "labels": {"route": "/v1"} }`
  - `GET /metrics/summary`
- **Log Pipeline**
  - `POST /logs`: `{ "source": "gateway", "level": "INFO", "message": "request completed" }`
  - `GET /logs/recent`
- **UGC Worker**
  - `POST /jobs`: `{ "content_id": "123", "author_id": "user", "body": "example" }`
  - `GET /jobs/next`
- **Notification Service**
  - `POST /notify`: `{ "channel": "email", "recipient": "user@example.com", "template": "welcome_email", "data": {"Name": "Ada"} }`
  - `GET /notifications/recent`

## Configuration Reference

| Service | Variable | Default | Description |
|---------|----------|---------|-------------|
| Metrics | `METRICS_HTTP_ADDR` | `:8081` | Listen address. |
| Log Pipeline | `LOG_PIPELINE_HTTP_ADDR` | `:8082` | Listen address. |
| Log Pipeline | `LOG_PIPELINE_QUEUE_SIZE` | `256` | Event queue capacity. |
| Log Pipeline | `LOG_PIPELINE_MIN_LEVEL` | `INFO` | Minimum severity to process. |
| Log Pipeline | `LOG_PIPELINE_RECENT_CAPACITY` | `200` | Size of in-memory recent log buffer. |
| UGC Worker | `UGC_HTTP_ADDR` | `:8083` | Listen address. |
| UGC Worker | `UGC_QUEUE_SIZE` | `256` | Job queue capacity. |
| UGC Worker | `UGC_WORKERS` | `4` | Number of moderation workers. |
| UGC Worker | `UGC_BANNED_TERMS` | `spam,scam` | Comma-separated banned phrases. |
| Notification | `NOTIFY_HTTP_ADDR` | `:8084` | Listen address. |
| Notification | `NOTIFY_RECENT_CAPACITY` | `200` | History size for recent deliveries. |

## Testing

Run the test suite from within the module:

```cmd
cd peripherals
go test ./...
```

This executes unit tests covering aggregators, pipelines, worker queues, and HTTP handlers.
