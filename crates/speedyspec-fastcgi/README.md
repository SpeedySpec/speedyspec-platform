# speedyspec-fastcgi

[![crates.io](https://img.shields.io/crates/v/speedyspec-fastcgi.svg)](https://crates.io/crates/speedyspec-fastcgi)
[![docs.rs](https://docs.rs/speedyspec-fastcgi/badge.svg)](https://docs.rs/speedyspec-fastcgi)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

FastCGI 1.0: records and name-value pairs, a blocking responder (the PHP worker's side) and an async client (the server's side), streaming in both directions with backpressure.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** port. **Seam:** none (the platform kernel every seam may use).

## What it gives you

- `Record`, `RecordType`, `BeginRequest` and `EndRequest`: FastCGI 1.0 records.
- `encode_pairs` and `decode_pairs`: name-value pairs.
- `Responder`: the blocking side a PHP worker runs; `Client`: the async side the server runs. Both stream with backpressure.

## Install

```sh
cargo add speedyspec-fastcgi
```

## Requirements

Rust 1.99.

## Example

Name-value pairs round-trip through their wire form.

```rust
use speedyspec_fastcgi::{decode_pairs, encode_pairs};

let pairs = [("SCRIPT_FILENAME", "/srv/index.php"), ("REQUEST_METHOD", "GET")];
let decoded = decode_pairs(&encode_pairs(&pairs)).unwrap();
assert_eq!(decoded[0], (b"SCRIPT_FILENAME".to_vec(), b"/srv/index.php".to_vec()));
assert_eq!(decoded.len(), 2);
```

## Where it fits

The server's process manager talks to PHP workers with `Client`, and `speedyspec-php-runtime` answers with
`Responder`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
