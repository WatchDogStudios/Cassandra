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

### UGC Service (`cmd/ugc-service`)

- **Purpose**: Persist metadata for submitted assets, surface moderation status, and drive the console experience.
- **Ingress**: `POST /content` captures submissions with `{content_id, tenant_id, project_id, filename, mime_type, size_bytes, labels, attributes}`.
- **Moderation**: Review decisions arrive via `POST /content/{id}/review` with `{state, reason}`. States align with proto enum `ContentState` (`pending`, `approved`, `rejected`, `archived`).
- **Egress**: `GET /content` lists submissions filtered by tenant, project, or state; responses mirror the gRPC contract in `cnproto/proto/ugc.proto`.
- **Core Package**: `internal/ugc` owns HTTP translation, domain validation, and delegates persistence to pluggable stores (in-memory today, Postgres planned).

### Orchestration Service (`cmd/orchestrator`)

- **Purpose**: Manage agent assignments and lifecycle transitions for workloads scheduled by the control plane.
- **Ingress**: `POST /assignments` registers work for an agent with `{agent_id, workload_id, tenant_id, project_id, metadata}`.
- **Lifecycle**: `PATCH /assignments/{id}` updates status (`pending`, `assigned`, `in_progress`, `completed`, `failed`, `cancelled`) and optional status messages.
- **Egress**: `GET /assignments` lists assignments filtered by agent, tenant, project, or status aligning with `cassandra.orchestration.v1` proto messages.
- **Core Package**: `internal/orchestration` provides validation plus swappable persistence with an in-memory store for local development.

### Messaging Service (`cmd/messaging-service`)

- **Purpose**: Provide publish/pull semantics for gameplay and platform events prior to integrating external brokers.
- **Ingress**: `POST /topics/{topic}/messages` accepts `{tenant_id, project_id, key, payload_base64, priority, attributes}` and queues messages.
- **Consumption**: `GET /topics/{topic}/messages` streams messages with optional tenant/project filters and configurable limits.
- **Ack Flow**: `POST /topics/{topic}/messages/{id}/ack` removes messages after successful processing. Priorities map to `cassandra.messaging.v1` proto enums.
- **Core Package**: `internal/messaging` encapsulates storage and HTTP presentation with a memory-backed store that will be replaced by Postgres (and optional Redis cache) later.

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

1. Extend the in-memory stores with Postgres-backed implementations and migrations.
2. Wire the new services into gateway/gRPC handlers once proto integration lands.
3. Provide a top-level README with quickstart commands, docker-compose definitions, and troubleshooting tips.
