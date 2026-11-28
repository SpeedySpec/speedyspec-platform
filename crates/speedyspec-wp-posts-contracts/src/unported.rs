//! The ports to code that is not ported yet (Adapter): users, taxonomy, comments, rewrite, uploads, the theme,
//! revisions and the database's column charsets. The PHP adapters call WordPress's own functions; Rust adapters
//! replace them as each seam is ported, and PHP may replace any of them.
//!
//! Each method stands for the WordPress function its doc names, that function's own hooks, cache writes and statements
//! included, so query counts and hook order match. A callback's failure comes back as [`PostError::Hook`], and the use
//! case stops there, as WordPress would. Options go through `speedyspec-wp-options-contracts` and cron through
//! `speedyspec-wp-cron-contracts`, so they have no port here.

use speedyspec_php_serialize::PhpValue;

use crate::{Post, PostError, PostId, PostTypeName};

/// The users functions the post functions call (user.php, capabilities.php, pluggable.php).
pub trait Users {
    /// `is_user_logged_in()` (pluggable).
    fn is_user_logged_in(&self) -> Result<bool, PostError>;
    /// `get_current_user_id()`: 0 when nobody is logged in. `Err` carries a `determine_current_user` callback's
    /// failure.
    fn get_current_user_id(&self) -> Result<u64, PostError>;
    /// `current_user_can( $capability, ...$args )`, with the capability as PHP holds it (a post type's or taxonomy's
    /// `cap` property may hold anything a plugin put there) and the arguments after it: none for
    /// `current_user_can( $post_type_object->cap->publish_posts )`, the post id for
    /// `current_user_can( 'publish_post', $post_id )`.
    fn current_user_can(&self, capability: &PhpValue, args: &[PhpValue])
    -> Result<bool, PostError>;
}

/// The taxonomy functions the post functions call (taxonomy.php, post.php's term wrappers, class-wp-post.php).
///
/// Taxonomy names are UTF-8: PHP keys them by bytes, so the adapter decides how a non-UTF-8 name maps (none is
/// registered when every name went through `sanitize_key()`).
pub trait Taxonomy {
    /// `update_object_term_cache( $object_ids, $object_type )`: `object_types` are the values of the array
    /// `_prime_post_caches()` builds with `array_unique( array_map( 'get_post_type', $ids ) )`, so `false` for an id
    /// with no post.
    fn update_object_term_cache(
        &self,
        ids: &[PostId],
        object_types: &[PhpValue],
    ) -> Result<(), PostError>;
    /// `clean_object_term_cache( $object_ids, $object_type )`.
    fn clean_object_term_cache(&self, ids: &[PostId], object_type: &str) -> Result<(), PostError>;
    /// `WP_Post::__get()`'s `post_category` and `tags_input` branch for `post` (a `WP_Post`): when
    /// `is_object_in_taxonomy( $post->post_type, $taxonomy )`, `wp_list_pluck( get_the_terms( $post, $taxonomy ),
    /// $field )`; an empty list or a `WP_Error` gives `array()`. `post_category` is `( 'category', 'term_id' )` and
    /// `tags_input` `( 'post_tag', 'name' )`.
    fn post_term_fields(
        &self,
        post: &PhpValue,
        taxonomy: &str,
        field: &str,
    ) -> Result<PhpValue, PostError>;
    /// `wp_set_post_terms( $post_id, $terms, $taxonomy, $append )`: what it returns (the term taxonomy ids, `false`
    /// or a `WP_Error`), as PHP holds it. `wp_insert_post()` reaches it through `wp_set_post_categories()`,
    /// `wp_set_post_tags()` and `tax_input`, and `wp_publish_post()` for a default term.
    fn set_post_terms(
        &self,
        post: PostId,
        terms: &PhpValue,
        taxonomy: &str,
        append: bool,
    ) -> Result<PhpValue, PostError>;
    /// The names of `get_object_taxonomies( $post_type )`, in registration order. `get_object_taxonomies()` runs no
    /// hooks, so asking for names and for objects ([`Taxonomy::default_terms`]) apart is not observable.
    fn object_taxonomies(&self, post_type: &PostTypeName) -> Result<Vec<String>, PostError>;
    /// The names of the taxonomies in `get_object_taxonomies( $post_type, 'object' )` whose `default_term` is not
    /// empty, in registration order: the ones `wp_insert_post()` and `wp_publish_post()` give a default term.
    fn default_terms(&self, post_type: &PostTypeName) -> Result<Vec<String>, PostError>;
    /// `get_taxonomy( $taxonomy )`: `None` where it returns `false`; otherwise its `cap->assign_terms`, as PHP holds
    /// it, which `wp_insert_post()`'s `tax_input` checks with `current_user_can()`.
    fn assign_terms_cap(&self, taxonomy: &str) -> Result<Option<PhpValue>, PostError>;
    /// `wp_get_object_terms( $post_id, $taxonomy, $args )`: what it returns, as PHP holds it.
    fn get_object_terms(
        &self,
        post: PostId,
        taxonomy: &str,
        args: &PhpValue,
    ) -> Result<PhpValue, PostError>;
    /// `get_the_terms( $post, $taxonomy )`: the terms, `false` or a `WP_Error`, as PHP holds them.
    /// `wp_update_post()` compares `tags_input` with their names; `wp_publish_post()` checks them for emptiness.
    fn get_the_terms(&self, post: PostId, taxonomy: &str) -> Result<PhpValue, PostError>;
    /// `is_object_in_taxonomy( $post_type, $taxonomy )`.
    fn is_object_in_taxonomy(
        &self,
        post_type: &PostTypeName,
        taxonomy: &str,
    ) -> Result<bool, PostError>;
    /// `wp_delete_object_term_relationships( $post_id, $taxonomies )`.
    fn delete_object_term_relationships(
        &self,
        post: PostId,
        taxonomies: &[String],
    ) -> Result<(), PostError>;
}

/// The comment functions the post functions call (comment.php, and post.php's comment trashing, which stays
/// WordPress's with the comment statements it sends).
pub trait Comments {
    /// `get_default_comment_status( $post_type, $comment_type )`: `comment` for `comment_status`, `pingback` for
    /// `ping_status`. What its filter returns, as PHP holds it.
    fn default_comment_status(
        &self,
        post_type: &PostTypeName,
        comment_type: &str,
    ) -> Result<PhpValue, PostError>;
    /// `wp_trash_post_comments( $post_id )`, which `wp_trash_post()` calls after the post is trashed.
    fn trash_post_comments(&self, post: PostId) -> Result<(), PostError>;
    /// `wp_untrash_post_comments( $post_id )`, which `wp_untrash_post()` calls after the post is restored.
    fn untrash_post_comments(&self, post: PostId) -> Result<(), PostError>;
    /// `wp_delete_post()`'s comment loop (post.php 4118-4124): `SELECT comment_ID FROM {comments} WHERE
    /// comment_post_ID = %d ORDER BY comment_ID DESC` through `$wpdb->get_col()`, then `wp_delete_comment( $id, true
    /// )` for each id, in that order.
    fn delete_post_comments(&self, post: PostId) -> Result<(), PostError>;
    /// `wp_defer_comment_counting( $defer )`; `false` flushes the deferred counts (`wp_update_comment_count( null,
    /// true )`).
    fn defer_counting(&self, defer: bool) -> Result<(), PostError>;
}

/// The rewrite functions the post functions call (link-template.php).
pub trait Rewrite {
    /// `get_permalink( $post_id )`, for the GUID: `None` where it returns `false`.
    fn permalink(&self, post: PostId) -> Result<Option<Vec<u8>>, PostError>;
}

/// The attachment code the post functions hand over to (post.php's and media.php's attachment functions), which phase
/// 7 ports.
pub trait Uploads {
    /// `wp_insert_attachment( $postarr, false, 0, $wp_error )`: `wp_update_post()`'s branch for an attachment. What
    /// it returns (the id, `0` or a `WP_Error`), as PHP holds it.
    fn insert_attachment(&self, postarr: PhpValue, wp_error: bool) -> Result<PhpValue, PostError>;
    /// `wp_delete_attachment( $post_id, $force_delete )`: `wp_delete_post()`'s branch for an attachment. What it
    /// returns (the `WP_Post`, `false` or `null`), as PHP holds it.
    fn delete_attachment(&self, post: PostId, force: bool) -> Result<PhpValue, PostError>;
    /// `update_attached_file( $post_id, $file )`, for an attachment's `file` argument.
    fn update_attached_file(&self, post: PostId, file: &PhpValue) -> Result<(), PostError>;
    /// `wp_attachment_is( $type, $post_id )`, for an audio or video attachment's thumbnail support.
    fn attachment_is(&self, kind: &str, post: PostId) -> Result<bool, PostError>;
    /// `set_post_thumbnail( $post_id, $thumbnail_id )`, for `_thumbnail_id`.
    fn set_post_thumbnail(&self, post: PostId, thumbnail: i64) -> Result<(), PostError>;
    /// `delete_post_thumbnail( $post_id )`, for `_thumbnail_id` `-1`.
    fn delete_post_thumbnail(&self, post: PostId) -> Result<(), PostError>;
}

/// The theme functions the post functions call (theme.php, class-wp-theme.php).
pub trait Theme {
    /// `wp_get_theme()->get_page_templates( $post )`, with `$post->page_template` set to `page_template`: the
    /// templates, file name to name, as PHP holds them after the `theme_{$post_type}_templates` filter.
    fn page_templates(&self, post: &Post, page_template: &PhpValue) -> Result<PhpValue, PostError>;
    /// `current_theme_supports( $feature, $arg )`: `post-thumbnails` for a post type, or for `attachment:audio` and
    /// `attachment:video`.
    fn current_theme_supports(&self, feature: &str, arg: &str) -> Result<bool, PostError>;
}

/// The revision functions the post functions call (revision.php), which phase 6 ports.
pub trait Revisions {
    /// `wp_save_post_revision( $post_id )`.
    fn save_post_revision(&self, post: PostId) -> Result<(), PostError>;
    /// `wp_delete_post_revision( $revision_id )`, which `wp_delete_post()` calls for each revision.
    fn delete_post_revision(&self, revision: PostId) -> Result<(), PostError>;
}

/// The database's column charsets, for `wp_insert_post()`'s emoji check.
pub trait Charset {
    /// `$wpdb->get_col_charset( $table, $column )`, its `pre_get_col_charset` filter included: the charset name
    /// (`utf8mb4`, `utf8`, `utf8mb3`), `None` where it returns `false` or a `WP_Error`. `wp_insert_post()` encodes
    /// emoji in `post_title`, `post_content` and `post_excerpt` with `wp_encode_emoji()` when it is `utf8` or
    /// `utf8mb3`.
    fn col_charset(&self, table: &str, column: &str) -> Result<Option<String>, PostError>;
}
