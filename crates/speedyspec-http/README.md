# speedyspec-http

[![crates.io](https://img.shields.io/crates/v/speedyspec-http.svg)](https://crates.io/crates/speedyspec-http)
[![docs.rs](https://docs.rs/speedyspec-http/badge.svg)](https://docs.rs/speedyspec-http)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Contracts for HTTP messages, factories, handlers and middleware, shaped after PSR-7, PSR-17, PSR-15 and PSR-18.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`route`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/route).

## What it gives you

- `Request`, `Response`, `ServerRequest`, `Uri`, `Headers` and `Body`: HTTP messages, shaped after PSR-7.
- Factories after PSR-17, `Handler` and `Middleware` after PSR-15, and `Client` after PSR-18.
- `Method`, `StatusCode` and `HttpError`.

## Install

```sh
cargo add speedyspec-http
```

## Requirements

Rust 1.99.

## Example

Methods, status codes and headers are plain values. Method tokens are case-sensitive; header names are not.

```rust
use speedyspec_http::{Headers, Method, StatusCode};

assert_eq!(Method::parse("POST"), Method::Post);
assert_eq!(Method::parse("post").as_str(), "post");
assert_eq!(StatusCode::new(404).unwrap().reason(), "Not Found");

let headers = Headers::new().with("Content-Type", vec!["text/html".to_owned()]).unwrap();
assert_eq!(headers.get("content-type"), Some("text/html".to_owned()));
```

## Where it fits

The server's HTTP message implementation and its handlers implement these contracts. The REST and XML-RPC
transports use them for their requests and responses.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
