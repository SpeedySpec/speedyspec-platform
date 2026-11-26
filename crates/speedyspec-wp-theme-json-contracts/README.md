# speedyspec-wp-theme-json-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-theme-json-contracts.svg)](https://crates.io/crates/speedyspec-wp-theme-json-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-theme-json-contracts/badge.svg)](https://docs.rs/speedyspec-wp-theme-json-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's theme.json contracts: the typed model, the value objects, the schema tables and the ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`theme`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/theme).

## What it gives you

- `model`: the typed theme.json model and its value objects.
- `schema`: `WP_Theme_JSON`'s schema tables, embedded from the release `schema::DATA_RELEASE` names.
- `Origin`, `ByOrigin` and `Leaf`; `ports`; `ThemeJsonError`.

## Install

```sh
cargo add speedyspec-wp-theme-json-contracts
```

## Requirements

Rust 1.99.

## Example

Origins are WordPress's, in WordPress's order.

```rust
use speedyspec_wp_theme_json_contracts::Origin;
use speedyspec_wp_theme_json_contracts::schema::DATA_RELEASE;

assert_eq!(Origin::parse(b"theme"), Some(Origin::Theme));
assert_eq!(Origin::Custom.as_str(), "custom");
assert!(!DATA_RELEASE.is_empty());
```

## Where it fits

The theme.json port in speedyspec-server implements these ports, and checks that `DATA_RELEASE` matches its
compat area's WordPress release. See the guide's theme seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
