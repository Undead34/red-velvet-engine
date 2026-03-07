# Testing Strategy

## Goals

- Keep production code and test suite loosely coupled.
- Test behavior through public contracts first.
- Allow private/internal tests without publishing them in this repository.

## Validation Baseline

Run this exact baseline from repository root:

```bash
cargo check
cargo test -p rve-core
cargo test -p rve-test-suite
cargo test -p rve
```

What this validates:

- `cargo check`: workspace compiles.
- `rve-core`: crate-level unit/doc tests (if any).
- `rve-test-suite`: black-box contract tests through public APIs.
- `rve`: adapter/API behavior and integration-like tests in that crate.

## Public Suite

- Public black-box tests live in `crates/rve-test-suite`.
- These tests depend on `rve-core` and/or `rve` public APIs only.
- Run with:

```bash
cargo test -p rve-test-suite
```

## In-Crate Unit Tests

- Keep only unit tests that need private internals inside source modules.
- Prefer moving contract/invariant tests to `rve-test-suite` when possible.
- Keep unit tests in crate modules focused and local.

## Private Suite

- Use a local folder `private-tests/` (already gitignored).
- This allows sensitive fraud scenarios, production payloads, and red-team cases
  to stay private.
- Suggested structure:

```text
private-tests/
  Cargo.toml
  suites/
    core_private_contracts.rs
```

- Example `private-tests/Cargo.toml`:

```toml
[package]
name = "rve-private-tests"
version = "0.1.0"
publish = false
edition = "2024"

[dependencies]
rve-core = { path = "../crates/rve-core" }
rve = { path = "../crates/rve" }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- Run private tests from project root:

```bash
cargo test --manifest-path private-tests/Cargo.toml
```

## Recommended CI Split

- Public CI in this repository:

```bash
cargo check
cargo test -p rve-core
cargo test -p rve-test-suite
cargo test -p rve
```

- Private CI (separate repo or private pipeline):

```bash
cargo test --manifest-path private-tests/Cargo.toml
```
