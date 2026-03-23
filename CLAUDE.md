# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`easy_init_newrelic_opentelemetry` is a Rust library crate that provides a simple builder-pattern API (`NewRelicSubscriberInitializer`) to initialize OpenTelemetry tracing, metrics, and logging exporters configured for New Relic's OTLP endpoint. It wires together `tracing-subscriber`, `tracing-opentelemetry`, and `opentelemetry-otlp` into a single `init()` call.

## Build Commands

```bash
cargo build          # Build the library
cargo test           # Run tests (includes doc tests)
cargo clippy         # Lint
cargo fmt --check    # Check formatting
cargo doc --open     # Build and open documentation
```

## Architecture

- **`src/lib.rs`** — Public API. Defines `NewRelicSubscriberInitializer` (builder pattern) and the `init()` method that assembles all tracing/metrics/logging layers into a `tracing_subscriber::Registry`.
- **`src/initialize.rs`** — Internal module. Contains functions to create OTLP exporters and providers (`init_tracer_provider`, `init_metrics`, `init_logger_provider`, `init_process_metrics`). All exporters use HTTP+JSON protocol with `api-key` header authentication.

## Key Design Decisions

- **HTTP transport only** — `opentelemetry-otlp` has `default-features = false` with `reqwest-blocking-client` explicitly enabled. This avoids the default `tonic` HTTP client which can conflict/panic in `BatchSpanProcessor`. Do not re-enable the default hyper/tonic HTTP client features.
- **Process metrics (Linux)** — `init_process_metrics` reads `/proc/self/status` for memory/thread gauges. These are gated behind `#[cfg(target_os = "linux")]`.
- **Cargo.lock is gitignored** — This is a library crate, so `Cargo.lock` is not committed.
