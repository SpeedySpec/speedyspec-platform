# speedyspec-wp-meta-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-meta-contracts.svg)](https://crates.io/crates/speedyspec-wp-meta-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-meta-contracts/badge.svg)](https://docs.rs/speedyspec-wp-meta-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's metadata API contracts: the meta values, the entity and its errors.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`meta`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/meta).

## What it gives you

- `Meta`: the metadata entity, a plain object that knows nothing about storage, and `MetaRow`, a row as
  `update_meta_cache()` reads it (no meta id).
- `MetaType`, `MetaKey`, `MetaId`, `ObjectId` and `SerializedValue`.
- `Metadata`: the metadata API, one method per `meta.php` function, with `UpdateOutcome` and `MetadataError`.
- `MetaKeys` (a read port over the registered keys and the subtypes they sit under) and `RegisteredMetaKey`.
- The repository ports, one `$wpdb` statement per method (`MetaType::object_ids_query` builds the one
  `wp_delete_post()` and its siblings run), `Installing`, `MetaError`, and a `conformance` suite.

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

Implementations live in speedyspec-server and are bound in the service container (`speedyspec-container`). Bind
your own by implementing a port and binding it in their place; a PHP implementation replaces it through the
seam's PHP twin.

**Ports:** `Callables`, `DefaultSchemaCheck`, `Installing`, `MetaKeys`, `Metadata`, `MetaRepositories`, `MetaRepository`, `ObjectSubtypes`, `ProtectedMeta`, `RevisionSupport`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
