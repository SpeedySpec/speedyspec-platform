# speedyspec-compat

[![crates.io](https://img.shields.io/crates/v/speedyspec-compat.svg)](https://crates.io/crates/speedyspec-compat)
[![docs.rs](https://docs.rs/speedyspec-compat/badge.svg)](https://docs.rs/speedyspec-compat)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Recorded compatibility: run a catalogue of scenarios against WordPress's original code, record the results, and replay them against the Rust ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** platform. **Seam:** none (the platform kernel every seam may use).

## What it gives you

- `Catalogue`, `Scenario` and `Fixture`: an area's scenarios and what WordPress's own code returned for them.
- `ScenarioRunner`: one side of a replay (WordPress, or a port); `PhpRunner` runs scenarios in the embedded PHP.
- `replay`: runs an area's scenarios through a runner and returns each `Mismatch` against the fixture.
- `try_compat_dir`: where the catalogue lives. It reads `SPEEDYSPEC_COMPAT_DIR` and has no fallback.

## Install

```sh
cargo add speedyspec-compat
```

## Requirements

A static PHP built by `scripts/build-php.sh` at `speedyspec-php`'s pinned `php-version`, with `SPEEDYSPEC_PHP_PREFIX`, `PHP`, `PHP_CONFIG` and `SODIUM_LIB_DIR` set; see [`speedyspec-php`](https://docs.rs/speedyspec-php). Rust 1.99.

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
