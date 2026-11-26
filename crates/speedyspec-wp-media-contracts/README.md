# speedyspec-wp-media-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-media-contracts.svg)](https://crates.io/crates/speedyspec-wp-media-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-media-contracts/badge.svg)](https://docs.rs/speedyspec-wp-media-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The media reader port and the library-neutral description of a media file (MediaInfo).

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`media`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/media).

## What it gives you

- `MediaReader` (bound under `MEDIA_READER`) and `MediaSource`: reading a media file's description.
- `MediaInfo` and `Tree`: the library-neutral description of a media file.
- `ProbeLimits` and `Limit`: what a read may cost; `ReadError`.
- Ports for content sniffing, tag sanitizing and time parsing.

## Install

```sh
cargo add speedyspec-wp-media-contracts
```

## Requirements

Rust 1.99.

## Example

The default limits bound every read.

```rust
use std::time::Duration;

use speedyspec_wp_media_contracts::ProbeLimits;

let limits = ProbeLimits::default();
assert_eq!(limits.max_bytes, 64 << 20);
assert_eq!(limits.deadline, Duration::from_secs(5));
```

## Where it fits

The FFmpeg-backed reader in speedyspec-server implements `MediaReader`. See the guide's media seam.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
