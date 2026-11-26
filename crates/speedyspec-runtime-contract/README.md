# speedyspec-runtime-contract

[![crates.io](https://img.shields.io/crates/v/speedyspec-runtime-contract.svg)](https://crates.io/crates/speedyspec-runtime-contract)
[![docs.rs](https://docs.rs/speedyspec-runtime-contract/badge.svg)](https://docs.rs/speedyspec-runtime-contract)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

What speedyspec-server and speedyspec-php-runtime agree on: the worker spec, the per-request report and the contract version.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** none (the platform kernel every seam may use).

## What it gives you

- `WorkerSpec`: what the server tells a PHP worker to run (app, socket, release, `php.ini` settings), with `check`.
- `WorkerReport`: what a worker reports after each request, as a prefixed `FCGI_STDERR` record.
- `RuntimeManifest` and `CONTRACT_MAJOR`: which contract versions a runtime speaks, and its PHP, sodium and FFmpeg versions.

## Install

```sh
cargo add speedyspec-runtime-contract
```

## Requirements

Rust 1.99.

## Example

A worker report round-trips through the record it is sent as.

```rust
use speedyspec_runtime_contract::{REPORT_PREFIX, WorkerReport};

let report = WorkerReport { memory: 4 << 20, requests: 12, poisoned: false, next_cron: Some(1_700_000_000) };
let record = report.encode();
assert!(record.starts_with(REPORT_PREFIX));
assert_eq!(WorkerReport::decode(&record), Some(report));
```

## Where it fits

speedyspec-server and `speedyspec-php-runtime` both depend on it, so each checks the other speaks a contract
major it knows before a worker starts.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
