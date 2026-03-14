<div align="center">
  <h1>Red Velvet Engine — Black Cherry 🍒</h1>
  <p><strong>Real-time fraud decisioning with clean domain boundaries</strong></p>
  <p>Built for Risk, Fraud Ops, and Trust teams that need speed, control, and auditability.</p>
  <br>

  [![License: BUSL-1.1](https://img.shields.io/badge/License-BUSL--1.1-blue.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/Rust-1.93+-orange)](https://www.rust-lang.org/)

  [📘 **API Contract**](docs/api.md) | [🧪 **Testing Guide**](testing/README.md) | [🧭 **OpenAPI**](/api-docs/openapi.json)
</div>

---

## 🌟 What is Red Velvet Engine?

**Red Velvet Engine (RVE)** is a fraud decision platform for high-risk digital flows.

It turns risk signals into deterministic decisions through:

- 📜 Policy-driven rules
- ⚡ Real-time event evaluation
- 🧾 Auditable outcomes

RVE is designed to keep the business domain clean while runtime components stay replaceable.

## 🚀 Why teams choose RVE

- 🎯 Consistent decisions under strict validation
- 🔍 Clear contracts for integration and governance
- 🧱 Strong invariants to avoid invalid fraud policies
- 🔄 Runtime flexibility without coupling business logic to infrastructure

## 🏗️ Architecture at a glance

RVE is split into two crates:

- `crates/rve-core`: fraud domain, invariants, ports, decision services
- `crates/rve`: HTTP API, runtime adapter, storage wiring

This keeps `rve-core` implementation-agnostic and focused on business correctness.

## ✨ Core capabilities

- ✅ Rule CRUD with lifecycle/state controls
- ✅ Real-time decision endpoint
- ✅ Metadata contract for fields and JSONLogic roots
- ✅ Runtime status and explicit ruleset reload
- ✅ OpenAPI docs for integration teams

## 🎯 Quick start

### Prerequisites

- Rust stable toolchain
- Docker (optional, for local cache services)

### Run

```bash
# Optional local cache
docker compose up -d dragonfly

# Start API
cargo run -p rve -- --host 0.0.0.0 --port 3439
```

### Health checks

```bash
curl -i http://localhost:3439/health
curl -s http://localhost:3439/status | jq
curl -s http://localhost:3439/api/v1/rules | jq
```

## 🌐 API surface

- `GET /health`
- `GET /status`
- `GET/POST /api/v1/rules`
- `GET/PUT/PATCH/DELETE /api/v1/rules/{id}`
- `POST /api/v1/decisions` (placeholder, returns `501`)
- `GET /api/v1/metadata/fields`
- `GET /api/v1/metadata/contract`
- `GET /api/v1/engine/status` (placeholder runtime status)
- `POST /api/v1/engine/reload` (placeholder, returns `501`)
- `GET /docs`
- `GET /api-docs/openapi.json`

## 📚 Documentation

- API contract: [`docs/api.md`](docs/api.md)
- Interactive docs: `http://localhost:3439/docs`
- OpenAPI JSON: `http://localhost:3439/api-docs/openapi.json`
- Testing strategy: [`testing/README.md`](testing/README.md)

## 🛠️ Developer workflow

```bash
cargo check
cargo test -p rve-core
cargo test -p rve
```

## 📄 License

Red Velvet Engine is distributed under **Business Source License 1.1**.
See [`LICENSE`](LICENSE) for complete terms.
