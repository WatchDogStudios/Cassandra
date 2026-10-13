# CassandraNet Roadmap – Road to Production

This document captures the staged plan for turning CassandraNet from an internal prototype into a production-ready platform. Tracks are grouped by phase; each deliverable lists the target outcomes, success criteria, and primary owners once teams are assigned.

## Phase 0 – Baseline (Complete)

- **Proto contracts & scaffolding**: Core protobufs, Rust gateway HTTP/gRPC services, Go peripheral services, and the React console shell are in place.
- **Persistence primitives**: In-memory stores and initial Postgres-backed implementations for orchestration, moderation, messaging, and content models.
- **Observability foundations**: Shared metrics & logging abstractions, gateway Prometheus endpoint, and lightweight rate-limiter scaffolding.
- **Tooling setup**: Go/Rust formatters, Go tests, React build, and `cargo test` pass locally.

## Phase 1 – Developer Experience (Complete)

| Epic | Description | Success Criteria | Status |
| --- | --- | --- | --- |
| Docker & Compose | Ship minimal container images (gateway, agent, console) and a compose stack with Postgres. | `docker compose up` runs end-to-end with seeded secrets; console surfaces live telemetry. | ✅ |
| CI pipeline | Unified GitHub Actions workflow covering Rust fmt/clippy/tests, Go tests, frontend build, and Docker image builds. | Every PR runs cleanly; failures block merge; caching keeps runtime under 12 minutes. | ✅ |
| Local DX | Documentation for developers (README, docker docs, onboarding checklist). | Fresh clone + documented steps get a developer to a running stack in <30 minutes. | ✅ |
| Frontend UX uplift | Expanded agent detail view, metrics timelines, and auth-friendly proxy behaviour. | Operator console shows agent selection, charts, and handles auth via API key or bearer token. | ✅ |

### Completed Tasks

- [x] Flesh out README with Docker/CI usage and restructure for Docker-first onboarding.
- [x] Publish `.env.example` guidance in README and docs.
- [x] Add linting (ESLint/TypeScript) to CI for the console.
- [x] Implement login flow with argon2 password hashing and httpOnly refresh cookies.
- [x] Add default admin user seeding with `CASS_ADMIN_PASSWORD` env var override.
- [x] Create AuthProvider with in-memory tokens and proactive 14-minute silent refresh.

> **Correction (v0.1.0):** the CI workflow lived under `CassandraNet/.github/`
> and never ran. It now lives at the repository root; see CHANGELOG.md.

## Phase 2 – Core Platform Hardening

**Status at v0.1.0:** token expiry and refresh-token separation, scope
enforcement on messaging/UGC and every gRPC service, Postgres audit logging,
a migration chain that applies cleanly (CI-tested against PostgreSQL), and
gRPC/HTTP wiring for agent jobs, builds and scripts have landed. Still open:
per-agent credentials, tenant isolation on messaging/UGC, the orchestration API
and CLI/SDK release hardening.

| Epic | Description | Success Criteria |
| --- | --- | --- |
| Auth & security | Implement API key management, JWT scope enforcement, rate limiting, and audit logging. | Authn/Authz flows fully enforced for `/agents`, `/tenants/*`, messaging endpoints; logs ship to sink. |
| Persistent data stores | Finalize Postgres schema migrations for orchestration assignments, moderation reviews, messaging queues. | SQLx migrations versioned, tested; failover between in-memory and DB validated. |
| gRPC & HTTP wiring | Surface orchestration/moderation/messaging flows via gateway HTTP + tonic services. | End-to-end tests create assignments, moderate UGC, publish/ack messages. |
| SDK & tooling | Harden Rust agent, publish CLI utilities, and validate C SDK binding. | CI builds release binaries; SDK integration tests cover registration + heartbeat. |

### Supporting Work

- Build integration test suite (`tests/combination_tests.rs` + dockerized checks) covering Postgres + services.
- Define error taxonomy and consistent HTTP response shapes.
- Instrument metrics for persistence latency/errors.

## Phase 3 – Production Readiness

| Epic | Description | Success Criteria |
| --- | --- | --- |
| Observability & SLOs | Define SLIs/SLOs, export structured logs, metrics, and traces (OTLP). | Grafana dashboards show agent counts, request latency, DB health; alerts configured. |
| Scalability | Introduce horizontal scaling strategy (stateless gateway, queue-backed workers, optional cache). | Load tests demonstrate 10k concurrent agents with <200ms request latency 99th percentile. |
| Secrets & configuration | Integrate with Vault/Azure Key Vault (or similar). Replace env secrets with secure storage. | Production deployment consumes secrets from secure provider; zero secrets in repo. |
| Deployment automation | Helm charts / Terraform modules / GitOps pipeline for deploying to Kubernetes or cloud VMs. | `helm install` (or equivalent) deploys entire stack with minimal manual steps; blue/green supported. |

### Additional Deliverables

- Disaster Recovery (DR) runbook covering backup/restore for Postgres and messaging queues.
- Performance regression suite with automated benchmarks.
- Support plan: tiered on-call rotation, incident response guidelines.

## Phase 4 – GA Launch

| Epic | Description | Success Criteria |
| --- | --- | --- |
| Documentation & onboarding | Product docs, APIs, SDK guides, tutorials, launch blog. | Public documentation site published; onboarding metrics tracked. |
| Licensing & compliance | Finalize licensing, third-party notices, and privacy/security compliance (SOC 2 readiness checklist). | Legal review complete; `license.html` & NOTICE up to date; compliance backlog defined. |
| Customer pilots | Engage design partners for pilot deployments, gather feedback. | At least two pilot customers running in staging; feedback loop integrated into roadmap. |
| Support tooling | Ticketing integration, status page, incident comms. | Status page operational; support SLAs defined; escalation path documented. |

## Tracking & Governance

- **Issue tracking**: GitHub Projects board separated by phase (Backlog → In Progress → QA → Done).
- **Release cadence**: Monthly milestones through Phase 2, then two-week release trains targeting GA.
- **Quality gates**: CI must pass; code review from platform owners; integration tests required for cross-service changes.
- **Communication**: Bi-weekly roadmap reviews; monthly stakeholder updates; post-mortems for Sev-1 incidents.

## Dependencies & Risks

- **Database availability**: Production durability hinges on Postgres clustering / HA strategy.
- **Security posture**: JWT secret handling and API key management must precede external pilots.
- **Operational staffing**: Need dedicated SRE/DevOps bandwidth by Phase 3.
- **Scope creep**: Guard against expanding plugin ecosystem before core orchestration/messaging are stable.

## Appendix – Quick Reference

- `.github/workflows/ci.yml` – CI pipeline configuration.
- `docker/` – Dockerfiles & nginx config for gateway, agent, console.
- `docker-compose.yml` – Local stack definition; requires secrets via `.env` (see `.env.example`).
- `docs/docker.md` – Detailed Docker/Compose instructions.
- `frontend/console` – Operator console; expects API key or JWT for non-public routes.
- `gateway/src` – HTTP & gRPC wiring, auth middleware, metrics.
- `core/src/platform` – Persistence traits, models, Postgres implementations.

This roadmap should be revisited at the end of each phase and refined based on customer feedback, operational learnings, and evolving priorities.
