# speedyspec-wp-hooks-memory

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-hooks-memory.svg)](https://crates.io/crates/speedyspec-wp-hooks-memory)
[![docs.rs](https://docs.rs/speedyspec-wp-hooks-memory/badge.svg)](https://docs.rs/speedyspec-wp-hooks-memory)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The in-memory hook store, with worker-lifetime and request-lifetime partitions.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** adapter. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `MemoryHookStore`: the `HookStore` port in memory, with a worker partition (registered before any request) and a request partition.
- `start_request` and `end_request`: ending a request drops what it registered, its run counts and its current-hook stack.

## Install

```sh
cargo add speedyspec-wp-hooks-memory
```

## Requirements

Rust 1.99.

## Example

A worker callback survives the end of a request.

```rust
use std::rc::Rc;

use speedyspec_wp_hooks::{IgnoreDeprecations, WpHookApi};
use speedyspec_wp_hooks_contracts::{Callback, HasFilterResult, HookError, HookName, HookValue};
use speedyspec_wp_hooks_memory::MemoryHookStore;

fn keep(args: &[HookValue]) -> Result<HookValue, HookError> {
    Ok(args.first().cloned().unwrap_or(HookValue::Null))
}

let store = Rc::new(MemoryHookStore::new());
let api = WpHookApi::new(store.clone(), Rc::new(IgnoreDeprecations));
let init = HookName::from("init");
api.add_filter(&init, Callback::rust("keep", keep), &HookValue::Int(10), 1).unwrap();
store.start_request();
store.end_request();
assert_eq!(api.has_filter(&init, None, None).unwrap(), HasFilterResult::Any);
```

## Where it fits

The reference store behind `speedyspec-wp-hooks`. A PHP store can replace it through the `HookStore` port's twin
in `speedyspec-wp-hooks-php`. See the guide's hooks seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
