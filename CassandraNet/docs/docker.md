# Docker & Compose Setup

This repository now ships container images for the primary services and a docker-compose.yml that stitches them together for local development or smoke testing.

## Prerequisites

- Docker Engine 23+ with the Compose plugin available as `docker compose`
- A `.env` file next to `docker-compose.yml` (see `.env.example`). Compose refuses
  to start until these are set:
  - `POSTGRES_PASSWORD` and a matching `CASS__DATABASE__URL`
  - `CASS_JWT_SECRET` (32+ random bytes, e.g. `openssl rand -hex 32`)
  - `CASS_ADMIN_PASSWORD` (console login for `admin@localhost`)
  - `CASS_AGENT_SECRET` (the agent's API key; compose grants it `agent:execute`)

Copy the template to get started:

```bash
cp .env.example .env
# then edit the new .env file with your own secrets
```

## Building Images

Each service has a dedicated Dockerfile under `docker/`:

- `docker/Dockerfile.gateway` – Rust gateway with the Postgres feature enabled
- `docker/Dockerfile.agent` – Rust edge agent
- `docker/Dockerfile.frontend` – React console built into an nginx image

To build everything locally:

```bash
docker compose build
```

## Running the Stack

With your `.env` populated, start the services:

```bash
docker compose up
```

This boots:

- `postgres` (compose network only)
- `gateway` on ports 8080 (HTTP) and 8081 (gRPC)
- `agent` registering against the gateway with `CASS_AGENT_SECRET`. It refuses
  commands and scripts unless `CASS_AGENT_ALLOW_EXEC=true`, and restarts itself
  via the `on-failure` policy when sent `restart`
- `console` on port 5173 with `/api` proxied to the gateway
- `dragonfly` (Redis-compatible agent registry cache, compose network only)
- `jaeger` UI on port 16686, receiving the gateway's OTLP traces

Uploaded builds live in the `builds-data` volume (`/data/builds` in the gateway).

Access the console at http://localhost:5173/ once the containers are healthy.

Stop the stack with `docker compose down` and add `-v` if you need to remove database volumes.

## CI Pipeline

The GitHub Actions workflow `.github/workflows/ci.yml` (at the repository root) runs on every push and pull request:

1. Rust formatting, clippy (with and without the `db` feature), and tests
2. Go vet and tests for the peripherals
3. Console lint, type-check, and build
4. Gateway, agent, and console Docker builds via Buildx

Pushing a `v*` tag runs `.github/workflows/release.yml`, which publishes Linux
binaries to a GitHub release and pushes images to `ghcr.io/<owner>/cassandranet-{gateway,agent,console}`.

Use the workflow as a reference when extending the pipeline or reproducing checks locally.
