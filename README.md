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
5. Explore the rules API scaffold (currently returning mock data while the storage adapter is completed):
   ```bash
   curl -s http://localhost:3439/api/v1/rules | jq
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
- `GET /api/v1/rules` — placeholder pagination endpoint that will back rule listing via Redis/Dragonfly.
- `GET/PUT/DELETE /api/v1/rules/{id}` — mock endpoints showcasing the `Rule` contract defined in `rve-core`.

Detailed payload expectations and planned lifecycle operations live in `docs/api.md`.

## Development Workflow

- Format & lint: `cargo fmt && cargo clippy --all-targets --all-features`.
- Type-check fast: `cargo check` or use `bacon` (see `bacon.toml`) for a live dev loop (`bacon` hotkeys: `c` for clippy, `t` for tests, `r` for `cargo run`).
- Run tests: `cargo test --all`.
- Logging is powered by `tracing`; adjust verbosity via CLI flags or `RUST_LOG`.

## License

Red Velvet Engine is distributed under the Business Source License 1.1 (BUSL-1.1). See `LICENSE` for the full text, parameters (licensor, change date, additional grants), and post-change open-source terms.
