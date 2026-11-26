# speedyspec-cqrs

[![crates.io](https://img.shields.io/crates/v/speedyspec-cqrs.svg)](https://crates.io/crates/speedyspec-cqrs)
[![docs.rs](https://docs.rs/speedyspec-cqrs/badge.svg)](https://docs.rs/speedyspec-cqrs)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Command and query buses: handlers resolved from the service container by key, middleware, and direct dispatch.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `Command` and `Query`: a use case that changes state or reads it, with its handler's container key.
- `CommandBus::dispatch` and `QueryBus::ask`: resolve the handler from the container and run the bus's `Middleware` around it.
- `HandlerRef` and `QueryRef`: direct dispatch, resolved once and again only when the binding changes.
- `bind_command_handler` and `bind_query_handler`: bind a handler as a port.

## Install

```sh
cargo add speedyspec-cqrs
```

## Requirements

Rust 1.99.

## Example

A query, its handler (bound through a named factory), and the bus that asks it.

```rust
use std::rc::Rc;

use speedyspec_container::{Container, ContainerError, Params};
use speedyspec_cqrs::{Query, QueryBus, QueryHandler, bind_query_handler};

struct WordCount(String);

impl Query for WordCount {
    type Result = usize;
    const KEY: &'static str = "example.word_count";
}

struct CountWords;

impl QueryHandler<WordCount> for CountWords {
    fn handle(&self, query: WordCount) -> usize {
        query.0.split_whitespace().count()
    }
}

fn count_words(_: &Container, _: &Params) -> Result<Rc<dyn QueryHandler<WordCount>>, ContainerError> {
    Ok(Rc::new(CountWords))
}

let container = Rc::new(Container::new());
bind_query_handler::<WordCount>(&container, count_words);
let bus = QueryBus::new(container);
assert_eq!(bus.ask(WordCount("one two three".to_owned())).unwrap(), 3);
```

## Where it fits

Ports such as `speedyspec-wp-hooks` are commands and queries over this bus. A handler is a container binding, so PHP
overrides one by binding the same key (the server's `speedyspec-cqrs-php`).

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
