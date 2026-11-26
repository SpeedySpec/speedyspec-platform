# speedyspec-php

[![crates.io](https://img.shields.io/crates/v/speedyspec-php.svg)](https://crates.io/crates/speedyspec-php)
[![docs.rs](https://docs.rs/speedyspec-php/badge.svg)](https://docs.rs/speedyspec-php)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Embedded, statically linked PHP interpreter for speedyspec.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `Php::boot`: starts the embedded interpreter once per process, with the hook registry and native areas in `BootOptions`.
- `Php::eval`, `Php::call`, `Php::require` and `Php::serve`: run code, call callables, load files and serve a `WebRequest`.
- `NativeArea`: a set of native functions and classes, with request start and end hooks.
- `DOCS_RS`: `build.rs` links nothing when it is set, so docs.rs builds the docs without a PHP.

## Install

```sh
cargo add speedyspec-php
```

## Requirements

A static PHP built by [`scripts/build-php.sh`](https://github.com/SpeedySpec/speedyspec-platform/blob/main/scripts/build-php.sh) at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set. Rust 1.99.

## Features

- `embed` (on by default) links the embedded PHP. It is required to run PHP; without it, `Php::boot` returns an
  error. It is off only for docs.rs builds, whose ext-php-rs bindings cannot build it.

## Example

Boot PHP and evaluate an expression.

```rust
use std::rc::Rc;

use speedyspec_hooks::Value;
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, Php};

let php = Php::boot(BootOptions { hooks: Rc::new(MemoryHooks::new()), preload: &[], areas: vec![] })
    .expect("PHP boots");
assert_eq!(php.eval("example", "return 6 * 7;").unwrap(), Value::Int(42));
```

## Where it fits

Every crate that runs PHP builds on this one: `speedyspec-php-ext` for the extension's shared parts,
`speedyspec-container-php` and `speedyspec-wp-hooks-php` for their areas, and the runtime itself. The pinned PHP
version ships in the package as `php-version`, and a consumer's `build-php.sh` reads it from the package it
resolved.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
