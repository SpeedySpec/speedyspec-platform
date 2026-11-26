# speedyspec-wp-cron-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-cron-contracts.svg)](https://crates.io/crates/speedyspec-wp-cron-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-cron-contracts/badge.svg)](https://docs.rs/speedyspec-wp-cron-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's cron contracts: the cron array as a plain value, the ports the scheduling core needs, and its outcomes.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`cron`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/cron).

## What it gives you

- `CronArray`, `ScheduledEvent`, `EventKey` and `EventData`: the `cron` option as a plain value, junk and order kept.
- The ports the scheduling core needs, and its outcomes (`CronResult`, `RunSummary`, `SaveOutcome`).
- `ksort_compare`: PHP's `ksort()` order for the timestamps.

## Install

```sh
cargo add speedyspec-wp-cron-contracts
```

## Requirements

Rust 1.99.

## Example

An empty cron array round-trips through the hook value it is stored as.

```rust
use speedyspec_hooks::Value;
use speedyspec_wp_cron_contracts::CronArray;

let empty = CronArray::from_value(Value::Array(vec![]));
assert!(empty.is_empty());
assert!(empty.events().is_empty());
assert_eq!(empty.to_value(), Value::Array(vec![]));
```

## Where it fits

The cron port in speedyspec-server implements `wp_schedule_event()` and the rest of `cron.php` over these
contracts. See the guide's cron seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
