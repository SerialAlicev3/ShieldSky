# ShieldSky

<p align="center">
  <a href="./USAGE.md">Usage</a>
  ·
  <a href="./rust/README.md">Rust workspace</a>
  ·
  <a href="./PARITY.md">Parity</a>
  ·
  <a href="./ROADMAP.md">Roadmap</a>
  ·
  <a href="./PHILOSOPHY.md">Philosophy</a>
</p>

ShieldSky (SSS — Sky Security System) is a Rust-based passive airspace security platform. It correlates multi-source sensor data — ADS-B, RF, EO/IR, weather, orbital context — to classify aerial threats, assess site exposure, and route automated response actions.

> [!IMPORTANT]
> Start with [`USAGE.md`](./USAGE.md) for build and runtime workflows. Use [`rust/README.md`](./rust/README.md) for crate-level details, [`PARITY.md`](./PARITY.md) for port status, and [`docs/container.md`](./docs/container.md) for the container-first deployment workflow.

## Repository shape

- **`rust/`** — canonical Rust workspace: passive scanner, skyshield engine, response router, API, CLI
- **`USAGE.md`** — task-oriented usage guide
- **`PARITY.md`** — Rust-port parity status and migration notes
- **`ROADMAP.md`** — active roadmap and open work
- **`PHILOSOPHY.md`** — project intent and system-design framing
- **`docs/`** — architecture, container deployment, API design docs
- **`src/` + `tests/`** — companion Python reference workspace and audit helpers

## Quick start

```bash
# 1. Clone and build
git clone https://github.com/SerialAlicev3/ShieldSky
cd ShieldSky/rust
cargo build --workspace

# 2. Run the test suite
cargo test --workspace

# 3. Start the SSS API (development)
cargo run -p sss-api
```

> [!NOTE]
> **Windows (PowerShell):** use `cargo run -p sss-api` or `cargo run -p rusty-claude-cli` to avoid path issues with the binary directly.

## Crate map

| Crate | Purpose |
|---|---|
| `sss-core` | Domain types, shared primitives |
| `sss-passive-scanner` | Multi-source ingest pipeline (ADS-B, RF, weather, orbital…) |
| `sss-skyshield` | Threat classification, track engine, signal fusion |
| `sss-response` | Policy evaluation and response action routing |
| `sss-site-registry` | Site and zone asset registry |
| `sss-api` | HTTP API with Semantic Timeline and dashboard views |
| `sss-storage` | Persistence layer |
| `sss-ingest` | External feed ingestion |
| `telemetry` | Metrics and tracing |

## Documentation map

- [`USAGE.md`](./USAGE.md) — build, run, config
- [`rust/README.md`](./rust/README.md) — crate map and workspace layout
- [`PARITY.md`](./PARITY.md) — port parity status
- [`ROADMAP.md`](./ROADMAP.md) — active roadmap
- [`PHILOSOPHY.md`](./PHILOSOPHY.md) — design intent
- [`docs/sss-skyshield-architecture.md`](./docs/sss-skyshield-architecture.md) — full system architecture
- [`docs/container.md`](./docs/container.md) — container deployment
