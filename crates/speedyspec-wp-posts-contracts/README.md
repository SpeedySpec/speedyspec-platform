# speedyspec-wp-posts-contracts

[![crates.io](https://img.shields.io/crates/v/speedyspec-wp-posts-contracts.svg)](https://crates.io/crates/speedyspec-wp-posts-contracts)
[![docs.rs](https://docs.rs/speedyspec-wp-posts-contracts/badge.svg)](https://docs.rs/speedyspec-wp-posts-contracts)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

WordPress's post contracts: the post entity and its values, post type and status names and values, the read and write surfaces, the errors and the ports.

Part of the [SpeedySpec platform](https://github.com/SpeedySpec/speedyspec-platform).
**Role:** contracts. **Seam:** [`query`](https://github.com/SpeedySpec/speedyspec-guide/tree/main/seams/query).

## What it gives you

- `Post`: the post entity, a plain object.
- Values: `PostId`, `PostParent` (a parent as PHP holds it, which may be negative while a post is written),
  `PostTypeName`, `PostStatusName`, `Slug`, `PostDateTime` and the post type and status objects.
- `Posts`: reading posts, one method per `post.php` function (`get_post()`, the field readers, pages and
  hierarchy, counts, the last post dates and the `posts` cache functions), with `PostRef`, `Filter`, `Timezone`,
  `LastPostField`, and `MimeFilter` and `PostTypeArg`, which keep WordPress's string-or-array arguments apart.
- `PostWrites`: writing posts, one method per `post.php` function (`wp_insert_post()`, `wp_update_post()`, delete,
  trash, untrash, publish, the status transition, sticky posts, pings, `set_post_type()` and auto-draft cleanup), with
  PHP-shaped arguments, `WriteResult`, `PostWriteError` (the `WP_Error`s they return) and `PostReturn`.
- `PostRepository`: one method per `wp_posts` statement `post.php` sends, reads and writes, each documented with
  WordPress 7.2-RC2's text (as `wpdb` builds it for `$wpdb->insert`, `update` and `delete`), and `PageRow`.
- Ports to code that is not ported yet: `Users`, `Taxonomy`, `Comments`, `Rewrite`, `Uploads`, `Theme`, `Revisions`
  and `Charset`. Options and cron go through their own contracts.
- `CurrentPost` (the global `$post`) and `PostQuery` (`WP_Query`, opaque until the query seam is ported).
- `PostError`, which carries a hook's failure or a cache error whole, `PostTypeError`, `PostWriteError`, and a
  `conformance` suite.
  `PostError` is not `Send` or `Sync`: a hook error's payload (a PHP exception) is held in an `Rc`, as in the
  metadata contracts' errors.

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

**Ports:** `BadSlugs`, `Charset`, `Clock`, `Comments`, `CurrentPost`, `PostQuery`, `PostReader`, `PostRepository`, `PostStatuses`, `PostTypeSupports`, `PostTypes`, `PostWrites`, `Posts`, `Revisions`, `Rewrite`, `RewriteContext`, `SlugTruncation`, `Taxonomy`, `Theme`, `TitleSanitizer`, `Translator`, `Uploads`, `Users`.

## Versioning

SemVer, 0.x: a minor version may break the API. MSRV is Rust 1.99. SpeedySpec's own version (ADR-028) is the PHP
runtime's major, not this crate's.

## License

MIT
