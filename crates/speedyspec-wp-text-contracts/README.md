# speedyspec-wp-text-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-text-contracts.svg)](https://crates.io/crates/speedyspec-wp-text-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-text-contracts/badge.svg)](https://docs.rs/speedyspec-wp-text-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The ports WordPress's formatting functions read outside facts through (site settings, translations, locale, diagnostics), apart from any implementation.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`text`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/text).

## What it gives you

- The ports WordPress's formatting functions read outside facts through: `SiteSettings`, `Translator`, `Locale`, `Diagnostics`, `MimeTypes`, `Smilies`, `RandomSource` and more.
- `TextError`.

## Install

```sh
cargo add speedyspec-wp-text-contracts
```

## Requirements

Rust 1.99.

## Example

A fixed locale, as a test adapter of the `Locale` port.

```rust
use speedyspec_wp_text_contracts::{Locale, TextError};

struct German;

impl Locale for German {
    fn locale(&self) -> Result<String, TextError> {
        Ok("de_DE".to_owned())
    }
}

let locale: &dyn Locale = &German;
assert_eq!(locale.locale().unwrap(), "de_DE");
```

## Where it fits

Implementations live in speedyspec-server and are bound in the service container (`speedyspec-container`). Bind
your own by implementing a port and binding it in their place; a PHP implementation replaces it through the
seam's PHP twin.

See the guide's text seam.

**Ports:** `Diagnostics`, `LinkContext`, `Locale`, `MimeTypes`, `RandomSource`, `SiteSettings`, `Smilies`, `TexturizeOverrides`, `Translator`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
