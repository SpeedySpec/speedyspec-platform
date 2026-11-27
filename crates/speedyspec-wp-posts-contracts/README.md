# speedyspec-wp-posts-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-posts-contracts.svg)](https://crates.io/crates/speedyspec-wp-posts-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-posts-contracts/badge.svg)](https://docs.rs/speedyspec-wp-posts-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's post contracts: the post entity and its values, post type and status names and values, the read surface, the errors and the ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`query`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/query).

## What it gives you

- `Post`: the post entity, a plain object.
- Values: `PostId`, `PostTypeName`, `PostStatusName`, `Slug`, `PostDateTime` and the post type and status objects.
- `Posts`: reading posts, one method per `post.php` function (`get_post()`, the field readers, pages and
  hierarchy, counts, the last post dates and the `posts` cache functions), with `PostRef`, `Filter`, `Timezone` and
  `LastPostField`.
- `PostRepository`: one method per `wp_posts` statement `post.php` sends, each documented with WordPress 7.2-RC2's
  text, and `PageRow`.
- `CurrentPost` (the global `$post`) and `PostQuery` (`WP_Query`, opaque until the query seam is ported).
- `PostError`, which carries a hook's failure or a cache error whole, `PostTypeError`, and a `conformance` suite.

## Install

```sh
cargo add speedyspec-wp-posts-contracts
```

## Requirements

Rust 1.99.

## Example

Post values are typed, so a post type name never stands in for a status.

```rust
use speedyspec_wp_posts_contracts::{PostId, PostStatusName, PostTypeName};

let id = PostId(42);
let post_type = PostTypeName::new("page");
let status = PostStatusName::new("draft");
assert_eq!((id.0, post_type.as_str(), status.as_str()), (42, "page", "draft"));
```

## Where it fits

Implementations live in speedyspec-server and are bound in the service container (`speedyspec-container`). Bind
your own by implementing a port and binding it in their place; a PHP implementation replaces it through the
seam's PHP twin.

See the guide's query seam.

**Ports:** `BadSlugs`, `Clock`, `CurrentPost`, `PostQuery`, `PostReader`, `PostRepository`, `Posts`, `PostStatuses`, `PostTypeSupports`, `PostTypes`, `RewriteContext`, `SlugTruncation`, `TitleSanitizer`, `Translator`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
