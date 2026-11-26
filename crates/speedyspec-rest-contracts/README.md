# speedyspec-rest-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-rest-contracts.svg)](https://crates.io/crates/speedyspec-rest-contracts)
[![docs.rs](https://docs.rs/speedyspec-rest-contracts/badge.svg)](https://docs.rs/speedyspec-rest-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The REST transport's contracts: bindings, the REST request and response, links, and the RestHooks port.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`rest-api`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/rest-api).

## What it gives you

- `RestBinding` and `RestMethod`: where an operation is served, as a WordPress route pattern.
- `RestRequest`, `RestResponse` and `RestLink`: the REST request and response.
- `RestHooks` and `RestOutcome`: the REST API's filters and actions, as a port.

## Install

```sh
cargo add speedyspec-rest-contracts
```

## Requirements

Rust 1.99.

## Example

A binding names its namespace, route and methods.

```rust
use speedyspec_rest_contracts::{RestBinding, RestMethod};

let binding = RestBinding {
    namespace: "acme/v1".to_owned(),
    route: "/reports/(?P<id>\\d+)".to_owned(),
    methods: vec![RestMethod::Get],
};
assert_eq!(binding.full_route(), "/acme/v1/reports/(?P<id>\\d+)");
assert!(binding.is_versioned());
assert_eq!(RestBinding::from_value(&binding.to_value()).unwrap(), binding);
```

## Where it fits

The REST transport in speedyspec-server implements these, over `speedyspec-operation-contracts` and
`speedyspec-transport-contracts`. See the guide's rest-api seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
