# speedyspec-container-php

[![crates.io](https://img.shields.io/crates/v/speedyspec-container-php.svg)](https://crates.io/crates/speedyspec-container-php)
[![docs.rs](https://docs.rs/speedyspec-container-php/badge.svg)](https://docs.rs/speedyspec-container-php)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The service container from PHP: SpeedySpec\Container\Container, PHP bindings as request-long overrides of Rust ports, and PHP autowiring.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform (PHP binding). **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `ContainerArea`: the native area that adds `SpeedySpec\Container\Container` to PHP, a facade over the Rust `Container`.
- `ContainerArea::port`: wraps a PHP object bound to a port's key in that port's PHP adapter, so Rust callers get the PHP implementation from `make_port`.
- `ForeignValue`: a PHP value held in the container. Everything PHP registers has request lifetime.
- `PhpAutowirer` and `install_php_bindings`: PHP class autowiring, and bindings read from configuration.

## Install

```sh
cargo add speedyspec-container-php
```

## Requirements

A static PHP built by `scripts/build-php.sh` at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set; see [`speedyspec-php`](https://docs.rs/speedyspec-php). Rust 1.99.

## Example

Boot PHP with the container area, bind a value from PHP and read it back from PHP.

```rust
use std::rc::Rc;

use speedyspec_container::Container;
use speedyspec_container_php::ContainerArea;
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, NativeArea, Php};
use speedyspec_php_ext::Value;

let container = Rc::new(Container::new());
let areas: Vec<Rc<dyn NativeArea>> = vec![Rc::new(ContainerArea::new("example", container.clone()))];
let php = Php::boot(BootOptions { hooks: Rc::new(MemoryHooks::new()), preload: &[], areas })
    .expect("PHP boots");
let exists = php
    .eval("example", "return class_exists('SpeedySpec\\\\Container\\\\Container');")
    .unwrap();
assert_eq!(exists, Value::Bool(true));
```

## Where it fits

The runtime adds `ContainerArea` to the `speedyspec_wordpress` extension, so PHP code can bind and resolve ports by
the same keys as Rust. A PHP binding overrides the Rust one until the request ends. Ports get their PHP twins from
`#[php_port]` (`speedyspec-php-ext`). See the guide's container seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
