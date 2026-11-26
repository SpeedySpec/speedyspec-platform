# speedyspec-hooks-memory

[![crates.io](https://img.shields.io/crates/v/speedyspec-hooks-memory.svg)](https://crates.io/crates/speedyspec-hooks-memory)
[![docs.rs](https://docs.rs/speedyspec-hooks-memory/badge.svg)](https://docs.rs/speedyspec-hooks-memory)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

In-memory backend for the speedyspec hook contract.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** adapter. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `MemoryHooks`: an in-process `Hooks` backend that passes the contract's conformance suite.

## Install

```sh
cargo add speedyspec-hooks-memory
```

## Requirements

Rust 1.99.

## Example

Register a named callback and filter a value through it.

```rust
use std::rc::Rc;

use speedyspec_hooks::{AddOptions, Callback, CallbackError, Hooks, Value};
use speedyspec_hooks_memory::MemoryHooks;

fn shout(args: &[Value]) -> Result<Value, CallbackError> {
    match args.first() {
        Some(Value::Str(text)) => Ok(Value::Str(text.to_uppercase())),
        other => Ok(other.cloned().unwrap_or(Value::Null)),
    }
}

let hooks = MemoryHooks::new();
let callback: Rc<Callback> = Rc::new(shout);
hooks.add("title", callback, AddOptions::default()).unwrap();
let title = hooks.filter("title", Value::Str("hello".to_owned()), &[]).unwrap();
assert_eq!(title, Value::Str("HELLO".to_owned()));
```

## Where it fits

Implements the `speedyspec-hooks` contract. The CLI and the guide link it, and `speedyspec-php` tests boot PHP
with it.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
