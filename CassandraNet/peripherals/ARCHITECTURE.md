# Peripheral Services

## Shared Foundations

- **Module Layout**: All services live under the `peripherals` Go module and share common utilities in `internal` packages (configuration, logging, HTTP helpers, in-memory storage abstractions).
- **Configuration**: Services consume environment variables using `internal/config`. Defaults target local development with the option to override ports, queue sizes, and sink behavior.
- **Logging & Metrics**: A structured logger (currently `log.Logger`) provides leveled output with contextual prefixing. Metrics collector exposes aggregate summaries for quick telemetry; other services report health via `/healthz` endpoints.
- **Graceful Shutdown**: Each service uses a shared server harness that listens for `context` cancellation and OS interrupts, shutting down background workers cleanly.

## Service Overviews

### Metrics Collector (`cmd/metrics-collector`)

- **Purpose**: Ingest custom metrics from edge clients and maintain rolling aggregates for dashboard consumption.
- **Ingress**: `POST /metrics/ingest` with JSON payload `{namespace, name, value, labels}`.
- **Egress**: `GET /metrics/summary` returns per-metric statistics (count, min, max, mean).
- **Core Package**: `internal/metricscollector` provides a thread-safe aggregator with configurable roll-up intervals.

### Log Pipeline (`cmd/log-pipeline`)

- **Purpose**: Receive structured log events, apply filtering/enrichment, and forward to registered sinks.
- **Ingress**: `POST /logs` accepts log entries `{source, level, message, fields}`.
- **Processing**: Events flow through a buffered channel to worker goroutines. Each event is enriched with timestamps and delivered to sinks (initially in-memory ring buffer and stdout sink).
- **Core Package**: `internal/logpipeline` manages sinks, filtering, and backpressure.

### UGC Processing Worker (`cmd/ugc-worker`)

- **Purpose**: Moderate user-generated content and emit review decisions.
- **Ingress**: `POST /jobs` enqueues review jobs with `{content_id, author_id, body}`.
- **Processing**: Dedicated worker pool scans content for disallowed phrases and marks items for review or approval.
- **Egress**: Workers emit decisions to a result stream exposed via `GET /jobs/next` for manual review tooling.
- **Core Package**: `internal/ugcworker` implements the queue, moderation policy engine, and result storage.

### Notification Service (`cmd/notification`)

- **Purpose**: Deliver transactional email and in-app notifications triggered by domain events.
- **Ingress**: `POST /notify` accepts `{channel, recipient, template, data}`.
- **Processing**: Templates render using Go's `text/template`; messages are dispatched to channel-specific senders (email vs. webhook) with in-memory providers for local runs.
- **Egress**: `GET /notifications/recent` exposes recently dispatched messages for debugging.
- **Core Package**: `internal/notification` handles routing, templating, and delivery provider abstractions.

## Testing Strategy

- Each core package ships with unit tests covering happy-path and edge scenarios (duplicate metrics, log backpressure, moderation edge cases, notification template failures).
- Integration-style tests exercise HTTP handlers using the standard library's `httptest` harness.

## Next Steps

1. Scaffold the shared Go module (`go.mod`, `go.work` if needed) and internal utilities.
2. Implement each service following the above contracts.
3. Provide a top-level README with quickstart commands and troubleshooting tips.
