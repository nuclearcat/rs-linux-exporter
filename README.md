# rs-linux-exporter
Prometheus-compatible Linux metrics exporter written in Rust.

## Overview
rs-linux-exporter aims to provide the most important Linux host metrics with a
simple, safe, and fast Rust implementation. The goal is to avoid external
dependencies (especially tools and unsafe languages) to keep the binary small
and the software reliable.

## Goals
- Provide essential Linux metrics for Prometheus scraping
- Keep the binary size small and startup time fast
- Avoid external tool dependencies and unsafe language bindings
- Favor correctness, safety, and predictable performance

## Planned metrics (initial focus)
- System uptime
- CPU usage and load averages
- Memory usage (total/free/available/buffers/cache)
- Disk usage (total/used/free per filesystem)
- Disk I/O (bytes, ops, time per device)
- Network I/O (bytes, packets, errors per interface)
- Process count and basic system limits

## Contributing
Please check dependency freshness and binary size when submitting changes.

To install helper tools:
- `cargo install cargo-outdated`
- `cargo install cargo-bloat`

Recommended checks:
- `cargo outdated`
- `cargo bloat --release`

## Status
This project is a work in progress.
