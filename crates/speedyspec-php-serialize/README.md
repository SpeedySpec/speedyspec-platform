# speedyspec-php-serialize

[![crates.io](https://img.shields.io/crates/v/speedyspec-php-serialize.svg)](https://crates.io/crates/speedyspec-php-serialize)
[![docs.rs](https://docs.rs/speedyspec-php-serialize/badge.svg)](https://docs.rs/speedyspec-php-serialize)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

PHP's serialize format in Rust: serialize, unserialize, maybe_serialize, and conversions to hook values.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `serialize` and `unserialize`: PHP's format, byte for byte, over `PhpValue` and `PhpKey`.
- `maybe_serialize`, `maybe_unserialize` and `is_serialized`: WordPress's helpers.
- `to_hook_value` and `from_hook_value`: conversions to the Hook API's `Value`.

## Install

```sh
cargo add speedyspec-php-serialize
```

## Requirements

Rust 1.99.

## Example

Serialize an array, check it, and read it back.

```rust
use speedyspec_php_serialize::{PhpKey, PhpValue, is_serialized, serialize, unserialize};

let value = PhpValue::Array(vec![(PhpKey::Str(b"a".to_vec()), PhpValue::Int(1))]);
let bytes = serialize(&value);
assert_eq!(bytes, b"a:1:{s:1:\"a\";i:1;}".to_vec());
assert!(is_serialized(&bytes, true));
assert_eq!(unserialize(&bytes).unwrap(), value);
```

## Where it fits

Options, metadata and theme.json values are stored serialized, so the contracts for those seams
(`speedyspec-wp-meta-contracts`, `speedyspec-wp-posts-contracts`, `speedyspec-wp-theme-json-contracts`) use these
types. No PHP runtime is involved.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
