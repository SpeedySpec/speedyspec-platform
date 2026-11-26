# speedyspec-native-stage

[![crates.io](https://img.shields.io/crates/v/speedyspec-native-stage.svg)](https://crates.io/crates/speedyspec-native-stage)
[![docs.rs](https://docs.rs/speedyspec-native-stage/badge.svg)](https://docs.rs/speedyspec-native-stage)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The contract for answering a request in Rust before PHP runs: stages, their outcome and a chain of them.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** `http`.

## What it gives you

- `NativeStage`: answers a request in Rust before PHP runs, or lets it through.
- `StageChain`: the stages in order; the first that answers wins.
- `StageRequest`, `StageOutcome` and `NativeResponse`: what a stage sees (the FastCGI params, never the body) and what it answers.

## Install

```sh
cargo add speedyspec-native-stage
```

## Requirements

Rust 1.99.

## Example

A stage that answers one path and lets every other request through to PHP.

```rust
use std::rc::Rc;

use speedyspec_native_stage::{NativeResponse, NativeStage, StageChain, StageOutcome, StageRequest};

struct Health;

impl NativeStage for Health {
    fn name(&self) -> &str {
        "health"
    }

    fn answer(&self, request: &StageRequest<'_>) -> StageOutcome {
        if request.param("REQUEST_URI") == Some("/health") {
            StageOutcome::Respond(NativeResponse { status: 200, headers: vec![], body: b"ok".to_vec() })
        } else {
            StageOutcome::Continue
        }
    }
}

let mut chain = StageChain::new();
chain.push(Rc::new(Health));
let params = vec![("REQUEST_URI".to_owned(), "/blog".to_owned())];
assert_eq!(chain.answer(&StageRequest { params: &params }), StageOutcome::Continue);
```

## Where it fits

The runtime builds a `StageChain` once per worker. Native replacements of plugins (caches, firewalls, sitemaps)
are stages in speedyspec-server.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
