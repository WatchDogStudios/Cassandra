# CassandraNet

![Cassandra Logo](data/images/branding/CASS_LOGO_CORE_.svg)

A Rust & Go powered server management platform for games. Used internally at WD Studios & now open-sourced.

## Quick Start (Docker)

Get the full stack running in under 5 minutes:

```bash
# Clone the repository
git clone https://github.com/WatchDogStudios/Cassandra.git
cd Cassandra/CassandraNet

# Copy environment config, then fill in every secret (see the comments inside)
cp .env.example .env

# Start all services
docker compose up -d
```

Open [http://localhost:5173](http://localhost:5173) to access the operator console.

**Login:**
- Email: `admin@localhost`
- Password: the `CASS_ADMIN_PASSWORD` you set in `.env`

See [docs/docker.md](CassandraNet/docs/docker.md) for detailed Docker configuration options.

## What is CassandraNet?

CassandraNet manages multiple servers (Virtual Machines, containers, etc.) through a unified portal & API. It helps server owners:

- Connect and authenticate users across game servers
- Verify user sessions and manage player bans
- Monitor server load and agent health in real-time
- Orchestrate workloads across your fleet
- Extend functionality via plugins/add-ons

## Architecture Overview

| Component | Technology | Purpose |
|-----------|------------|---------|
| Gateway | Rust (Axum) | HTTP/gRPC API, auth, metrics |
| Agent | Rust | Runs on each managed node, reports telemetry |
| Console | React + Vite | Operator dashboard |
| Peripherals | Go | Messaging, orchestration, UGC services |
| Database | PostgreSQL | Persistent storage |

## Running Locally

### With Docker Compose (Recommended)

See [Quick Start](#quick-start-docker) above. The compose stack includes:
- PostgreSQL 15 database
- Gateway API (ports 8080/8081)
- Agent (connects to gateway with `CASS_AGENT_SECRET`)
- Console UI (port 5173)
- Dragonfly cache and Jaeger tracing (UI on port 16686)

### Without Docker

**Prerequisites:**
- Rust (stable toolchain) + Cargo
- Node.js 18+
- Go 1.22+
- PostgreSQL (optional, for `db` feature)

**Build & Run** (from the `CassandraNet/` directory):

```bash
# Build all crates
cargo build

# Run the gateway. It refuses to start without a JWT secret. Without
# CASS_ADMIN_PASSWORD it logs a generated password for admin@localhost.
export CASS_JWT_SECRET="$(openssl rand -hex 32)"
cargo run -p cngateway

# Run an agent; its key needs the agent:execute scope
# (e.g. start the gateway with CASS_API_KEYS="my-agent-key:agent:execute").
# Add --allow-exec and/or --deploy-dir <dir> to accept commands, scripts and builds.
cargo run -p cnagent -- --secret my-agent-key

# In another terminal, run the console
cd frontend/console
npm ci
npm run dev
```

## Configuration Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `CASS_JWT_SECRET` | HS256 signing secret (use 32+ random bytes) | Required |
| `CASS_ADMIN_PASSWORD` | Password for `admin@localhost` | Generated and logged (in-memory mode only) |
| `CASS__DATABASE__URL` | PostgreSQL connection string (`db` builds) | - |
| `CASS__HTTP__BIND_ADDR` | Gateway HTTP bind address; gRPC uses port + 1 | `0.0.0.0:8080` |
| `CASS_API_KEYS` | Bootstrap API keys, `key:scope\|scope,...` | - |
| `CASS_COOKIE_SECURE` | `Secure` flag on the refresh cookie | `true` |
| `CASS_RATE_LIMITS` | Per-route overrides, `route:limit:window_secs,...` | Built-in defaults |
| `CASS_REDIS_URL` | Redis/Dragonfly for the agent registry | In-memory |
| `CASS_STORAGE_BASE_URL` | Base URL for UGC upload targets | `s3://cassandranet` |
| `CASS_BUILD_STORAGE_DIR` | Where uploaded builds are stored | `data/builds` (`/data/builds` in the image) |
| `CASS_MAX_BUILD_BYTES` | Largest accepted build upload | 2 GiB |
| `COLLECTOR_OTLP_ENABLED` | Export traces via OTLP (`OTEL_EXPORTER_OTLP_ENDPOINT`) | Off |
| `CASS_GATEWAY_ADDR` | Agent: gateway gRPC address | `http://127.0.0.1:8081` |
| `CASS_AGENT_SECRET` | Agent: API key with `agent:execute` | Required |
| `CASS_AGENT_ALLOW_EXEC` | Agent: run `exec` commands and scripts | `false` |
| `CASS_AGENT_DEPLOY_DIR` | Agent: deploy builds into `<dir>/<build_id>/` | Unset (deploys refused) |
| `CASS_AGENT_NODE_ID` | Agent: UUID identity | Derived from the hostname |
| `RUST_LOG` | Log level | `info` |

See [.env.example](CassandraNet/.env.example) for a complete template.

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/version` | GET | Build info |
| `/metrics` | GET | Prometheus metrics |
| `/agents` | GET | List connected agents |
| `/auth/login` | POST | Authenticate user |
| `/auth/refresh` | POST | Refresh access token |
| `/auth/me` | GET | Get current user |
| `/auth/logout` | POST | Clear session |
| `/agents/{id}/config` | GET, PUT | Agent config (`agent:read` / `agent:write`) |
| `/agents/{id}/commands` | GET, POST | Command history / queue `restart`, `shutdown`, `exec`, `collect_logs`, `deploy_build` |
| `/builds` | GET, POST | List builds / upload one as `multipart/form-data` (`build:read` / `build:write`) |
| `/deployments` | POST | Deploy a build to agents (`build:deploy`) |
| `/scripts`, `/scripts/{id}/execute` | … | Script library and runs (`script:*`) |
| `/settings` | GET, PUT | Platform settings (`admin:read` / `admin:write`) |
| `/docs` | GET | Swagger UI with the full API (requires `docs:read`) |

Commands, script runs and deployments are queued as jobs. Agents pick them up
on their next heartbeat and report results, which appear in the console.

The gRPC port (HTTP port + 1) serves `AgentControl` (registration, heartbeats,
job results, build downloads), `UgcService` and `MessagingService`. Every call
must carry `x-api-key` or `authorization: Bearer …` metadata with the same
scopes as HTTP (`agent:execute`, `ugc:read`/`ugc:write`,
`messaging:read`/`messaging:write`).

## Security Notes

- The gateway fails to start without `CASS_JWT_SECRET`; there is no fallback key.
- Access tokens expire after 15 minutes; refresh tokens are only accepted at `/auth/refresh`.
- HTTP and gRPC are served in plaintext. Put the gateway behind a TLS-terminating
  proxy before exposing it beyond a trusted network.
- `CASS_API_KEYS` is a bootstrap mechanism; with the `db` feature, manage keys
  through the API (`apikey:manage`) instead.
- Agents run operator-supplied code only when started with `--allow-exec`, and
  only deploy builds into an explicit `--deploy-dir`. Enabling `--allow-exec`
  gives every holder of `agent:write` or `script:execute` code execution on that
  host.
- Agents currently share credentials: any `agent:execute` key can act as any
  agent ID. Issue that scope only to agents you trust.

## Development Guide

### Running Tests

From the `CassandraNet/` directory (these mirror CI):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Go peripherals
cd peripherals && go vet ./... && go test ./...

# Console (lint, type-check, build)
cd frontend/console && npm ci && npm run lint && npm run build
```

### Building from Source

```bash
# Debug build
cargo build

# Release gateway with PostgreSQL support
cargo build --release -p cngateway --features db

# C SDK
cargo build -p cnsdk_c --release
```

## Contributing

Feel free to submit a PR anytime. We embrace open-source and would love to grow this project with the OSS community.

CassandraNet follows the [Rust Lang's Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Third-Party Libraries

See [license.html](CassandraNet/license.html) for the libraries used in this project.

## Releases

Tagging `vX.Y.Z` (matching the crate versions) runs the release workflow, which
publishes Linux binaries and the C SDK to a GitHub release and pushes container
images to GHCR. See [CHANGELOG.md](CHANGELOG.md).

## License

MIT License - see LICENSE file for details.
