# speedyspec-wp-kses-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-kses-contracts.svg)](https://crates.io/crates/speedyspec-wp-kses-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-kses-contracts/badge.svg)](https://docs.rs/speedyspec-wp-kses-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's kses sanitizer as contracts: the allowlist types and the ports any kses implementation fills.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`html`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/html).

## What it gives you

- The allowlist types: allowed HTML, `AttrRules`, `AttrRule` and `RuleValue`.
- `Attr`, `KsesError` and `Payload`.
- The ports any kses implementation fills.

## Install

```sh
cargo add speedyspec-wp-kses-contracts
```

## Requirements

Rust 1.99.

## Example

An attribute is required only when its `required` rule is `true`.

```rust
use speedyspec_wp_kses_contracts::{AttrRule, AttrRules, RuleValue};

let required = AttrRules::Rules(vec![AttrRule::Other(b"required".to_vec().into(), RuleValue::Bool(true))]);
assert!(required.is_required());
assert!(!AttrRules::Allowed.is_required());
```

## Where it fits

`speedyspec-wp-kses-lexical` holds kses's pure string functions over these types, and the kses port in
speedyspec-server implements the rest. See the guide's html seam.

**Ports:** `AllowedHtmlSource`, `AttributeEscaper`, `HtmlApiSanitizer`, `Kses`, `KsesLexical`, `OperatingMode`, `SafeCss`, `UnfilteredHtmlPolicy`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
