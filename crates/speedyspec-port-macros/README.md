# speedyspec-port-macros

[![crates.io](https://img.shields.io/crates/v/speedyspec-port-macros.svg)](https://crates.io/crates/speedyspec-port-macros)
[![docs.rs](https://docs.rs/speedyspec-port-macros/badge.svg)](https://docs.rs/speedyspec-port-macros)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

#[port]: a port trait's key and its re-resolving container proxy, with no PHP; edge ports also carry their signature to php_twin!.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `#[port(key = "...")]` on a port trait: the trait unchanged, `<TRAIT>_KEY` (the key it is bound under), and `Proxy<Trait>`, which forwards every call to whatever the container binds under the key now.
- `#[port(key = "...", edge)]`: also carries the trait's signature to `php_twin!` in `speedyspec-php-port-macros`, so the seam's PHP crate can generate the twin. Nothing here needs PHP.

## Install

```sh
cargo add speedyspec-port-macros
```

## Requirements

Rust 1.99.

## Example

A proxy follows the binding: rebind the key and every holder of the proxy reaches the new implementation. The
generated code names `::speedyspec_container`, so a crate that uses `#[port]` depends on it.

```rust
use std::rc::Rc;

use speedyspec_container::{Container, ContainerError, Params};
use speedyspec_port_macros::port;

#[port(key = "Example\\Greeter")]
pub trait Greeter {
    fn greet(&self, name: &str) -> String;
}

struct English;

impl Greeter for English {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {name}")
    }
}

fn english(_: &Container, _: &Params) -> Result<Rc<dyn Greeter>, ContainerError> {
    Ok(Rc::new(English))
}

fn main() {
    let container = Rc::new(Container::new());
    container.singleton_port::<dyn Greeter>(GREETER_KEY, english);
    let greeter = ProxyGreeter::new(&container);
    assert_eq!(GREETER_KEY, "Example\\Greeter");
    assert_eq!(greeter.greet("Ada"), "Hello, Ada");
}
```

## Where it fits

Every contracts crate declares its ports with `#[port]`, and providers hand out the proxies, so a rebind reaches
every caller. For an edge port, the seam's PHP crate passes the carried signature to `php_twin!`
(`speedyspec-php-port-macros`). See the guide's container seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
