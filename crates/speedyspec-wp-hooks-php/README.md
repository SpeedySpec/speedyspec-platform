# speedyspec-wp-hooks-php

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-hooks-php.svg)](https://crates.io/crates/speedyspec-wp-hooks-php)
[![docs.rs](https://docs.rs/speedyspec-wp-hooks-php/badge.svg)](https://docs.rs/speedyspec-wp-hooks-php)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's hook functions, WP_Hook and the hook globals in PHP, native over the Rust Hook API.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** php. **Seam:** [`hooks`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/hooks).

## What it gives you

- `WpHooksArea`: WordPress's hook functions, `WP_Hook` and the hook globals in PHP, declared at the start of every request with their exact signatures.
- Proxies for `$wp_filter`, `$wp_actions`, `$wp_filters` and `$wp_current_filter` over the Rust store.
- `HOOK_STORE`: the container key of the store, which a PHP implementation may rebind.

## Install

```sh
cargo add speedyspec-wp-hooks-php
```

## Requirements

A static PHP built by [`scripts/build-php.sh`](https://github.com/SpeedySpec/speedyspec-platform/blob/main/scripts/build-php.sh) at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set. Rust 1.99.

## Features

- `embed` (on by default) links the embedded PHP. It is required to run PHP; without it, `Php::boot` returns an
  error. It is off only for docs.rs builds, whose ext-php-rs bindings cannot build it.

## Example

Boot PHP with the hooks area, then add and apply a filter from PHP with a named function.

```rust
use std::rc::Rc;

use speedyspec_container::Container;
use speedyspec_container_php::ContainerArea;
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, NativeArea, Php};
use speedyspec_php_ext::{Value, twins_area};
use speedyspec_wp_hooks_memory::MemoryHookStore;
use speedyspec_wp_hooks_php::WpHooksArea;

let container = Rc::new(Container::new());
let hooks = WpHooksArea::new(container.clone(), Rc::new(MemoryHookStore::new()));
let areas: Vec<Rc<dyn NativeArea>> =
    vec![twins_area(), Rc::new(ContainerArea::new("example", container)), Rc::new(hooks)];
let php = Php::boot(BootOptions { hooks: Rc::new(MemoryHooks::new()), preload: &[], areas })
    .expect("PHP boots");
let title = php
    .eval(
        "example",
        "function example_shout($title) { return strtoupper($title); }
         add_filter('the_title', 'example_shout');
         return apply_filters('the_title', 'hello');",
    )
    .unwrap();
assert_eq!(title, Value::Str("HELLO".to_owned()));
```

## Where it fits

The runtime adds `WpHooksArea` to the `speedyspec_wordpress` extension, so WordPress and plugins call the Rust
Hook API (`speedyspec-wp-hooks`). A Rust caller whose hook has only Rust callbacks never enters PHP. The hooks
compat area and WordPress's own hook PHPUnit tests run against it in this repository. See the guide's hooks seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
