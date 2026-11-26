# speedyspec-cache

[![crates.io](https://img.shields.io/crates/v/speedyspec-cache.svg)](https://crates.io/crates/speedyspec-cache)
[![docs.rs](https://docs.rs/speedyspec-cache/badge.svg)](https://docs.rs/speedyspec-cache)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The Caching Store contract: byte values in groups, with expiry. No implementations.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`object-cache`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/object-cache).

## What it gives you

- `CachingStore`: byte values in groups, with expiry, `add`/`replace` semantics and memcached-style counters.
- `CacheError`: how a store fails.

## Install

```sh
cargo add speedyspec-cache
```

## Requirements

Rust 1.99.

## Example

A store is any type that implements `CachingStore`; code that uses one takes `&dyn CachingStore`.

```rust
use speedyspec_cache::{CacheError, CachingStore};

fn cached_title(store: &dyn CachingStore, id: &str) -> Result<Option<String>, CacheError> {
    Ok(store.get("titles", id)?.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
}

let _: fn(&dyn CachingStore, &str) -> Result<Option<String>, CacheError> = cached_title;
```

## Where it fits

The in-memory, Redis and memcached stores implement it in speedyspec-server, and the object cache is built on it.
Bind your own store by implementing `CachingStore`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
