# speedyspec-operation-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-operation-contracts.svg)](https://crates.io/crates/speedyspec-operation-contracts)
[![docs.rs](https://docs.rs/speedyspec-operation-contracts/badge.svg)](https://docs.rs/speedyspec-operation-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The unified operation API's contracts: the neutral value, schemas, types, operations, errors, the caller and the ports. Depends on no SpeedySpec crate.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`operation`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/operation).

## What it gives you

- `Value`: the neutral value every transport maps to and from.
- `Schema`, `TypeSpec` and the operation types: what an operation takes and returns.
- `OperationError` and `Category`: errors every transport can map.
- `Caller`, `Identity` and `PermissionCheck`, and the catalog and mapper ports.

## Install

```sh
cargo add speedyspec-operation-contracts
```

## Requirements

Rust 1.99.

## Example

An error carries a category that each transport maps to its own status.

```rust
use speedyspec_operation_contracts::{Category, OperationError, Value};

let error = OperationError::new("report_missing", Category::NotFound, "No such report.");
assert_eq!(error.category.as_str(), "not_found");
assert_eq!(Category::parse("not_found"), Some(Category::NotFound));
assert_eq!(Value::default(), Value::Null);
```

## Where it fits

REST, XML-RPC and Abilities each expose operations through their transport contracts
(`speedyspec-rest-contracts`, `speedyspec-xmlrpc-contracts`, `speedyspec-abilities-contracts`). The dispatcher
and catalogs live in speedyspec-server. This crate depends on no SpeedySpec crate.

**Ports:** `ContractMapper`, `Extract`, `Hydrate`, `Identity`, `OperationCatalog`, `PermissionCheck`, `TypeCatalog`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
