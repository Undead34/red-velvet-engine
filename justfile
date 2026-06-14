app_name := `grep -m1 '^name = ' Cargo.toml | sed 's/.*"\(.*\)"/\1/'`
version  := `cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].version' || echo "?"`

# Infrastructure (docker compose)
up:        # Start all required services
    docker compose up -d dragonfly

down:      # Stop all services
    docker compose down

reset:     # Wipe and restart fresh
    docker compose down -v && docker compose up -d dragonfly

ps:        # Show service status
    docker compose ps

# Dev workflow (just)
check:     # Fast compile check
    cargo check --workspace

lint:      # Clippy lints
    cargo clippy --workspace -- -D warnings

fmt:       # Format code
    cargo fmt

fmt-check: # Check formatting (CI)
    cargo fmt --check

test:      # Run all workspace tests
    cargo test --workspace

test-core: # Run core crate tests only
    cargo test -p rve-core

run:       # Start the API server
    cargo run -p rve -- --host 0.0.0.0 --port 3439

dev: up    # Infra + run
    just run

# API helpers (requires server running)
health:    # Health check
    curl -s http://localhost:3439/health | jq .

builder-config: # UI builder schema
    curl -s http://localhost:3439/api/v1/ui/builder-config | jq .

openapi:   # OpenAPI spec
    curl -s http://localhost:3439/openapi.json | jq .

seed:      # Create example rule + reload engine
    curl -s -X POST http://localhost:3439/api/v1/rules \
      -H 'Content-Type: application/json' \
      -d '{"meta":{"name":"High Value USD","author":"just","version":"1.0.0"},"state":{"mode":"active","audit":{"created_at_ms":1,"updated_at_ms":1}},"schedule":{},"rollout":{"percent":100},"evaluation":{"condition":true,"logic":{"in":[{"var":"payload.money.ccy"},["USD"]]}},"enforcement":{"score_impact":5.0,"action":"review","severity":"moderate","tags":["example"]}}' | jq .
    curl -s -X POST http://localhost:3439/api/v1/engine/reload | jq .

# Build
build:     # Release build
    cargo build --release

clean:     # Clean all artifacts
    cargo clean

default:
    @just --list
