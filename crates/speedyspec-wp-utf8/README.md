# speedyspec-wp-utf8

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-utf8.svg)](https://crates.io/crates/speedyspec-wp-utf8)
[![docs.rs](https://docs.rs/speedyspec-wp-utf8/badge.svg)](https://docs.rs/speedyspec-wp-utf8)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's UTF-8 helpers (utf8.php, compat-utf8.php) as of WordPress 7.1.5: scanning, validation, scrubbing and code point counting over bytes.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`text`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/text).

## What it gives you

- `is_valid_utf8` and `scan_utf8`: validation over bytes, with a scalar reference to check against.
- `scrub_utf8`, `check_invalid_utf8` and `has_noncharacters`: WordPress's repair and checks.
- `codepoint_count`, `codepoint_span`, `mb_strlen` and `mb_substr`: counting and slicing as WordPress does.
- `utf8_encode_latin1` and `utf8_decode_latin1`.

## Install

```sh
cargo add speedyspec-wp-utf8
```

## Requirements

Rust 1.99.

## Example

Validate and scrub bytes as WordPress does.

```rust
use speedyspec_wp_utf8::{is_valid_utf8, mb_strlen, scrub_utf8};

assert!(is_valid_utf8("café".as_bytes()));
assert!(!is_valid_utf8(b"caf\xe9"));
assert_eq!(&*scrub_utf8(b"caf\xe9"), "caf\u{fffd}".as_bytes());
assert_eq!(mb_strlen("café".as_bytes(), true), 4);
```

## Where it fits

The formatting functions in speedyspec-server call these. It depends on `simdutf8` only. See the guide's text
seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
