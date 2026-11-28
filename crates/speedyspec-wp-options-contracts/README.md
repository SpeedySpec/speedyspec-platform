# speedyspec-wp-options-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-options-contracts.svg)](https://crates.io/crates/speedyspec-wp-options-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-options-contracts/badge.svg)](https://docs.rs/speedyspec-wp-options-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's options API contracts: the option API other seams call, its errors and the typed reader.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`options`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/options).

## What it gives you

- `Options`: `get_option`, `add_option`, `update_option` and `delete_option`, with WordPress's arguments, plus
  `update_option_if`, `stored_bytes`, `forget_cached` and `serialized_form` for compare-and-swap writers.
- `ConditionalWrite`: what a compare-and-swap `update_option_if` did.
- `OptionsError`, which keeps a callback's PHP exception whole.
- `TypedOptions` and `FromOptionValue`: typed reads (`HookValue`, `String`, `i64`, `bool`) over any `Options`.
- `Autoload`, `AutoloadRequest` and `DEFAULT_AUTOLOAD_VALUES`, and PHP's `(string)` and `empty()` for option values.

## Install

```sh
cargo add speedyspec-wp-options-contracts
```

## Requirements

Rust 1.99.

## Example

A stored `autoload` value reads back as the column holds it.

```rust
use speedyspec_wp_options_contracts::Autoload;

assert_eq!(Autoload::parse("auto-on"), Autoload::AutoOn);
assert_eq!(Autoload::parse("legacy").as_str(), "legacy");
```

## Where it fits

Implementations live in speedyspec-server and are bound in the service container (`speedyspec-container`). Bind
your own by implementing a port and binding it in their place; a PHP implementation replaces it through the
seam's PHP twin.

**Ports:** `Options`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
