# speedyspec-wp-html-syntax

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-html-syntax.svg)](https://crates.io/crates/speedyspec-wp-html-syntax)
[![docs.rs](https://docs.rs/speedyspec-wp-html-syntax/badge.svg)](https://docs.rs/speedyspec-wp-html-syntax)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's HTML API syntax pieces: WP_HTML_Decoder, the HTML5 named character references, start-tag attributes.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`html`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/html).

## What it gives you

- `decode`, `decode_text_node` and `decode_attribute`: `WP_HTML_Decoder`.
- `read_character_reference` and `code_point_to_utf8`.
- `start_tag_attributes`: a start tag's attributes, deduplicated as the HTML API does.

## Install

```sh
cargo add speedyspec-wp-html-syntax
```

## Requirements

Rust 1.99.

## Example

Decode character references in text and attribute values.

```rust
use speedyspec_wp_html_syntax::{decode_attribute, decode_text_node};

assert_eq!(&*decode_text_node(b"Fish &amp; Chips"), b"Fish & Chips");
assert_eq!(&*decode_attribute(b"a&lt;b"), b"a<b");
```

## Where it fits

`speedyspec-wp-kses-lexical` and the HTML API port in speedyspec-server build on it. See the guide's html seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
