# speedyspec-php-port-macros

[![crates.io](https://img.shields.io/crates/v/speedyspec-php-port-macros.svg)](https://crates.io/crates/speedyspec-php-port-macros)
[![docs.rs](https://docs.rs/speedyspec-php-port-macros/badge.svg)](https://docs.rs/speedyspec-php-port-macros)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

#[php_port]: generates a Rust port trait's PHP twin interface and its PHP adapter.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** none (the platform kernel every seam may use).

## What it gives you

- `#[php_port(name = "PHP\\Interface")]` on a port trait: the trait unchanged, its PHP adapter `Php<Trait>`, and the twin interface's PHP source, built from the trait at compile time so the two cannot drift.
- Use it through `speedyspec-php-ext`, which re-exports it with the runtime pieces the generated code needs.

## Install

```sh
cargo add speedyspec-php-port-macros
```

## Requirements

A static PHP built by `scripts/build-php.sh` at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set; see [`speedyspec-php`](https://docs.rs/speedyspec-php). Rust 1.99.

## Example

The generated code calls into `speedyspec-php-ext`, so the example uses the macro through that crate. It never
boots PHP, but the crate links the static PHP, so the requirements above apply.

```rust
use speedyspec_php_ext::{php_port, twins};

#[php_port(name = "SpeedySpec\\Example\\Counter")]
pub trait Counter {
    fn next(&self, n: i64) -> i64;
}

struct Increment;

impl Counter for Increment {
    fn next(&self, n: i64) -> i64 {
        n + 1
    }
}

fn is_counter(twin: &&speedyspec_php_ext::Twin) -> bool {
    twin.name == "SpeedySpec\\Example\\Counter"
}

fn main() {
    assert_eq!(Increment.next(1), 2);
    assert!(twins().iter().any(is_counter));
}
```

## Where it fits

A seam's PHP crate puts the attribute on each port PHP may implement. The twins area of `speedyspec-php-ext`
declares the interfaces, and `speedyspec-container-php` wraps PHP implementations in the adapters.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
