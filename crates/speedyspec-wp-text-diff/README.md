# speedyspec-wp-text-diff

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-text-diff.svg)](https://crates.io/crates/speedyspec-wp-text-diff)
[![docs.rs](https://docs.rs/speedyspec-wp-text-diff/badge.svg)](https://docs.rs/speedyspec-wp-text-diff)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's wp_text_diff(): a port of Horde's native diff engine and WordPress's table renderer, with SIMD string work.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`text-diff`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/text-diff).

## What it gives you

- `TextDiff`: `wp_text_diff()`, over Horde's native diff engine (`HordeSequenceDiffer`) and WordPress's table renderer.
- `inline_diff`, `split_words` and `html_special_chars`: the pieces, with SIMD string work (`simd`).
- `WordPressTextDiffer`: the `TextDiffer` port of `speedyspec-wp-text-diff-contracts`.

## Install

```sh
cargo add speedyspec-wp-text-diff
```

## Requirements

Rust 1.99.

## Example

Equal texts render nothing; different texts render a table.

```rust
use speedyspec_wp_text_diff::TextDiff;
use speedyspec_wp_text_diff::contracts::{NoLineFilter, RowLabels, TableOptions};

let diff = TextDiff::default();
let (options, labels) = (TableOptions::default(), RowLabels::default());
assert_eq!(diff.table(b"same", b"same", &options, &labels, &NoLineFilter).unwrap(), None);
let table = diff.table(b"old line", b"new line", &options, &labels, &NoLineFilter).unwrap().unwrap();
assert!(table.starts_with(b"<table"));
```

## Where it fits

Implements `speedyspec-wp-text-diff-contracts`. The server's `speedyspec-wp-text-diff-php` exposes it to PHP and
replays the text-diff compat area against it. See the guide's text-diff seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
