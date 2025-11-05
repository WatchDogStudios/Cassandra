# Docker & Compose Setup

This repository now ships container images for the primary services and a docker-compose.yml that stitches them together for local development or smoke testing.

## Prerequisites

- Docker Engine 23+ with the Compose plugin available as `docker compose`
- A `.env` file in the repository root (see `.env.example`) that provides at minimum:
  - `POSTGRES_PASSWORD`
  - `CASS__DATABASE__URL`
  - `CASS_JWT_SECRET`

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

- `postgres` on port 5432 (internal only)
- `gateway` on ports 8080 (HTTP) and 8081 (gRPC)
- `agent` registering against the gateway
- `console` on port 5173 with `/api` proxied to the gateway

Access the console at http://localhost:5173/ once the containers are healthy.

Stop the stack with `docker compose down` and add `-v` if you need to remove database volumes.

## CI Pipeline

The GitHub Actions workflow `.github/workflows/ci.yml` runs on every push and pull request:

1. Rust formatting, clippy, unit tests, and a database-feature check
2. Go module tests for the peripherals
3. Node build for the frontend console
4. Gateway, agent, and console Docker builds via Buildx

Use the workflow as a reference when extending the pipeline or reproducing checks locally.
