//! Reading posts as one contract: a method per post-reading function of WordPress's `post.php`.
//!
//! Facade: [`Posts`] is what runtime features read posts through. Command/CQRS: each method is one handler, bound in
//! the container. Observer: implementations run WordPress's hooks through the hook API. Identity Map: the `posts`
//! cache group, which stays WordPress's `stdClass` rows.

use speedyspec_php_serialize::PhpValue;

use crate::{MimeType, Post, PostError, PostId, PostStatusName, PostTypeName};

/// The post a read is about: an id, or the current post (the global `$post`).
///
/// `get_post()` falls back to the global `$post` when its argument is empty (no argument, `null` or `0`); a binding
/// maps those to [`PostRef::Current`], and any other id to [`PostRef::Id`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PostRef {
    /// A post id.
    Id(PostId),
    /// The global `$post`, read through [`crate::CurrentPost`].
    Current,
}

macro_rules! words {
    ($(#[$doc:meta])* $ty:ident { $($(#[$vdoc:meta])* $variant:ident => $text:literal,)+ }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $ty {
            $($(#[$vdoc])* $variant,)+
        }

        impl $ty {
            /// The word WordPress uses.
            pub fn as_str(&self) -> &'static str {
                match self {
                    $($ty::$variant => $text,)+
                }
            }
        }
    };
}

/// `$mime_type` as `wp_count_attachments()` and `wp_post_mime_type_where()` receive it: a string or an array, which
/// WordPress treats differently, so the distinction is kept.
///
/// - Parsing: a [`MimeFilter::One`] string is split on `,`, each item trimmed, and empty items dropped. A
///   [`MimeFilter::Many`] array is used as given: an empty item stays (and matches everything), and an item holding a
///   comma is one pattern.
/// - Cache key: `wp_count_attachments()` keys the `counts` group with `implode( '-', (array) $mime_type )`, so
///   `One("image,video")` and `Many(["image", "video"])` are different keys, as they are in PHP.
/// - Filters: `pre_wp_count_attachments` and `wp_count_attachments` receive the original value, which the binding
///   rebuilds from this.
///
/// No filter at all (`$mime_type` empty, the default) is `None` where a method takes `Option<&MimeFilter>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MimeFilter {
    /// A string, comma-separated.
    One(String),
    /// An array, each item one pattern.
    Many(Vec<String>),
}

/// `$post_type` as `get_page_by_path()` receives it: a string or an array, which WordPress treats differently, so the
/// distinction is kept.
///
/// - Statement: [`PostTypeArg::One`] searches `[ $post_type, 'attachment' ]`; [`PostTypeArg::Many`] searches exactly
///   the list. [`PostTypeArg::types`] gives the list the statement matches.
/// - Matching: with `One`, a full-path attachment match is kept as a fallback and the loop keeps looking, so an
///   attachment cannot hide a page of the requested type with the same path; with `Many` the loop stops at the first
///   full-path match (post.php 6570-6582).
/// - Cache key: `md5( $page_path . serialize( $post_type ) )`, so `One("page")` and `Many(["page"])` are different
///   keys, as they are in PHP.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PostTypeArg {
    /// A string: one post type, which also searches attachments.
    One(PostTypeName),
    /// An array: exactly these post types.
    Many(Vec<PostTypeName>),
}

impl PostTypeArg {
    /// The post types the statement matches, in the order `get_page_by_path()` lists them.
    pub fn types(&self) -> Vec<PostTypeName> {
        match self {
            PostTypeArg::One(name) => vec![name.clone(), PostTypeName::new("attachment")],
            PostTypeArg::Many(names) => names.clone(),
        }
    }
}

words!(
    /// A sanitize context: `get_post()`'s `$filter` and `sanitize_post_field()`'s `$context`.
    ///
    /// `sanitize_post_field()` treats any word it does not know as [`Filter::Display`]; a binding that receives one
    /// decides how to map it ([`Filter::parse`] gives `None`).
    Filter {
        /// `raw`: the stored value, unfiltered.
        Raw => "raw",
        /// `edit`: for a form field (`edit_{$field}`, `format_to_edit()`).
        Edit => "edit",
        /// `db`: for the database (`pre_{$field}`).
        Db => "db",
        /// `display`: for output (`{$field}`).
        Display => "display",
        /// `attribute`: `display`, then `esc_attr()`.
        Attribute => "attribute",
        /// `js`: `display`, then `esc_js()`.
        Js => "js",
    }
);

impl Filter {
    /// The context `word` names, compared exactly as WordPress compares it; `None` for any other word.
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "raw" => Filter::Raw,
            "edit" => Filter::Edit,
            "db" => Filter::Db,
            "display" => Filter::Display,
            "attribute" => Filter::Attribute,
            "js" => Filter::Js,
            _ => return None,
        })
    }
}

words!(
    /// `_get_last_post_time()`'s `$field`: which date column pair it reads.
    LastPostField {
        /// `date`: `post_date` and `post_date_gmt`.
        Date => "date",
        /// `modified`: `post_modified` and `post_modified_gmt`.
        Modified => "modified",
    }
);

impl LastPostField {
    /// The field `word` names; `None` for any other word, where `_get_last_post_time()` returns `false`.
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "date" => Some(LastPostField::Date),
            "modified" => Some(LastPostField::Modified),
            _ => None,
        }
    }
}

words!(
    /// `get_lastpostdate()`'s `$timezone`: which clock the answer is in.
    Timezone {
        /// `blog`: the local column (`post_date`).
        Blog => "blog",
        /// `gmt`: the GMT column (`post_date_gmt`).
        Gmt => "gmt",
        /// `server`: the GMT column plus PHP's `gmdate( 'Z' )` seconds, which is always `0`.
        Server => "server",
    }
);

impl Timezone {
    /// The timezone `word` names after `strtolower()`, as `_get_last_post_time()` lowercases it; `None` for any other
    /// word, where WordPress sends no statement and answers `false`.
    pub fn parse(word: &str) -> Option<Self> {
        match word.to_ascii_lowercase().as_str() {
            "blog" => Some(Timezone::Blog),
            "gmt" => Some(Timezone::Gmt),
            "server" => Some(Timezone::Server),
            _ => None,
        }
    }
}

/// Reading posts (`get_post()` and the functions beside it), WordPress 7.2-RC2.
///
/// Every method stands for the WordPress function its doc names and runs that function's filters, actions and cache
/// reads. `Err` carries a hook callback's failure, a cache error, or a database failure `$wpdb` would not hide.
pub trait Posts {
    /// `get_post( $post, OBJECT, $filter )`: the raw post (as `raw` reads it, from the `posts` cache group or the
    /// table), `None` where WordPress returns `null`.
    ///
    /// The caller applies `sanitize_post( $post, $filter )` for any other context: the `{$field}` filters
    /// `display`, `attribute` and `js` run may return any PHP value, which [`Post`]'s typed columns cannot hold, so
    /// that is the PHP adapter's job (`speedyspec-wp-posts-php`), as is shaping `ARRAY_A` and `ARRAY_N`. `filter` is
    /// passed so an implementation can short-circuit `raw`, as `WP_Post::filter()` does.
    fn get(&self, id: PostRef, filter: Filter) -> Result<Option<Post>, PostError>;

    /// `get_post_field( $field, $post, $context )`: the field sanitized for `filter`; `''` when there is no post or
    /// no such field.
    fn field(&self, name: &str, id: PostRef, filter: Filter) -> Result<PhpValue, PostError>;

    /// `get_post_status()`: the status (an attachment's parent's, for `inherit`), `None` where WordPress returns
    /// `false`.
    fn status(&self, id: PostRef) -> Result<Option<PostStatusName>, PostError>;

    /// `get_post_type()`: `None` where WordPress returns `false`.
    fn post_type(&self, id: PostRef) -> Result<Option<PostTypeName>, PostError>;

    /// `get_post_mime_type()`: `None` where WordPress returns `false`.
    fn mime_type(&self, id: PostRef) -> Result<Option<MimeType>, PostError>;

    /// `get_post_ancestors()`: the parent first, then its parent, stopping at a loop.
    fn ancestors(&self, id: PostRef) -> Result<Vec<PostId>, PostError>;

    /// `wp_get_post_parent_id()`: `None` where WordPress returns `false` (no post, or no parent).
    fn parent_id(&self, id: PostRef) -> Result<Option<PostId>, PostError>;

    /// `get_page_by_path( $page_path, OBJECT, $post_type )`: the raw post, `None` where WordPress returns `null`.
    /// [`PostTypeArg`] keeps the string-or-array distinction the matching loop and the cache key depend on.
    fn page_by_path(&self, path: &str, post_type: &PostTypeArg) -> Result<Option<Post>, PostError>;

    /// `get_page_uri()`: the path of slugs from the top, as bytes (slugs are bytes), `None` where WordPress returns
    /// `false`.
    fn page_uri(&self, id: PostRef) -> Result<Option<Vec<u8>>, PostError>;

    /// `get_all_page_ids()`, through the `posts` cache group's `all_page_ids` key.
    fn all_page_ids(&self) -> Result<Vec<PostId>, PostError>;

    /// `wp_count_posts( $type, $perm )`: the `stdClass` WordPress returns, after the `wp_count_posts` filter.
    ///
    /// Unfiltered, it holds every status in `get_post_stati()` order, then any status found in the table but not
    /// registered; database counts are numeric strings and the defaults ints. The filter may return anything, so the
    /// value is PHP's, not a typed count list (the repository's [`crate::PostRepository::count_by_status`] is typed).
    fn count_posts(&self, post_type: &PostTypeName, perm: &str) -> Result<PhpValue, PostError>;

    /// `wp_count_attachments( $mime_type )`: the `stdClass` WordPress returns (each mime type, keyed by its bytes,
    /// then `trash`), after the `wp_count_attachments` filter, which may return anything. `None` is no filter.
    fn count_attachments(&self, mime_type: Option<&MimeFilter>) -> Result<PhpValue, PostError>;

    /// `get_lastpostdate( $timezone, $post_type )`. `timezone` is the raw value: the filters see it un-lowercased,
    /// and an unknown one still runs `get_lastpostdate` with `false` ([`Timezone::parse`] picks the statement). The
    /// value the filter returns, `false` included.
    fn last_post_date(
        &self,
        timezone: &str,
        post_type: &PostTypeName,
    ) -> Result<PhpValue, PostError>;

    /// `get_lastpostmodified( $timezone, $post_type )`, as [`Posts::last_post_date`], after the
    /// `get_lastpostmodified` filter.
    fn last_post_modified(
        &self,
        timezone: &str,
        post_type: &PostTypeName,
    ) -> Result<PhpValue, PostError>;

    /// `get_available_post_mime_types( $type )`, after the `pre_get_available_post_mime_types` filter.
    fn available_mime_types(&self, post_type: &PostTypeName) -> Result<Vec<MimeType>, PostError>;

    /// `update_post_cache( $posts )`: writes each post to the `posts` group.
    fn update_cache(&self, posts: &[Post]) -> Result<(), PostError>;

    /// `clean_post_cache( $post )`: its keys, its children's, and the `clean_post_cache` and
    /// `clean_page_cache` actions.
    fn clean_cache(&self, id: PostId) -> Result<(), PostError>;

    /// `_prime_post_caches( $ids, $update_term_cache, $update_meta_cache )`.
    fn prime_caches(&self, ids: &[PostId], terms: bool, meta: bool) -> Result<(), PostError>;

    /// `_prime_post_parent_id_caches( $ids )`.
    fn prime_parent_caches(&self, ids: &[PostId]) -> Result<(), PostError>;
}
