## Red Velvet Engine — Black Cherry 🍒

Red Velvet Engine (RVE) is a stateless fraud decision engine focused on fast HTTP ingestion, clear auditing metadata, and rule-based scoring. The repository hosts two Rust crates:

- `crates/rve-core`: domain types (events, rules, scoring) plus service ports shared across adapters.
- `crates/rve`: CLI, HTTP adapter (Axum), logging, and Redis/Dragonfly integration for runtime state.

Documentation that dives deeper into the API and rule semantics lives under `docs/` (`docs/api.md`, `docs/A.md`).

## Quick Start

1. Install a recent stable Rust toolchain (`rustup default stable`), plus Docker if you want to run the bundled Dragonfly cache.
2. Start the cache locally (optional but required for the rule routes to hit Redis):
   ```bash
   docker compose up -d dragonfly
   ```
3. Run the engine:
   ```bash
   cargo run -p rve -- --host 0.0.0.0 --port 3439
   ```
4. Hit the health endpoints:
   ```bash
   curl -i http://localhost:3439/health
   curl -s http://localhost:3439/status | jq
   ```
5. Explore the API:
   ```bash
   curl -s http://localhost:3439/api/v1/rules | jq
   curl -s http://localhost:3439/api/v1/metadata/contract | jq
   curl -s http://localhost:3439/api-docs/openapi.json | jq '.paths | keys'
   ```

## CLI Reference

The `rve` binary understands the following flags (see `crates/rve/src/cli.rs`):

| Flag | Default | Description |
| --- | --- | --- |
| `--host` | `[::]` | Bind address for the HTTP listener. |
| `-p`, `--port` | `3439` | TCP port for the API. |
| `-v`, `-vv`, `-vvv` | `0` | Increase verbosity (INFO/DEBUG/TRACE). |
| `-q`, `--quiet` | _false_ | Only emit fatal logs and disable the banner. |

## API Surface (current)

- `GET /health` — lightweight liveness check with a custom header.
- `GET /status` — includes the crate version exported from `rve-core`.
- `GET/POST /api/v1/rules` — list and create rules.
- `GET/PUT/PATCH/DELETE /api/v1/rules/{id}` — full CRUD over rule documents.
- `POST /api/v1/decisions` — evaluates an `EventInput` request body and returns a decision.
- `GET /api/v1/metadata/fields` and `GET /api/v1/metadata/contract` — field catalog and contract metadata.
- `GET /api/v1/engine/status` and `POST /api/v1/engine/reload` — runtime engine status and explicit ruleset reload.
- `GET /docs` and `GET /api-docs/openapi.json` — API documentation endpoints.

Detailed payload expectations and planned lifecycle operations live in `docs/api.md`.

## Rule Contract Notes (strict, no legacy compatibility)

- `meta.author` is required for rule metadata.
- `meta.autor` is rejected (legacy alias removed).
- `evaluation.condition` and `evaluation.logic` must use canonical operators supported by domain validation.
- Legacy expression aliases are not normalized automatically (e.g., `=` or `not_in` are rejected).
- Rule writes do not hot-reload the engine; call `POST /api/v1/engine/reload` after create/update/patch/delete.

## Minimal Examples

Create a rule:

```bash
curl -sS -X POST http://localhost:3439/api/v1/rules \
  -H 'content-type: application/json' \
  -d '{
    "meta": {
      "code": "FRAUD-HV-001",
      "name": "High value transaction",
      "description": "Flags large transaction amounts",
      "version": "1.0.0",
      "author": "RiskOps",
      "tags": ["high_value"]
    },
    "state": {
      "mode": "active",
      "audit": {
        "created_at_ms": 1730000000000,
        "updated_at_ms": 1730000001000
      }
    },
    "schedule": {
      "active_from_ms": 1730000000000
    },
    "rollout": { "percent": 100 },
    "evaluation": {
      "condition": true,
      "logic": { ">": [ { "var": "payload.money.value" }, 1000 ] }
    },
    "enforcement": {
      "score_impact": 6.5,
      "action": "review",
      "severity": "high",
      "tags": ["financial_fraud"]
    }
  }' | jq
```

Create a decision (direct body, no `event` wrapper):

```bash
curl -sS -X POST http://localhost:3439/api/v1/decisions \
  -H 'content-type: application/json' \
  -d '{
    "header": {
      "timestamp": "2026-01-01T00:00:00Z",
      "source": "checkout",
      "event_id": "evt_123",
      "instrument": "card",
      "channel": "web"
    },
    "context": {
      "geo": { "country": "US", "lat": 40.71, "lon": -74.01 },
      "net": { "source_ip": "203.0.113.10" },
      "env": { "device_id": "dev_1", "session_id": "sess_1" }
    },
    "features": {
      "fin": {
        "first_seen_at": 1730000000000,
        "last_seen_at": 1730000005000,
        "last_declined_at": null,
        "total_successful_txns": 12,
        "total_declined_txns": 1,
        "total_amount_spent": 150000,
        "max_ticket_ever": 45000,
        "consecutive_failed_logins": 0,
        "current_hour_count": 2,
        "current_hour_amount": 1500,
        "current_day_count": 3,
        "current_day_amount": 1500,
        "consecutive_declines": 0,
        "known_ips": ["203.0.113.10"],
        "known_devices": ["dev_1"]
      }
    },
    "signals": { "flags": {} },
    "payload": {
      "money": { "value": 1500.0, "ccy": "USD" },
      "parties": {
        "originator": {
          "entity_type": "individual",
          "acct": "acc_1",
          "country": "US",
          "bank": "bank_1",
          "kyc": "tier_2",
          "watchlist": "no",
          "sanctions_score": 0.01
        },
        "beneficiary": {
          "entity_type": "business",
          "acct": "acc_2",
          "country": "US",
          "bank": "bank_2",
          "kyc": "tier_3",
          "watchlist": "no",
          "sanctions_score": 0.0
        }
      }
    }
  }' | jq
```

## Development Workflow

- Format & lint: `cargo fmt && cargo clippy --all-targets --all-features`.
- Type-check fast: `cargo check` or use `bacon` (see `bacon.toml`) for a live dev loop (`bacon` hotkeys: `c` for clippy, `t` for tests, `r` for `cargo run`).
- Run tests (public baseline):
  - `cargo test -p rve-core`
  - `cargo test -p rve-test-suite`
  - `cargo test -p rve`
- Full testing strategy (including private suites): `testing/README.md`.
- Logging is powered by `tracing`; adjust verbosity via CLI flags or `RUST_LOG`.

## License

Red Velvet Engine is distributed under the Business Source License 1.1 (BUSL-1.1). See `LICENSE` for the full text, parameters (licensor, change date, additional grants), and post-change open-source terms.
