# speedyspec-data

[![crates.io](https://img.shields.io/crates/v/speedyspec-data.svg)](https://crates.io/crates/speedyspec-data)
[![docs.rs](https://docs.rs/speedyspec-data/badge.svg)](https://docs.rs/speedyspec-data)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Data-access contracts for speedyspec: values, statements, SQL/NoSQL/Direct stores, engines and the unit of work. No implementations.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** `data`.

## What it gives you

- `DbValue`, `Row` and `Rows`: values and rows as the database returns them, with mysqli's text forms.
- `Statement`, `StatementKind` and `TxControl`: a parsed statement (`sqlparser`'s AST, re-exported as `ast`).
- `Engine`, `Dialect` and the SQL, NoSQL and direct stores: the ports a data engine implements.
- The unit of work: changes collected and committed together.

## Install

```sh
cargo add speedyspec-data
```

## Requirements

Rust 1.99.

## Example

Values and rows read the way mysqli hands them to PHP.

```rust
use std::rc::Rc;

use speedyspec_data::{DbValue, Row};

assert_eq!(DbValue::Int(-5).to_mysql_text(), Some(b"-5".to_vec()));
assert_eq!(DbValue::Null.to_mysql_text(), None);

let columns: Rc<[String]> = Rc::from(vec!["id".to_owned(), "title".to_owned()]);
let row = Row::new(columns, vec![DbValue::Int(1), DbValue::Text("Hello".to_owned())]);
assert_eq!(row.get("title"), Some(&DbValue::Text("Hello".to_owned())));
```

## Where it fits

The data engines (MySQL, SQLite and the others) implement these ports in speedyspec-server.
`speedyspec-wp-db-contracts` builds `wpdb` on top of them.

**Ports:** `Dialect`, `DirectStore`, `Engine`, `EngineFactory`, `EngineLookup`, `Identified`, `Introspector`, `NoSqlStore`, `PendingChange`, `Repository`, `SqlStore`, `UnitOfWork`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
