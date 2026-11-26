# speedyspec-platform

The shared crates of SpeedySpec: the contracts every seam is written against, the small ports that depend on
nothing else, and the embedded PHP the runtime is built on. They were split out of
[speedyspec-server](https://github.com/SpeedySpec/speedyspec-server), with their history, so that the server, the CLI
and the guide all build on the same published versions from crates.io.

Every crate is 0.x and versioned on its own. Implementations, adapters, data engines, the seams' PHP glue and every
`speedyspec-server*` crate stay in the server.

## The crates

**Platform: the runtime's own pieces**

| Crate | What it is |
|---|---|
| `speedyspec-hooks`, `speedyspec-hooks-memory` | The hook contract (the `Hooks` trait and `Value`), and its in-memory backend. |
| `speedyspec-container` | The service container: bindings, singletons, scoped instances, tags, worker and request lifetimes. |
| `speedyspec-cqrs` | Command and query buses over the container. |
| `speedyspec-php` | The embedded, statically linked PHP interpreter. |
| `speedyspec-php-ext`, `speedyspec-php-port-macros` | The `speedyspec_wordpress` extension's shared parts, and `#[php_port]`, which gives a Rust port its PHP twin. |
| `speedyspec-container-php` | The container from PHP: PHP bindings as request-long overrides of Rust ports. |
| `speedyspec-compat` | Recorded compatibility: scenarios run against WordPress's original code, replayed against the Rust ports. |
| `speedyspec-fastcgi` | FastCGI 1.0, the PHP worker's side and the server's. |
| `speedyspec-runtime-contract` | What the server and the PHP runtime agree on: the worker spec, the request report, the contract version. |
| `speedyspec-php-serialize` | PHP's `serialize` format. |

**Contracts: ports and plain values, no implementations**

| Seam | Crates |
|---|---|
| Hook API | `speedyspec-wp-hooks-contracts` |
| Data access | `speedyspec-data`, `speedyspec-cache`, `speedyspec-wp-db-contracts`, `speedyspec-wp-object-cache-contracts` |
| HTTP and routing | `speedyspec-http`, `speedyspec-routing`, `speedyspec-native-stage` |
| Operations and transports | `speedyspec-operation-contracts`, `speedyspec-transport-contracts`, `speedyspec-rest-contracts`, `speedyspec-xmlrpc-contracts`, `speedyspec-abilities-contracts` |
| WordPress APIs | `speedyspec-wp-cron-contracts`, `speedyspec-wp-crypto-contracts`, `speedyspec-wp-i18n-contracts`, `speedyspec-wp-kses-contracts`, `speedyspec-wp-media-contracts`, `speedyspec-wp-meta-contracts`, `speedyspec-wp-password-contracts`, `speedyspec-wp-posts-contracts`, `speedyspec-wp-shortcode-contracts`, `speedyspec-wp-text-contracts`, `speedyspec-wp-text-diff-contracts`, `speedyspec-wp-theme-json-contracts` |

**Ports: WordPress functions in Rust that need nothing outside the platform**

| Seam | Crates |
|---|---|
| Hook API | `speedyspec-wp-hooks` (commands and queries over a hook store), `speedyspec-wp-hooks-memory`, `speedyspec-wp-hooks-php` (the hook functions and `WP_Hook` in PHP) |
| Text and HTML | `speedyspec-wp-utf8`, `speedyspec-wp-token-map`, `speedyspec-wp-html-split`, `speedyspec-wp-html-syntax`, `speedyspec-wp-kses-lexical`, `speedyspec-wp-text-diff` |

## The dependency rule

A platform crate depends only on crates.io and on other crates under `crates/`, never on a server, CLI or guide
crate, not even as a dev-dependency, and the root `Cargo.toml` has no `[patch]`. CI checks it:

```bash
cargo metadata --format-version 1 --locked > metadata.json   # with dependencies, so patches show
scripts/platform-closure.sh check metadata.json "$PWD"
scripts/platform-closure.sh closure metadata.json speedyspec-wp-hooks   # what must publish with it
```

Inside the platform, a crate names another as `{ path = "../<crate>", version = "<version>" }`, so it builds here and
publishes against crates.io.

## Building PHP

Five crates link the static PHP: `speedyspec-php`, `speedyspec-php-ext`, `speedyspec-container-php`,
`speedyspec-compat` and `speedyspec-wp-hooks-php`. The tests of `speedyspec-php-port-macros` link it too, through
`speedyspec-php-ext`. Build it once; `.cargo/config.toml` points cargo at it.

```bash
scripts/build-php.sh                         # macOS or Linux, into .php-build/
scripts/linux-php.sh scripts/build-php.sh    # Linux musl in a container, into .php-build-linux*/
cargo test -p speedyspec-php --test extensions
```

Everything else builds and tests without PHP:

```bash
cargo test --workspace --exclude speedyspec-php --exclude speedyspec-php-ext --exclude speedyspec-container-php \
  --exclude speedyspec-compat --exclude speedyspec-wp-hooks-php --exclude speedyspec-php-port-macros
```

## Working on the platform from a consumer

The server, the CLI and the guide take these crates from crates.io. To try an unreleased platform change in one of
them, run `scripts/platform-local.sh <platform-checkout>` there. It writes a git-ignored `platform.local.toml`, with one
`[patch.crates-io]` line per crate in this repository's `scripts/publish-crates.txt`, and you build with
`cargo --config platform.local.toml …`. A `[patch]` for a `speedyspec-*` crate is never committed; the consumers'
CI fails if one is.

## Releases

Every crate has its own version, `CHANGELOG.md` and GitHub release, tagged `speedyspec-<crate>-v<version>`.
release-please (`release-please-config.json`, with the `cargo-workspace` plugin) keeps one release pull request open
on `main`, built from Conventional Commits: `feat` bumps a crate's minor version while it is 0.x, `fix` and the rest its
patch, and a breaking change (`feat!:` or a `BREAKING CHANGE:` footer) its minor. When a crate is released, its
dependents' requirements on it are bumped too. Merging the release pull request tags the releases, and
`.github/workflows/release.yml` publishes every crate in `scripts/publish-crates.txt` whose new version crates.io
lacks, in that order (`scripts/publish-pending.sh`). Changelogs list features, bug fixes, performance, refactoring,
documentation, build and CI commits; `chore` and `test` commits are left out.

**PHP-observable changes.** The runtime promises SpeedySpec 1 users WordPress's PHP-observable behaviour, so the
server must not take a platform change that alters what PHP code can see (a return value, a hook's arguments or
order, an error) without a departures-register entry, or before a SpeedySpec major. Flag such a commit twice:

- a `PHP-Observable:` footer saying what PHP sees differently, so `git log --grep '^PHP-Observable:'` finds it; and
- `PHP-observable:` at the start of the subject's description, for example
  `fix(wp-hooks): PHP-observable: remove_filter returns false for an unknown priority`, because release-please
  copies only the subject (and `BREAKING CHANGE` notes) into the changelog. That is how the server session sees it in
  the bump pull request.

## License

MIT. See [LICENSE](LICENSE).
