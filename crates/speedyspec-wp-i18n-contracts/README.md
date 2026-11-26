# speedyspec-wp-i18n-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-i18n-contracts.svg)](https://crates.io/crates/speedyspec-wp-i18n-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-i18n-contracts/badge.svg)](https://docs.rs/speedyspec-wp-i18n-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Ports and plain values of the WordPress translations port: file readers, the file probe and the file cache.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`i18n`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/i18n).

## What it gives you

- Ports of the translations seam: file readers, the file probe and the file cache.
- Plain values: `PhpValue`, `WordCountType`, and `php_truthy`, PHP's truthiness.
- `PortError`: how a port fails.

## Install

```sh
cargo add speedyspec-wp-i18n-contracts
```

## Requirements

Rust 1.99.

## Example

`php_truthy` follows PHP: `"0"` is false, `"0.0"` is true.

```rust
use speedyspec_wp_i18n_contracts::{PhpValue, php_truthy};

assert!(!php_truthy(&PhpValue::Str("0".to_owned())));
assert!(php_truthy(&PhpValue::Str("0.0".to_owned())));
assert!(php_truthy(&PhpValue::Str("de_DE".to_owned())));
```

## Where it fits

The translations port in speedyspec-server (`.mo`, `.l10n.php` and the text domain registry) implements these.
See the guide's i18n seam.

**Ports:** `FileProbe`, `LocaleStrings`, `ScriptSources`, `SwitchEffects`, `TranslationFileCache`, `TranslationFileReader`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
