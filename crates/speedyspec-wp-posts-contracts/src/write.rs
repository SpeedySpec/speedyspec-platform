//! Writing posts as one contract: a method per post-writing function of WordPress's `post.php`.
//!
//! Command/CQRS: [`PostWrites`] is the command side, apart from the [`crate::Posts`] queries, and each method is one
//! handler, bound in the container. Template Method: `wp_insert_post()` runs as ordered steps, in WordPress's order.
//! Observer: implementations run WordPress's hooks through the hook API. Adapter: the code that is not ported is
//! reached through the ports in this crate, and options and cron through their own contracts.

use speedyspec_php_serialize::PhpValue;

use crate::{Post, PostError, PostId, PostRef, PostStatusName, PostWriteError};

/// What `wp_insert_post()` and `wp_update_post()` answer: the post's id, or the `WP_Error` they return. `Ok(PostId(0))`
/// is a `0` WordPress returns without an error (code still WordPress's answered it).
pub type WriteResult = Result<PostId, PostWriteError>;

/// What `wp_delete_post()`, `wp_trash_post()` and `wp_untrash_post()` return: the post, or another PHP value.
#[derive(Clone, Debug, PartialEq)]
pub enum PostReturn {
    /// The `WP_Post`, as it was read before the change (the binding builds the object). Boxed: a post is large.
    Post(Box<Post>),
    /// Anything else, as PHP holds it: `false`, `null`, what a `pre_delete_post`, `pre_trash_post` or
    /// `pre_untrash_post` filter returned, or what `wp_delete_attachment()` returned.
    Value(PhpValue),
}

/// Writing posts (`wp_insert_post()` and the functions beside it), WordPress 7.2-RC2.
///
/// Every method stands for the WordPress function its doc names and runs that function's filters, actions, cache
/// writes and statements, in WordPress's order. `Err` carries a hook callback's failure, a cache error, or a database
/// failure `$wpdb` would not hide; whatever WordPress wrote before it stays written, and nothing after it runs.
///
/// Arguments are PHP-shaped where WordPress takes an array (`$postarr`, `$uri`) or casts with `(int)` (the sticky
/// ids, `wp_delete_post()`'s id). A post id that falls back to the global `$post` when empty is a [`PostRef`].
pub trait PostWrites {
    /// `wp_insert_post( $postarr, $wp_error, $fire_after_hooks )`. `wp_error` changes what WordPress does, not only
    /// what it returns: without it an invalid `page_template` is stored as `default` and the insert goes on.
    fn insert(
        &self,
        postarr: PhpValue,
        wp_error: bool,
        fire_after_hooks: bool,
    ) -> Result<WriteResult, PostError>;

    /// `wp_update_post( $postarr, $wp_error, $fire_after_hooks )`; `postarr` may be an object, which WordPress reads
    /// with `get_object_vars()` and slashes. An attachment goes to `wp_insert_attachment()` through
    /// [`crate::Uploads`], whose `WP_Error` comes back as [`PostWriteError::Delegated`].
    fn update(
        &self,
        postarr: PhpValue,
        wp_error: bool,
        fire_after_hooks: bool,
    ) -> Result<WriteResult, PostError>;

    /// `wp_delete_post( $post_id, $force_delete )`; `post_id` is `(int) $post_id`, so `0` or below calls
    /// `_doing_it_wrong()` and returns `false`. A post or page that is not trashed is trashed instead, while
    /// `EMPTY_TRASH_DAYS` is set; an attachment goes to `wp_delete_attachment()` through [`crate::Uploads`].
    fn delete(&self, post_id: i64, force: bool) -> Result<PostReturn, PostError>;

    /// `wp_trash_post( $post_id )`.
    fn trash(&self, post: PostRef) -> Result<PostReturn, PostError>;

    /// `wp_untrash_post( $post_id )`.
    fn untrash(&self, post: PostRef) -> Result<PostReturn, PostError>;

    /// `wp_publish_post( $post )`, which returns nothing.
    fn publish(&self, post: PostRef) -> Result<(), PostError>;

    /// `check_and_publish_future_post( $post )`: publishes a `future` post whose time has come, or schedules
    /// `publish_future_post` again for one whose time has not.
    fn check_and_publish_future(&self, post: PostRef) -> Result<(), PostError>;

    /// `wp_transition_post_status( $new_status, $old_status, $post )`: `transition_post_status`, then
    /// `{$old_status}_to_{$new_status}` and `{$new_status}_{$post_type}` as the statuses and post type are registered.
    /// `old` may be `new`, as `wp_insert_post()` passes for a new post.
    fn transition(
        &self,
        new: &PostStatusName,
        old: &PostStatusName,
        post: &Post,
    ) -> Result<(), PostError>;

    /// `stick_post( $post_id )`; `post_id` is `(int) $post_id`, the value added to `sticky_posts`.
    fn stick(&self, post_id: i64) -> Result<(), PostError>;

    /// `unstick_post( $post_id )`; `post_id` is `(int) $post_id`.
    fn unstick(&self, post_id: i64) -> Result<(), PostError>;

    /// `add_ping( $post, $uri )`; `uri` is a string or an array, as PHP holds it. The rows the update changed, `None`
    /// where WordPress returns `false` (no post, or a failed update).
    fn add_ping(&self, post: PostRef, uri: &PhpValue) -> Result<Option<u64>, PostError>;

    /// `set_post_type( $post_id, $post_type )`; `post_type` as PHP holds it, before `sanitize_post_field( 'post_type',
    /// $post_type, $post_id, 'db' )`. The rows the update changed, `None` where WordPress returns `false`.
    fn set_post_type(&self, post: PostId, post_type: &PhpValue) -> Result<Option<u64>, PostError>;

    /// `wp_delete_auto_drafts()`: force-deletes every auto-draft more than a week old.
    fn delete_auto_drafts(&self) -> Result<(), PostError>;
}
