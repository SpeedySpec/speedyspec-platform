# speedyspec-wp-text-diff-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-text-diff-contracts.svg)](https://crates.io/crates/speedyspec-wp-text-diff-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-text-diff-contracts/badge.svg)](https://docs.rs/speedyspec-wp-text-diff-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Contracts for WordPress's text diff: edit operations, table options, and the differ and line filter ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`text-diff`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/text-diff).

## What it gives you

- `DiffOp`: copy, delete, add and change, over line ranges.
- `SequenceDiffer`, `TextDiffer` (bound under `TEXT_DIFFER`), `LineHtmlFilter` and `WhitespaceNormalizer`.
- `TableOptions`, `RowLabels` and `Title`: how the table renders.

## Install

```sh
cargo add speedyspec-wp-text-diff-contracts
```

## Requirements

Rust 1.99.

## Example

A differ is a strategy; this one only knows equal inputs.

```rust
use speedyspec_wp_text_diff_contracts::{DiffOp, SequenceDiffer};

struct EqualOnly;

impl SequenceDiffer for EqualOnly {
    fn diff(&self, orig: &[&[u8]], fin: &[&[u8]]) -> Vec<DiffOp> {
        vec![DiffOp::Copy { orig: 0..orig.len(), fin: 0..fin.len() }]
    }
}

let lines: [&[u8]; 2] = [b"a", b"b"];
assert_eq!(EqualOnly.diff(&lines, &lines), vec![DiffOp::Copy { orig: 0..2, fin: 0..2 }]);
```

## Where it fits

`speedyspec-wp-text-diff` implements these. See the guide's text-diff seam.

**Ports:** `CheckedLineFilter`, `LineHtmlFilter`, `SequenceDiffer`, `TextDiffHost`, `TextDiffer`, `WhitespaceNormalizer`. **Container keys:** `TEXT_DIFFER`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
