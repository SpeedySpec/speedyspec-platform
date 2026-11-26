# speedyspec-routing

[![crates.io](https://img.shields.io/crates/v/speedyspec-routing.svg)](https://crates.io/crates/speedyspec-routing)
[![docs.rs](https://docs.rs/speedyspec-routing/badge.svg)](https://docs.rs/speedyspec-routing)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Routing contracts: routes, resolutions, route data, the table store, dispatch strategies and WordPress's routing concepts.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** `http`.

## What it gives you

- `RouteId`, `RouteName`, `Params` and `Fingerprint`: routes and what a match carries.
- `RouteData`, `StaticRoute` and `DynamicRoute`: the route table, and `RouteTableStore`, where it is kept.
- `DispatchStrategy` and `CompiledDispatch`: how a table is matched.
- WordPress's routing concepts: rewrite rules, query vars and resolutions.

## Install

```sh
cargo add speedyspec-routing
```

## Requirements

Rust 1.99.

## Example

Route names and parameters are plain values.

```rust
use speedyspec_routing::{Params, RouteName};

let name = RouteName::new("single-post");
assert_eq!(name.as_str(), "single-post");

let params = Params(vec![("year".to_owned(), "2026".to_owned())]);
assert_eq!(params.get("year"), Some("2026"));
assert_eq!(params.get("month"), None);
```

## Where it fits

The router and its table stores implement these ports in speedyspec-server; `conformance` is the suite a
dispatch strategy runs.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
