# Changelog

All notable changes to CassandraNet are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). Until 1.0, minor versions may break
compatibility.

## [0.1.0] - Unreleased

First tagged pre-release.

### Added

- **Operator portal backed by real data.** Agent config, agent commands, builds,
  deployments, the script library and platform settings are persisted (in
  memory, or PostgreSQL with the `db` feature) instead of returning placeholders.
  - Commands, script runs and deployments become *agent jobs*. Agents receive
    them in the heartbeat response and report results over the new
    `ReportJobResult` RPC; results (exit code, stdout/stderr, errors) show up in
    the console.
  - Supported commands: `restart`, `shutdown`, `exec`, `collect_logs`,
    `deploy_build`. Script languages: bash, sh, PowerShell, Python, JavaScript
    (Node), Lua. Script arguments arrive as `CASS_ARG_<NAME>` environment
    variables.
  - Builds upload as `multipart/form-data` to `POST /builds` (SHA-256 recorded,
    stored under `CASS_BUILD_STORAGE_DIR`, limit `CASS_MAX_BUILD_BYTES`, default
    2 GiB). Agents download them over the new `DownloadBuild` gRPC stream and
    verify size and checksum before installing.
  - Agent heartbeat interval follows the agent's config (or the platform
    default).
- **gRPC authentication.** Every gRPC call needs the same credentials and scopes
  as HTTP (`agent:execute`, `ugc:*`, `messaging:*`).
- New scopes `messaging:read` / `messaging:write`; the seeded `admin@localhost`
  now has every operator scope it needs for the console.
- Postgres audit trail: audit events are written to `audit_logs`.
- Release workflow: tagging `vX.Y.Z` publishes Linux binaries (gateway, agent,
  CLI, C SDK) and GHCR images.

### Security

- Access tokens are now rejected once expired, and refresh tokens no longer
  authorize API calls (previously any signed token worked forever).
- No built-in JWT signing key: the gateway refuses to start without
  `CASS_JWT_SECRET` (it previously fell back to a public default that let anyone
  mint admin tokens).
- In-memory mode no longer seeds `admin`/`admin`; without
  `CASS_ADMIN_PASSWORD` a random password is generated and logged once.
- Messaging and UGC endpoints enforce `messaging:*` / `ugc:*` scopes instead of
  accepting any signed-in user.
- Refresh cookies are `Secure` by default (`CASS_COOKIE_SECURE=false` to opt out).
- Rejected API keys are logged by prefix only, and the Redis URL is no longer logged.
- Agents only run `exec` and scripts when started with `--allow-exec`, and only
  deploy builds when given `--deploy-dir`. Both are off by default.
- Containers run as a non-root user. Compose no longer publishes the
  unauthenticated Redis/Dragonfly port or the OTLP ports.

### Fixed

- PostgreSQL migrations now apply to a fresh database. 0006 referenced
  `tenants`/`projects` before 0010 created them, and 0007 tried to recreate
  `users` (and referenced a non-existent `orgs` table). Every migration after 0005
  failed, which broke login and API keys in `db` builds.
- Migrations are pinned to LF line endings so builds from Windows and Linux agree
  on their checksums.
- `cargo build --features db` compiles again.
- Docker images build: Rust 1.95 builder (dependencies need ≥ 1.85), bookworm
  builder to match the runtime's libssl, `Cargo.lock` committed for `--locked`
  builds, console image installs its build tooling.
- Compose: the agent reads `CASS_GATEWAY_ADDR` and no longer sends non-UUID
  tenant/project IDs; the gateway receives the admin password, Redis and OTLP
  settings; nginx accepts large build uploads.
- CI actually runs. The workflow lived under `CassandraNet/.github/`, which
  GitHub ignores. It now runs fmt, clippy (with and without `db`), tests, a
  PostgreSQL integration job, Go, console lint/type-check/build and Docker builds.
- Console: Builds and Scripts pages match the API, show command and script
  output, and drop the unsupported "Update Agent" command.
- Agents keep their identity: the default node ID is derived from the hostname
  (UUIDv5) instead of a new random UUID on every reconnect or restart, which had
  stranded queued jobs and split history. `--node-id` must be a UUID, and the
  gateway rejects other IDs (in `db` mode they registered but never appeared
  in `/agents`).
- The seeded admin is `admin@localhost` in both in-memory and `db` modes.

### Removed

- The JSON `POST /builds` + `upload_url` flow (it never accepted the file);
  upload via multipart instead.
- `labels`/`attributes`/`visibility` from the UGC upload-session request; they
  were accepted and ignored. Send them when completing the upload.

### Known limitations

- Agents share credentials: any key with `agent:execute` can heartbeat or report
  as any agent ID. Per-agent credentials are planned.
- HTTP and gRPC are plaintext. Terminate TLS in front of the gateway.
- On timeout, `exec`/script jobs kill only the direct child process.
- Agents apply only `heartbeat_interval_seconds` from their config. The other
  config fields and platform settings are stored for operators but not acted on.
- Builds are stored on the gateway's local disk (one gateway instance).
- Jobs dispatched to an agent that dies stay `dispatched`.
- Messaging and UGC moderation HTTP endpoints need the `db` feature, and they do
  not yet check that the caller belongs to the project's tenant.
- The orchestration store is not exposed through the API yet.

### Upgrading from pre-release builds

- Set `CASS_JWT_SECRET` (32+ random bytes) and `CASS_ADMIN_PASSWORD`, and give
  agents an API key with `agent:execute` via `CASS_AGENT_SECRET`.
- Agents that set a non-UUID `CASS_AGENT_NODE_ID` must switch to a UUID (or unset
  it to use the hostname-derived default).
- If you ran a `db` gateway built from a Windows checkout, recreate the database:
  migration checksums changed with the LF pinning, and that database could not
  have migrated past 0005 anyway.
