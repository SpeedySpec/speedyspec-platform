# speedyspec-transport-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-transport-contracts.svg)](https://crates.io/crates/speedyspec-transport-contracts)
[![docs.rs](https://docs.rs/speedyspec-transport-contracts/badge.svg)](https://docs.rs/speedyspec-transport-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The unified operation API's transport contracts: the Transport port and its default mapping, bindings, exposure lookup, authentication, codecs and hook stages.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`operation`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/operation).

## What it gives you

- `TransportName`: a transport's name, checked.
- The `Transport` port and its default mapping, bindings, exposure lookup, authentication, codecs and hook stages.

## Install

```sh
cargo add speedyspec-transport-contracts
```

## Requirements

Rust 1.99.

## Example

A transport name is lowercase letters, digits and hyphens.

```rust
use speedyspec_transport_contracts::TransportName;

assert_eq!(TransportName::parse("rest").unwrap().as_str(), "rest");
assert!(TransportName::parse("Not Valid").is_err());
```

## Where it fits

Each transport (REST, XML-RPC, Abilities) implements these ports in speedyspec-server, over
`speedyspec-operation-contracts`.

**Ports:** `Authenticator`, `BindingStore`, `Codec`, `ExposureLookup`, `StageHooks`, `Transport`, `TransportHooks`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
