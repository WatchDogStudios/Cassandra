# CassandraNet Gateway

Initial HTTP gateway service providing a `/health` endpoint.

## Run

```
cargo run -p cngateway
```

Environment overrides (prefix `CASS__`):
- `CASS__SERVICE_NAME` (default: cassandra-gateway)
- `CASS__HTTP__BIND_ADDR` (default: 127.0.0.1:8080)

## Test

Visit http://127.0.0.1:8080/health

```json
{"status":"ok","service":"cassandra-gateway"}
```
