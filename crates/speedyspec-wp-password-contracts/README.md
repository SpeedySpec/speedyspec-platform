# speedyspec-wp-password-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-password-contracts.svg)](https://crates.io/crates/speedyspec-wp-password-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-password-contracts/badge.svg)](https://docs.rs/speedyspec-wp-password-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Contracts for WordPress's password functions: the pluggable ports, the filters, $wp_hasher and bcrypt.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`password`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/password).

## What it gives you

- The pluggable ports: `PasswordHasher`, `PasswordChecker`, `RehashPolicy`, `PasswordGenerator`.
- Their container keys (`PASSWORD_HASHER`, `PASSWORD_CHECKER`, ...), which are also their PHP twins' names.
- `PasswordError`.

## Install

```sh
cargo add speedyspec-wp-password-contracts
```

## Requirements

Rust 1.99.

## Example

A port's key is the name PHP binds its implementation under.

```rust
use speedyspec_wp_password_contracts::{PASSWORD_CHECKER, PASSWORD_HASHER};

assert_eq!(PASSWORD_HASHER, "SpeedySpec\\WpPassword\\PasswordHasher");
assert_eq!(PASSWORD_CHECKER, "SpeedySpec\\WpPassword\\PasswordChecker");
```

## Where it fits

The bcrypt implementation in speedyspec-server binds these keys; `wp_hash_password()` and friends are its
pluggable functions, so PHP may rebind each one. See the guide's password seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
