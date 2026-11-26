# speedyspec-container

[![crates.io](https://img.shields.io/crates/v/speedyspec-container.svg)](https://crates.io/crates/speedyspec-container)
[![docs.rs](https://docs.rs/speedyspec-container/badge.svg)](https://docs.rs/speedyspec-container)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

A service container with the functionality of Laravel's: bindings, singletons, scoped instances, aliases, extenders, contextual bindings, tags and resolving callbacks, with worker and request lifetimes.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `Container`: bindings, singletons, scoped instances, aliases, extenders, contextual bindings (`when`), tags and resolving callbacks, with the functionality of Laravel's container.
- `bind_port`, `singleton_port` and `make_port`: a port (a trait object) bound by key and resolved as `Rc<dyn Port>`.
- `Lifetime::Worker` and `Lifetime::Request`: `Container::end_request` drops every request-lifetime entry and puts each key back on its worker entry.
- Foreign entries (`bind_foreign`, `is_foreign`): what a language bridge such as `speedyspec-container-php` registers to override a Rust binding.

## Install

```sh
cargo add speedyspec-container
```

## Requirements

Rust 1.99.

## Example

Bind a port to a named factory and resolve it.

```rust
use std::rc::Rc;

use speedyspec_container::{Container, ContainerError, Params};

trait Greeter {
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

let container = Container::new();
container.singleton_port::<dyn Greeter>("greeter", english);
let greeter = container.make_port::<dyn Greeter>("greeter").unwrap();
assert_eq!(greeter.greet("Ada"), "Hello, Ada");
```

## Where it fits

Every seam's contracts crate names its ports by key, and the runtime binds an implementation to each key here. PHP
overrides a binding through `speedyspec-container-php`, for one request. `speedyspec-cqrs` resolves its handlers
from this container. See the guide's container seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
