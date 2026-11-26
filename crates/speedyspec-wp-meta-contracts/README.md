# speedyspec-wp-meta-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-meta-contracts.svg)](https://crates.io/crates/speedyspec-wp-meta-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-meta-contracts/badge.svg)](https://docs.rs/speedyspec-wp-meta-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's metadata API contracts: the meta values, the entity and its errors.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** `meta`.

## What it gives you

- `Meta`: the metadata entity, a plain object that knows nothing about storage.
- `MetaType`, `MetaKey`, `MetaId`, `ObjectId` and `SerializedValue`.
- The repository ports, `MetaError`, and a `conformance` suite.

## Install

```sh
cargo add speedyspec-wp-meta-contracts
```

## Requirements

Rust 1.99.

## Example

A meta type knows its table and columns.

```rust
use speedyspec_wp_meta_contracts::MetaType;

let post = MetaType::new("post").expect("a non-empty type");
assert_eq!(post.table_suffix(), "postmeta");
assert_eq!(post.object_column(), "post_id");
assert_eq!(post.cache_group(), "post_meta");
```

## Where it fits

Implementations live in speedyspec-server and bind to the port's key in the service container
(`speedyspec-container`). Bind your own by implementing the trait and binding it under the same key; a PHP
implementation replaces it through the seam's PHP twin.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
