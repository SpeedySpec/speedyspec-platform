# speedyspec-wp-html-split

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-html-split.svg)](https://crates.io/crates/speedyspec-wp-html-split)
[![docs.rs](https://docs.rs/speedyspec-wp-html-split/badge.svg)](https://docs.rs/speedyspec-wp-html-split)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's wp_html_split() in Rust: splits HTML into text and tag chunks with get_html_split_regex()'s exact semantics, without allocating.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`html`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/html).

## What it gives you

- `wp_html_split` and `split`: text and tag chunks with `get_html_split_regex()`'s exact semantics, without allocating.
- `split_with` and `Escapes`: whether CDATA counts as an escape.
- `HTML_SPLIT_REGEX`: WordPress's pattern, for reference.

## Install

```sh
cargo add speedyspec-wp-html-split
```

## Requirements

Rust 1.99.

## Example

Split HTML into text and tag chunks.

```rust
use speedyspec_wp_html_split::wp_html_split;

let chunks = wp_html_split(b"abcd <!-- <html> --> efgh");
assert_eq!(chunks, vec![&b"abcd "[..], b"<!-- <html> -->", b" efgh"]);
```

## Where it fits

The formatting functions in speedyspec-server (`wptexturize()`, `wp_replace_in_html_tags()`) use it. It depends
on `memchr` only. See the guide's html seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
