# speedyspec-wp-token-map

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-token-map.svg)](https://crates.io/crates/speedyspec-wp-token-map)
[![docs.rs](https://docs.rs/speedyspec-wp-token-map/badge.svg)](https://docs.rs/speedyspec-wp-token-map)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's WP_Token_Map in Rust, in WordPress's own layout, and the HTML5 named character references as a compile-time table.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`html`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/html).

## What it gives you

- `TokenMap`: `WP_Token_Map` in Rust, in WordPress's own layout, with `from_array` and `to_array`.
- `html5`, `named_reference` and `named_reference_count`: the HTML5 named character references as a compile-time table.

## Install

```sh
cargo add speedyspec-wp-token-map
```

## Requirements

Rust 1.99.

## Example

Look up named character references, longest match first.

```rust
use speedyspec_wp_token_map::named_reference;

assert_eq!(named_reference(b"amp;x", 0), Some((&b"&"[..], 4)));
assert_eq!(named_reference(b"notin;", 0), Some(("\u{2209}".as_bytes(), 6)));
assert_eq!(named_reference(b"zzz", 0), None);
```

## Where it fits

`speedyspec-wp-html-syntax` decodes character references with it. It depends on `memchr` only. See the guide's
html seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
