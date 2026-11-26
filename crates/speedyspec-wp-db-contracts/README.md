# speedyspec-wp-db-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-db-contracts.svg)](https://crates.io/crates/speedyspec-wp-db-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-db-contracts/badge.svg)](https://docs.rs/speedyspec-wp-db-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The wpdb port (wpdb, as other seams call it), its values and its errors.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** `database`.

## What it gives you

- `Wpdb`: the `wpdb` port other seams call.
- `WpValue`, `WpRow`, `QueryOutcome` and `Field`: what a query returns, as `wpdb` gives it to PHP.
- `WpDbError` and `take_hook_error`.

## Install

```sh
cargo add speedyspec-wp-db-contracts
```

## Requirements

Rust 1.99.

## Example

A query's outcome as `wpdb::query()` returns it.

```rust
use speedyspec_wp_db_contracts::{QueryOutcome, WpValue};

let outcome = QueryOutcome::Affected { rows: 3, insert_id: 9 };
assert_eq!(outcome.to_wp(), WpValue::Int(3));
```

## Where it fits

The `wpdb` port in speedyspec-server implements `Wpdb` over the data engines (`speedyspec-data`). Options,
posts and meta take `Rc<dyn Wpdb>` from the container.

**Ports:** `Wpdb`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
