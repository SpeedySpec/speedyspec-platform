# speedyspec-wp-hooks

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-hooks.svg)](https://crates.io/crates/speedyspec-wp-hooks)
[![docs.rs](https://docs.rs/speedyspec-wp-hooks/badge.svg)](https://docs.rs/speedyspec-wp-hooks)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's Hook API (plugin.php and WP_Hook) as commands and queries over a hook store port.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `WpHookApi`: every `plugin.php` function over the `HookStore` port, implementing `HookApi`.
- `WpHook`: `WP_Hook`, ported literally, with `HookIteration` and `PriorityOrder`.
- `CallbackIdentity`: `_wp_filter_build_unique_id()`.
- Each function as a command or query (`messages`), bound in the container by `register_hook_handlers`.

## Install

```sh
cargo add speedyspec-wp-hooks
```

## Requirements

Rust 1.99.

## Example

Add a named filter callback and apply the filter, over the in-memory store.

```rust
use std::rc::Rc;

use speedyspec_wp_hooks::{IgnoreDeprecations, WpHookApi};
use speedyspec_wp_hooks_contracts::{Callback, HookError, HookName, HookValue};
use speedyspec_wp_hooks_memory::MemoryHookStore;

fn append_a(args: &[HookValue]) -> Result<HookValue, HookError> {
    match args.first() {
        Some(HookValue::Str(s)) => Ok(HookValue::Str(format!("{s}-a"))),
        other => Ok(other.cloned().unwrap_or(HookValue::Null)),
    }
}

let api = WpHookApi::new(Rc::new(MemoryHookStore::new()), Rc::new(IgnoreDeprecations));
let title = HookName::from("the_title");
api.add_filter(&title, Callback::rust("append_a", append_a), &HookValue::Int(10), 1).unwrap();
let filtered = api.apply_filters(&title, HookValue::Str("Hello".to_owned()), vec![]).unwrap();
assert_eq!(filtered, HookValue::Str("Hello-a".to_owned()));
```

## Where it fits

Implements `speedyspec-wp-hooks-contracts`' `HookApi`. `speedyspec-wp-hooks-memory` is its store, and
`speedyspec-wp-hooks-php` declares the same functions in PHP over it. See the guide's hooks seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
