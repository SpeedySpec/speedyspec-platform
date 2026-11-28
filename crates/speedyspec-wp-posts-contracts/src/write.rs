//! Writing posts as one contract: a method per post-writing function of WordPress's `post.php`.
//!
//! Command/CQRS: [`PostWrites`] is the command side, apart from the [`crate::Posts`] queries, and each method is one
//! handler, bound in the container. Template Method: `wp_insert_post()` runs as ordered steps, in WordPress's order.
//! Observer: implementations run WordPress's hooks through the hook API. Adapter: the code that is not ported is
//! reached through the ports in this crate, and options and cron through their own contracts.

use speedyspec_php_serialize::PhpValue;
use speedyspec_wp_hooks_contracts::HookValue;

use crate::{Post, PostError, PostId, PostRef, PostStatusName, PostWriteError};

/// What `wp_insert_post()` and `wp_update_post()` answer: the post's id, or the `WP_Error` they return. `Ok(PostId(0))`
/// is a `0` WordPress returns without an error (code still WordPress's answered it).
///
/// The id is always an integer in PHP, so a [`PostId`] carries it whole. On the update path WordPress returns
/// `$postarr['ID']` uncast (post.php 4884, 5544), but only after `sanitize_post( $postarr, 'db' )`, which casts `ID`
/// with `(int)` before and after its `pre_post_ID` and `ID_pre` filters (`sanitize_post_field()`'s `$int_fields`).
/// Checked on WordPress 7.2-RC2: `wp_update_post( array( 'ID' => '8' ) )` returns the int `8`. A value at or below 0
/// never gets that far: `get_post()` finds no post and WordPress answers `invalid_post`.
pub type WriteResult = Result<PostId, PostWriteError>;

/// A post object a PHP caller passed or WordPress built: its typed columns and, when there is one, the PHP object
/// itself.
///
/// WordPress passes post objects by handle: the hooks `wp_transition_post_status()` fires receive the caller's object,
/// `wp_publish_post()` sets `$post->post_status = 'publish'` on the object it was given, and `wp_insert_post()` sets a
/// dynamic `page_template` property on the object its later hooks receive. `handle` is that object, carried opaque
/// ([`HookValue::Opaque`] for a PHP object), so an implementation passes the same value to every hook and writes back
/// to it through the binding; the caller sees the changes. With `handle` as `None` there is no PHP object yet, and the
/// binding builds one `WP_Post` from `post` and passes that same one to every hook of the call.
#[derive(Clone, Debug, PartialEq)]
pub struct PostObject {
    /// The object's columns, typed.
    pub post: Post,
    /// The PHP object, when there is one.
    pub handle: Option<HookValue>,
}

impl PostObject {
    /// A post with no PHP object yet.
    pub fn new(post: Post) -> Self {
        Self { post, handle: None }
    }

    /// A post and the PHP object it was read from.
    pub fn with_handle(post: Post, handle: HookValue) -> Self {
        Self {
            post,
            handle: Some(handle),
        }
    }
}

/// A post argument that WordPress runs through `get_post()`: an id, the global `$post`, or an object.
///
/// `get_post()` hands back a raw `WP_Post` it is given as that same instance, so the functions taking a
/// [`PostArg::Object`] read the object's own values (not the table's), fire their hooks with it, and return it.
#[derive(Clone, Debug, PartialEq)]
pub enum PostArg {
    /// An id, or the global `$post` (an empty argument).
    Ref(PostRef),
    /// A post object. Boxed: a post is large.
    Object(Box<PostObject>),
}

impl From<PostRef> for PostArg {
    fn from(post: PostRef) -> Self {
        PostArg::Ref(post)
    }
}

impl From<PostObject> for PostArg {
    fn from(post: PostObject) -> Self {
        PostArg::Object(Box::new(post))
    }
}

/// What `wp_delete_post()`, `wp_trash_post()` and `wp_untrash_post()` return: the post, or another PHP value.
#[derive(Clone, Debug, PartialEq)]
pub enum PostReturn {
    /// The `WP_Post`, as it was read before the change: the caller's own object when it passed one (its handle), or
    /// one the binding builds. Boxed: a post is large.
    Post(Box<PostObject>),
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
/// ids, `wp_delete_post()`'s id). A post argument WordPress runs through `get_post()` is a [`PostArg`], which keeps a
/// passed object's identity.
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

    /// `wp_trash_post( $post_id )`, which returns the post it read: the object, when it was given one.
    fn trash(&self, post: PostArg) -> Result<PostReturn, PostError>;

    /// `wp_untrash_post( $post_id )`, which returns the post it read: the object, when it was given one.
    fn untrash(&self, post: PostArg) -> Result<PostReturn, PostError>;

    /// `wp_publish_post( $post )`, which returns nothing. Given an object, it reads that object's status, sets its
    /// `post_status` to `publish` (post.php 5690-5692) and fires the transition and `save_post` hooks with it.
    fn publish(&self, post: PostArg) -> Result<(), PostError>;

    /// `check_and_publish_future_post( $post )`: publishes a `future` post whose time has come, or schedules
    /// `publish_future_post` again for one whose time has not. Given an object, it reads that object's status and
    /// date.
    fn check_and_publish_future(&self, post: PostArg) -> Result<(), PostError>;

    /// `wp_transition_post_status( $new_status, $old_status, $post )`: `transition_post_status`, then
    /// `{$old_status}_to_{$new_status}` and `{$new_status}_{$post_type}` as the statuses and post type are registered.
    /// `old` may be `new`, as `wp_insert_post()` passes for a new post. The three hooks receive `post`'s handle, the
    /// same object each time.
    fn transition(
        &self,
        new: &PostStatusName,
        old: &PostStatusName,
        post: &PostObject,
    ) -> Result<(), PostError>;

    /// `stick_post( $post_id )`; `post_id` is `(int) $post_id`, the value added to `sticky_posts`.
    fn stick(&self, post_id: i64) -> Result<(), PostError>;

    /// `unstick_post( $post_id )`; `post_id` is `(int) $post_id`.
    fn unstick(&self, post_id: i64) -> Result<(), PostError>;

    /// `add_ping( $post, $uri )`; `uri` is a string or an array, as PHP holds it. The rows the update changed, `None`
    /// where WordPress returns `false` (no post, or a failed update). Given an object, it reads that object's
    /// `pinged`.
    fn add_ping(&self, post: PostArg, uri: &PhpValue) -> Result<Option<u64>, PostError>;

    /// `set_post_type( $post_id, $post_type )`; `post_id` as `%d` prints it (so `-3` updates no row), `post_type` as
    /// PHP holds it, before `sanitize_post_field( 'post_type', $post_type, $post_id, 'db' )`. The rows the update
    /// changed, `None` where WordPress returns `false`.
    fn set_post_type(&self, post_id: i64, post_type: &PhpValue) -> Result<Option<u64>, PostError>;

    /// `wp_delete_auto_drafts()`: force-deletes every auto-draft more than a week old.
    fn delete_auto_drafts(&self) -> Result<(), PostError>;
}
