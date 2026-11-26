# speedyspec-wp-shortcode-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-shortcode-contracts.svg)](https://crates.io/crates/speedyspec-wp-shortcode-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-shortcode-contracts/badge.svg)](https://docs.rs/speedyspec-wp-shortcode-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's Shortcode API contracts: the plain types, the ports and strategies, and WordPress's pattern strings.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`shortcode`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/shortcode).

## What it gives you

- `TagName` and `TagSet`: shortcode names, checked as WordPress does.
- `ShortcodeMatch`, `MatchGroups` and `ParsedAtts`: what parsing finds.
- The ports and strategies, and WordPress's pattern strings.

## Install

```sh
cargo add speedyspec-wp-shortcode-contracts
```

## Requirements

Rust 1.99.

## Example

A tag name may not hold characters WordPress rejects.

```rust
use speedyspec_wp_shortcode_contracts::TagName;

assert!(TagName::new(b"gallery").is_ok());
assert!(TagName::new(b"bad tag").is_err());
```

## Where it fits

The Shortcode API port in speedyspec-server implements these. See the guide's shortcode seam.

**Ports:** `AttsParser`, `Diagnostics`, `HtmlSplitter`, `KsesAttributes`, `PhpStrings`, `ShortcodeFilters`, `ShortcodeHandler`, `ShortcodeRegistry`, `ShortcodeScanner`, `ShortcodeTagNames`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
