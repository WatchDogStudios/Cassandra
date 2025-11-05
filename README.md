# CassandraNet

![Cassandra Logo](data/images/branding/CASS_LOGO_CORE_.svg)

A Rust & Go powered server management platform for games. Used internally at WD Studios & is now being open-sourced, and ported over to rust.

## What is this really?

CassandraNet simply just manages multiple servers (Virtual Machines, etc...) in one organized portal & API.

It helps server owners have a unified portal for all of their server needs, like connecting with authenticated users, verifying user sessions, banning players, managing server load, and so much more.

CassandraNet also gives you the ability to add "Plugins/Add-ons" to your instance to essentially fulfill more needs for your use case.

## How would i Intergrate this within my own systems?

For example, if you use Azure Playfab for managing your games servers, ever since they [Sunset Legacy Multiplayer Servers](https://community.playfab.com/questions/58173/i-wanted-to-host-custom-dedicated-servers-not-on-a.html), You can't connect non-azure servers to Playfab anymore, so you are left with no other option.

What we recommend, is that you use Services like Playfab if you want a full live-ops solution right off the bat.

If you want a more managed solution with more control, CassandraNet is the way to go.

## Code-of-conduct

CassandraNet follows the [Rust Lang's Code-of-Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Contributing

Feel free to submit a PR at anytime. we embrace open-source, and would love to grow this project with the OSS Community.

## Third-Party libraries Used

Check [license.html](CassandraNet/license.html) for the Libraries used in the project.

## Running & Testing (Current Snapshot)

This section documents how to run what already exists in the repo today. Components are early-stage and APIs will change.

### Prerequisites
* Rust (stable toolchain) + Cargo
* Node.js 18+ (for the React console)
* (Optional) PostgreSQL if you want to enable the `db` feature
* Set an env var for JWT signing if you want to issue / validate HS256 tokens: `CASS_JWT_SECRET="devsecret"`

### Workspace Layout (active crates)
* `core` (`cncore`): config, tracing, optional DB pool & migrations
* `gateway` (`cngateway`): HTTP API (health, version, metrics, agents), OpenAPI UI, auth stubs, metrics middleware, Prometheus
* `cnproto`: gRPC/protobuf definitions (AgentControl)
* `cnagent`: Simple agent that registers & sends heartbeats over gRPC
* `cnsdk_c`: C ABI stub (builds staticlib/cdylib)
* `frontend/marketing`: Next.js marketing site (landing page, product overview)
* `frontend/console`: React + Vite console (operator dashboard)

### Build Everything
```
cargo build
```
Add `--features db` to include Postgres integration (see DB section below).

### Run the Gateway
```
cargo run -p cngateway
```
Environment overrides (all optional) use the `CASS__` prefix (double underscore for nesting):
```
set CASS__HTTP__BIND_ADDR=127.0.0.1:8080
set CASS_JWT_SECRET=devsecret
cargo run -p cngateway
```
Endpoints:
* `GET /health` – liveness
* `GET /version` – build metadata (git sha/tag if provided at build time)
* `GET /metrics` – Prometheus metrics (includes request counters, latency histogram, process metrics)
* `GET /agents` – current in‑memory agent registry
* `GET /docs` – Swagger UI (serves OpenAPI from `/api-docs/openapi.json`)

### Generate a JWT (HS256)
```
set CASS_JWT_SECRET=devsecret
cargo run -p cngateway -- gen-token --sub test-user
```
Returns a compact JWT valid for 1 hour (no advanced claims yet). Auth on HTTP routes is currently permissive: public system routes bypass auth; other future routes will require either `x-api-key` or `Authorization: Bearer <token>`.

### Enable & Use the Database Feature
1. Start/PostgreSQL locally and create a database (default URL used if none specified):
	* Default: `postgres://localhost:5432/cassandra`
2. Override via env:
```
set CASS__DATABASE__URL=postgres://user:pass@localhost:5432/cassandra
set CASS__DATABASE__MAX_CONNECTIONS=10
cargo run -p cngateway --features db
```
Migrations run automatically on gateway startup (tables: users, orgs, memberships, titles, nodes, node_metrics). Agent register / heartbeat will upsert node rows and insert metrics.

### Run the Agent (gRPC)
Gateway gRPC listens on HTTP port + 1 (e.g. if HTTP is 8080, gRPC is 8081). Launch agent after gateway:
```
set CASS_AGENT_TENANT_ID=<tenant uuid>
set CASS_AGENT_PROJECT_ID=<project uuid>
cargo run -p cnagent -- --gateway http://127.0.0.1:8081
```
It will RegisterAgent then send periodic Heartbeat messages; the gateway exposes results at `GET /agents` and (if DB enabled) persists metadata.

### Frontend (Marketing & Console)

**Marketing site (Next.js)**
```
cd frontend/marketing
npm install   # first time
npm run dev
```
Runs the landing experience at <http://localhost:3000>. Tailwind powers the styling, and metadata is configured in `src/app/layout.tsx`.

**Operator console (Vite)**
```
cd frontend/console
npm install   # first time
npm run dev
```
Vite (default <http://localhost:5173>) proxies `/api/*` to the gateway (`http://127.0.0.1:8080`). The refreshed UI shows backend health, key agent telemetry, and quick actions.

### Integration Tests
Run all workspace tests:
```
cargo test
```
Notable tests:
* Gateway health route
* Metrics exposition & custom counters
* OpenAPI security scheme presence
* Agent register + heartbeat (spins ephemeral HTTP & gRPC servers then queries `/agents`)

### C SDK Stub
Build the C ABI crate (produces library artifacts under `target`):
```
cargo build -p cnsdk_c --release
```
Exports placeholder functions (init, authenticate, send_metric, get_server_session, shutdown). The header is in `cnsdk_c/include/cassandra.h`.

### Prometheus Metrics Snapshot
Example metric names (see `/metrics`):
* `gateway_http_requests_total{method,path,status}`
* `gateway_http_request_duration_seconds{...}`
* `gateway_http_errors_total{...}`
* `gateway_build_info` (constant gauge = 1)
* `process_cpu_percent`, `process_memory_bytes`

### Common Troubleshooting
* Build fails with missing OpenSSL / TLS: we're using `native-tls` indirectly via sqlx when `db` feature is enabled; ensure system SSL libraries present (on Windows this is usually fine by default).
* gRPC client cannot connect: verify port (HTTP+1) and that gateway log shows `grpc listening` line.
* `/agents` empty: run the agent, or ensure heartbeats are being sent (logs / run with `RUST_LOG=info`).

### Next Planned Enhancements (Roadmap Hints)
* Auth hardening (claims validation, roles, API keys management)
* Persistent agent registry & richer querying
* Additional services (orchestration, UGC, messaging) & Go microservices layer
* Expanded frontend (agent detail, metrics charts)
* Docker / Compose & CI pipelines

## UGC Upload & Metadata (prototype)

The current codebase now includes the first slice of a UGC pipeline:

* **Domain models** – `core::platform::models` defines `ContentMetadata`, `UploadSession`, and related enums for tracking uploads, access visibility, and project-scoped storage preferences.
* **Persistence contract** – `core::platform::persistence::ContentStore` abstracts storage providers. The in-memory implementation is wired into `InMemoryPersistence` so tests and early flows can exercise listing and querying metadata without an external database.
* **Listing support** – a `ContentQuery` struct drives filtered listings by tenant, project, tags, and free-text matching across filenames and metadata attributes.
* **Pluggable storage settings** – `ProjectStorageSettings` and additions to `TenantSettings` capture the target bucket/prefix configuration that real backends can interpret when wiring S3, Azure Blob, etc.

### Upload session flow (planned wiring)
1. Client requests an upload; the platform creates an `UploadSession` (status `Pending`) with a signed URL/header set for direct-to-storage uploads.
2. Client transfers the blob to storage.
3. Client (or storage webhook) finalises the session via `ContentStore::record_content_metadata`, transitioning the session to `Completed` and recording the blob metadata.
4. Downstream services can list or fetch UGC using the metadata catalogue while respecting tenant and project boundaries.

Middleware for rate limiting and validation is slated to sit in front of the session creation endpoint; see the "Security hardening" notes for the current plan.

## Observability & Logging Scaffolding

Shared utilities under `cncommon::observability` lay the groundwork for unified telemetry across services:

* `InMemoryMetricsRegistry` – captures counters, gauges, and histograms with label support. Ideal for unit tests or early prototypes before wiring to Prometheus / OpenTelemetry exporters.
* `LogPipeline` + `LogSink` – fan-out hub for structured log forwarding. An `InMemoryLogSink` is provided for tests; production sinks can forward to OTLP, Loki, or vendor APIs.
* `LogEvent` carries component identifiers, tenant/project context, and arbitrary JSON metadata so audit trails remain multi-tenant aware.

These utilities are lightweight, but already power unit tests to guarantee metrics and logs are emitted and captured. Future work will expose builders to plug them into the eventual HTTP gateway and background workers.

## Security Hardening Checklist (in progress)

To protect UGC uploads and multi-tenant workloads, the following items are being actively designed:

* **Rate limiting** – per-tenant counters built on the metrics registry will feed sliding-window throttles for upload/session creation APIs.
* **Tenant isolation** – storage metadata always records both tenant and project IDs; any persistence implementation must enforce these in query filters.
* **Audit & observability** – every upload and metadata mutation will emit structured `LogEvent`s so downstream SIEM tooling can detect anomalies.
* **End-to-end tests** – forthcoming integration specs will exercise the happy path (upload + metadata listing) alongside boundary cases (cross-tenant access, oversized objects, expired sessions).

Contributors interested in these areas can start by reading the `ContentStore` trait and the observability helpers before proposing concrete integrations.

---
This doc reflects the in-flight prototype state; expect breaking changes until a tagged pre-release.
