# speedyspec-php-ext

[![crates.io](https://img.shields.io/crates/v/speedyspec-php-ext.svg)](https://crates.io/crates/speedyspec-php-ext)
[![docs.rs](https://docs.rs/speedyspec-php-ext/badge.svg)](https://docs.rs/speedyspec-php-ext)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The speedyspec_wordpress PHP extension's shared parts: PHP twins of Rust ports, their PHP adapters, value conversion and the boundary counter.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform (PHP binding). **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `php_port`: a Rust port trait's PHP twin (an interface) and its PHP adapter (`Php<Trait>`).
- `twins_area`: declares every twin at the start of each request.
- `ToPhp` and `FromPhp`: the values that cross the boundary; `PortError` for a port whose PHP side can fail.
- `boundary_crossings`: counts entries into PHP, so a test can show a Rust path never entered it.

## Install

```sh
cargo add speedyspec-php-ext
```

## Requirements

A static PHP built by `scripts/build-php.sh` at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set; see [`speedyspec-php`](https://docs.rs/speedyspec-php). Rust 1.99.

## Example

Give a port a PHP twin, and check that PHP sees the interface.

```rust
use std::rc::Rc;

use speedyspec_hooks::Value;
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, NativeArea, Php};
use speedyspec_php_ext::{php_port, twins_area};

#[php_port(name = "SpeedySpec\\Example\\Greeter")]
pub trait Greeter {
    fn greet(&self, name: String) -> String;
}

fn main() {
    let areas: Vec<Rc<dyn NativeArea>> = vec![twins_area()];
    let php = Php::boot(BootOptions { hooks: Rc::new(MemoryHooks::new()), preload: &[], areas })
        .expect("PHP boots");
    let twin = php
        .eval("example", "return interface_exists('SpeedySpec\\\\Example\\\\Greeter');")
        .unwrap();
    assert_eq!(twin, Value::Bool(true));
}
```

## Where it fits

Contracts crates stay free of PHP; the seam's PHP crate puts `#[php_port]` on the port so PHP code can implement
it. `speedyspec-container-php` wraps a PHP implementation in the port's adapter, and `speedyspec-wp-hooks-php`
uses a twin for its hook store.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
