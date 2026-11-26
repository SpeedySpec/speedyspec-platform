# speedyspec-hooks

[![crates.io](https://img.shields.io/crates/v/speedyspec-hooks.svg)](https://crates.io/crates/speedyspec-hooks)
[![docs.rs](https://docs.rs/speedyspec-hooks/badge.svg)](https://docs.rs/speedyspec-hooks)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Hook contract for speedyspec: the Hooks trait, the Value type and a backend conformance suite. No storage.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `Hooks`: the backend contract (add, remove, has, run, filter, ran), with priority and insertion-order semantics every backend must keep.
- `Value` and `Key`: the values a hook passes, ordered like PHP arrays, with `Opaque` for values that never leave the process.
- `Callback`, `AddOptions`, `CallbackId` and `HookError`: what a hook runs, how it is registered, found and fails.
- `conformance` (feature `conformance`): the suite every backend runs.

## Install

```sh
cargo add speedyspec-hooks
```

## Requirements

Rust 1.99.

## Example

A callback is a named function over `Value`s; `AddOptions` says how it is registered.

```rust
use speedyspec_hooks::{AddOptions, CallbackError, Value};

fn shout(args: &[Value]) -> Result<Value, CallbackError> {
    match args.first() {
        Some(Value::Str(text)) => Ok(Value::Str(text.to_uppercase())),
        _ => Err(CallbackError("shout takes a string".to_owned())),
    }
}

let options = AddOptions { priority: 5, ..AddOptions::default() };
assert_eq!(options.priority, 5);
assert_eq!(shout(&[Value::Str("hi".to_owned())]).unwrap(), Value::Str("HI".to_owned()));
```

## Where it fits

`speedyspec-hooks-memory` is the in-memory backend, and `speedyspec-php` exposes the registry to PHP as
`SpeedySpec\Hooks\Hook`. Bind your own backend by implementing `Hooks` and running the `conformance` suite.

**Ports:** `Hooks`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
