# speedyspec-wp-kses-lexical

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-kses-lexical.svg)](https://crates.io/crates/speedyspec-wp-kses-lexical)
[![docs.rs](https://docs.rs/speedyspec-wp-kses-lexical/badge.svg)](https://docs.rs/speedyspec-wp-kses-lexical)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

kses.php's pure string functions (entities, protocols, attribute parsing) over fast byte matchers.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`html`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/html).

## What it gives you

- Entities: `normalize_entities`, `decode_entities` and `html_entity_decode`.
- Protocols: `bad_protocol` and its single-pass forms.
- Attributes: parsing and `check_attr_val`.
- Text: `no_null`, `stripslashes`, `array_lc`, and `scan` for tags.

## Install

```sh
cargo add speedyspec-wp-kses-lexical
```

## Requirements

Rust 1.99.

## Example

Strip disallowed protocols, and null bytes.

```rust
use speedyspec_wp_kses_contracts::Protocols;
use speedyspec_wp_kses_lexical::{bad_protocol, no_null};

let allowed = Protocols(vec![b"http".to_vec().into(), b"https".to_vec().into()]);
assert_eq!(&*bad_protocol(b"javascript:alert(1)", &allowed), b"alert(1)");
assert_eq!(&*bad_protocol(b"https://example.com", &allowed), b"https://example.com");
assert_eq!(&*no_null(b"a\0b", false), b"ab");
```

## Where it fits

Implements the pure string parts of `speedyspec-wp-kses-contracts`; the kses port in speedyspec-server builds the
rest on it. See the guide's html seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
