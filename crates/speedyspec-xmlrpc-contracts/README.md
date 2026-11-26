# speedyspec-xmlrpc-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-xmlrpc-contracts.svg)](https://crates.io/crates/speedyspec-xmlrpc-contracts)
[![docs.rs](https://docs.rs/speedyspec-xmlrpc-contracts/badge.svg)](https://docs.rs/speedyspec-xmlrpc-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The XML-RPC transport's contracts: the binding, method calls and responses, faults, and the mapping, fault, hooks and login ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`xmlrpc`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/xmlrpc).

## What it gives you

- `XmlRpcBinding` and `XmlRpcParam`: the method that serves an operation and its positional parameters.
- `MethodCall`, `MethodResponse`, `Fault` and `MethodEntry`: what the codec reads and writes, and the method table.
- Ports: `XmlRpcMapping` and `FaultMapping` (strategies), `XmlRpcLogin`, and `XmlRpcHooks` (`NoXmlRpcHooks` keeps WordPress's defaults).

## Install

```sh
cargo add speedyspec-xmlrpc-contracts
```

## Requirements

Rust 1.99.

## Example

A binding round-trips through the neutral value it is stored as.

```rust
use speedyspec_xmlrpc_contracts::{Fault, XmlRpcBinding, XmlRpcParam};

let binding = XmlRpcBinding {
    method: "acme.editReport".to_owned(),
    params: vec![XmlRpcParam::Username, XmlRpcParam::Password, XmlRpcParam::Field("id".to_owned())],
};
assert_eq!(XmlRpcBinding::from_value(&binding.to_value()), Ok(binding));
assert_eq!(Fault::new(404, "Not found").code, 404);
```

## Where it fits

The XML-RPC transport in speedyspec-server implements these ports (`PhpXmlRpcLogin` calls `wp_authenticate()`).
See the guide's xmlrpc seam.

**Ports:** `FaultMapping`, `XmlRpcHooks`, `XmlRpcLogin`, `XmlRpcMapping`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
