# speedyspec-wp-hooks-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-hooks-contracts.svg)](https://crates.io/crates/speedyspec-wp-hooks-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-hooks-contracts/badge.svg)](https://docs.rs/speedyspec-wp-hooks-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The Hook API port (plugin.php, as other seams call it) and the values passed through it.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `HookApi`: `plugin.php` as other seams call it (`apply_filters`, `do_action`, `add_filter`, ...).
- `HookName`, `HookKey` and `ArrayKey`: hook names and priorities as PHP array keys, hashed as PHP does.
- `Callback`, `HookCallback`, `CallbackId` and `HookError`: what a hook runs, how it is found again, and how it fails.

## Install

```sh
cargo add speedyspec-wp-hooks-contracts
```

## Requirements

Rust 1.99.

## Example

A Rust callback is a named function, wrapped with `Callback::rust`.

```rust
use speedyspec_wp_hooks_contracts::{Callback, HookCallback, HookError, HookName, HookValue};

fn append_a(args: &[HookValue]) -> Result<HookValue, HookError> {
    match args.first() {
        Some(HookValue::Str(s)) => Ok(HookValue::Str(format!("{s}-a"))),
        other => Ok(other.cloned().unwrap_or(HookValue::Null)),
    }
}

let _callback = Callback::rust("append_a", append_a);
let name = HookName::from("the_title");
assert!(name.key().is_ok());
assert_eq!(append_a.call(&[HookValue::Str("x".to_owned())]).unwrap(), HookValue::Str("x-a".to_owned()));
```

## Where it fits

`speedyspec-wp-hooks` implements `HookApi` (`WpHookApi`). Another seam takes `Rc<dyn HookApi>` from the container,
so it never links the implementation. See the guide's hooks seam.

**Ports:** `HookApi`, `HookCallback`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
