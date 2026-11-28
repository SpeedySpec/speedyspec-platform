# speedyspec-compat

[![crates.io](https://img.shields.io/crates/v/speedyspec-compat.svg)](https://crates.io/crates/speedyspec-compat)
[![docs.rs](https://docs.rs/speedyspec-compat/badge.svg)](https://docs.rs/speedyspec-compat)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Recorded compatibility: run a catalogue of scenarios against WordPress's original code, record the results, and replay them against the Rust ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** [`container`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/container).

## What it gives you

- `Catalogue`, `Scenario` and `Fixture`: an area's scenarios and what WordPress's own code returned for them.
- `ScenarioRunner`: one side of a replay (WordPress, or a port); `PhpRunner` runs scenarios in the embedded PHP.
- `replay`: runs an area's scenarios through a runner and returns each `Mismatch` against the fixture.
- Recording writes: a scenario or step may freeze PHP's clock (`clock`), log a user in (`user`), snapshot tables'
  rows ordered by primary key (`snapshot`) and report the normalised cron schedule (`cron`). The clock decorates
  `time()`, `microtime()`, `date()`/`gmdate()` without a timestamp and `DateTime` "now" in the engine's function
  table, so `current_time()` and `wp_date()` follow it.
- `try_compat_dir`: where the catalogue lives. It reads `SPEEDYSPEC_COMPAT_DIR` and has no fallback.

## Install

```sh
cargo add speedyspec-compat
```

## Requirements

A static PHP built by [`scripts/build-php.sh`](https://github.com/SpeedySpec/speedyspec-platform/blob/main/scripts/build-php.sh) at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set. Rust 1.99.

## Features

- `embed` (on by default) links the embedded PHP. It is required to run PHP; without it, `Php::boot` returns an
  error. It is off only for docs.rs builds, whose ext-php-rs bindings cannot build it.

## Example

A runner for a port is a `ScenarioRunner`. The catalogue is per-repository data: set `SPEEDYSPEC_COMPAT_DIR` in
`.cargo/config.toml` (`{ value = "compat", relative = true }`).

```rust
use speedyspec_compat::{Scenario, ScenarioRunner, StepResult, fixture_name};

struct NotPorted;

impl ScenarioRunner for NotPorted {
    fn run(&mut self, scenario: &Scenario) -> Result<Vec<StepResult>, String> {
        Err(format!("{} is not ported yet", scenario.name))
    }
}

let scenario: Scenario = serde_json::from_str(r#"{"name": "empty", "steps": []}"#).unwrap();
assert_eq!(NotPorted.run(&scenario), Err("empty is not ported yet".to_owned()));
assert_eq!(fixture_name("6.8", false), "wordpress-6.8.json");
```

## Where it fits

Each seam's PHP crate replays its area's catalogue against its port: the hooks area here, every other area in
speedyspec-server. The recorder runs the same scenarios against WordPress's original code from the pinned release.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
