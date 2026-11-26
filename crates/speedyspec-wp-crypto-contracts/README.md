# speedyspec-wp-crypto-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-crypto-contracts.svg)](https://crates.io/crates/speedyspec-wp-crypto-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-crypto-contracts/badge.svg)](https://docs.rs/speedyspec-wp-crypto-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Contracts for WordPress's sodium functions: the hash, signature, encoding, clock, translation and filter ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`password`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/password).

## What it gives you

- Ports for WordPress's sodium functions: `GenericHash`, `DetachedVerifier`, `Base64UrlNoPad`, `Argon2`, `RandomBytes` and `CryptoLibrary`.
- The outside facts they read: `Clock`, `FileHasher`, `Translator` and `TrustedKeysFilter`.
- Sizes such as `SIGN_BYTES` and `SIGN_PUBLICKEY_BYTES`.

## Install

```sh
cargo add speedyspec-wp-crypto-contracts
```

## Requirements

Rust 1.99.

## Example

A port is a trait; a fixed clock is a test adapter.

```rust
use speedyspec_wp_crypto_contracts::{Clock, SIGN_BYTES};

struct Fixed(i64);

impl Clock for Fixed {
    fn now(&self) -> i64 {
        self.0
    }
}

let clock: &dyn Clock = &Fixed(1_700_000_000);
assert_eq!(clock.now(), 1_700_000_000);
assert_eq!(SIGN_BYTES, 64);
```

## Where it fits

Implementations live in speedyspec-server and are bound in the service container (`speedyspec-container`). Bind
your own by implementing a port and binding it in their place; a PHP implementation replaces it through the
seam's PHP twin.

**Ports:** `Argon2`, `Base64UrlNoPad`, `Clock`, `CryptoLibrary`, `DetachedVerifier`, `FileHasher`, `GenericHash`, `RandomBytes`, `Translator`, `TrustedKeysFilter`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
