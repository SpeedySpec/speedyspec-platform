# speedyspec-wp-object-cache-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-object-cache-contracts.svg)](https://crates.io/crates/speedyspec-wp-object-cache-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-object-cache-contracts/badge.svg)](https://docs.rs/speedyspec-wp-object-cache-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The object cache port (wp_cache_*, as other seams call it) and its errors.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`object-cache`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/object-cache).

## What it gives you

- `ObjectCache`: the `wp_cache_*` port other seams call.
- `get_with`: read through the cache, filling it on a miss.
- `ObjectCacheError`.

## Install

```sh
cargo add speedyspec-wp-object-cache-contracts
```

## Requirements

Rust 1.99.

## Example

Other seams hold the port as `Rc<dyn ObjectCache>`.

```rust
use std::rc::Rc;

use speedyspec_wp_object_cache_contracts::ObjectCache;

fn holds(cache: Option<Rc<dyn ObjectCache>>) -> bool {
    cache.is_some()
}

assert!(!holds(None));
```

## Where it fits

Implementations live in speedyspec-server and bind to the port's key in the service container
(`speedyspec-container`). Bind your own by implementing the trait and binding it under the same key; a PHP
implementation replaces it through the seam's PHP twin.

See the guide's object-cache seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
