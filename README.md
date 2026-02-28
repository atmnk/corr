# Corr and Journey Programming Language

Corr is a Rust-based runtime and DSL for defining and executing automated API/system journeys, plus workload scenarios for load/performance style execution.

## Documentation

- Full documentation: [https://qalens.com/site/corr](https://qalens.com/site/corr)

## Documentation Summary

Based on the repository structure and examples, Corr provides:

- A `Journey` DSL (`*.journey`) to model executable flows.
- A `WorkLoad` DSL (`*.workload`) to orchestrate journey scenarios.
- Built-in step support for:
  - REST
  - WebSocket client/server
  - DB
  - System
  - Listener-style flows
- Template and extraction helpers for:
  - JSON, object, text, and form data
- Runtime outputs to:
  - `console`
  - `influxdb2`

Example (`examples/src/WSClient.journey`) shows WebSocket connect/send/listen behavior, while `examples/src/Server.journey` shows a broadcast-style WebSocket server journey.

## Repository Layout

- `corr/`: CLI and runners (`build` / `run`) for journeys and workloads.
- `corr-lib/`: DSL parsers, core runtime, templates, and step implementations.
- `playground/`: integration playground crate.
- `examples/`: sample `.journey` programs.
- `cfg/`: environment-specific config files.

## Developer Setup

### Prerequisites

- Rust toolchain (Docker build uses `rustlang/rust:nightly`).
- Cargo.

### Build

```bash
cargo build --workspace
```

### Run CLI

```bash
# Run a journey from source (build + run)
cargo run -p corr -- run -t . <default>

# Run a workload from source
cargo run -p corr -- run -t . -w <default>
```

Notes:

- `-w` / `--workload` switches execution mode from journey to workload.
- `-o` / `--out` supports `console` (default) and `influxdb2`.
- `-d` / `--debug` enables debug mode.

## Packaging (`.jpack`)

Corr can package journeys/workloads and their dependencies:

```bash
# Build package artifact under ./build/*.jpack
cargo run -p corr -- build -t . <default>
```

When `jpack.toml` exists at the target root, `package.name` is used for the generated artifact name.

## Docker

The repository includes a multi-stage `Dockerfile` that builds `corr` in a Rust builder image and runs it in a slim Debian image.
