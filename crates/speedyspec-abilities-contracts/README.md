# speedyspec-abilities-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-abilities-contracts.svg)](https://crates.io/crates/speedyspec-abilities-contracts)
[![docs.rs](https://docs.rs/speedyspec-abilities-contracts/badge.svg)](https://docs.rs/speedyspec-abilities-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The Abilities transport's contracts: the ability binding and annotations, and the mapping, hooks and steps ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`ability`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/ability).

## What it gives you

- `AbilityBinding` and `Annotations`: how an operation is exposed as an ability.
- `is_ability_name` and `is_category_slug`: WordPress's naming rules.
- The mapping, hooks and steps ports.

## Install

```sh
cargo add speedyspec-abilities-contracts
```

## Requirements

Rust 1.99.

## Example

Ability names are `namespace/name`.

```rust
use speedyspec_abilities_contracts::{AbilityBinding, is_ability_name};

assert!(is_ability_name("acme/edit-report"));
assert!(!is_ability_name("edit-report"));

let binding = AbilityBinding::default();
assert_eq!(AbilityBinding::from_value(&binding.to_value()), Ok(binding));
```

## Where it fits

The Abilities transport in speedyspec-server implements these ports over `speedyspec-operation-contracts`. See
the guide's ability seam.

**Ports:** `AbilityHooks`, `AbilityMapping`, `AbilitySteps`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
