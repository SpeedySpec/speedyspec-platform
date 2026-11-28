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

/// A post object a PHP caller passed or WordPress built: its typed columns, the dynamic properties WordPress set on
/// it, and, when there is one, the PHP object itself.
///
/// WordPress passes post objects by handle:
///
/// - the hooks `wp_transition_post_status()` fires receive the caller's object;
/// - `wp_publish_post()` sets `$post->post_status = 'publish'` on the object it was given;
/// - `wp_insert_post()` builds `$post = get_post( $post_id )`, sets a dynamic `page_template` property on it (post.php
///   5402), and passes that one object to `wp_transition_post_status()`, `edit_post`, `save_post_{$post_type}`,
///   `save_post`, `wp_insert_post` and `wp_after_insert_post`; `wp_publish_post()`, `wp_trash_post()`,
///   `wp_untrash_post()` and `wp_delete_post()` likewise fire several hooks with one `get_post()` instance.
///
/// `handle` is that object, carried opaque ([`HookValue::Opaque`] for a PHP object), so an implementation passes the
/// same value to every hook it fires and writes back to it through the binding; the caller sees the changes. With
/// `handle` as `None` there is no PHP object yet: the binding builds one `WP_Post` from `post` and `properties`, and a
/// use case keeps the object it gets back (from [`PostWrites::transition`]) for its later hooks, so one call builds
/// one object.
///
/// `properties` are the dynamic properties WordPress sets on the object beyond its columns, in the order it sets them
/// (`page_template`). The binding sets them on the object it builds; with a handle, they are already on it.
///
/// It is `#[non_exhaustive]`: build it with [`PostObject::new`] or [`PostObject::with_handle`], then
/// [`PostObject::with_property`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PostObject {
    /// The object's columns, typed.
    pub post: Post,
    /// The PHP object, when there is one.
    pub handle: Option<HookValue>,
    /// The dynamic properties WordPress set on the object, by name, in the order it set them.
    pub properties: Vec<(String, HookValue)>,
}

impl PostObject {
    /// A post with no PHP object yet and no dynamic properties.
    pub fn new(post: Post) -> Self {
        Self {
            post,
            handle: None,
            properties: Vec::new(),
        }
    }

    /// A post and the PHP object it was read from.
    pub fn with_handle(post: Post, handle: HookValue) -> Self {
        Self {
            handle: Some(handle),
            ..Self::new(post)
        }
    }

    /// The same object with a dynamic property set: replaced in place when it is already set, otherwise appended, as
    /// PHP keeps a property's first position.
    pub fn with_property(mut self, name: impl Into<String>, value: HookValue) -> Self {
        let name = name.into();
        match self.properties.iter_mut().find(|(n, _)| *n == name) {
            Some((_, slot)) => *slot = value,
            None => self.properties.push((name, value)),
        }
        self
    }

    /// One dynamic property.
    pub fn property(&self, name: &str) -> Option<&HookValue> {
        self.properties
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}

/// A post argument that WordPress runs through `get_post()`: an id, the global `$post`, or an object.
///
/// `get_post()` hands back a raw `WP_Post` it is given as that same instance, so the functions taking a
/// [`PostArg::Object`] read the object's own values (not the table's), fire their hooks with it, and return it.
///
/// Only a `WP_Post` whose `filter` is `raw` keeps its identity that way. `get_post()` re-reads any other
/// `WP_Post` (`WP_Post::filter( 'raw' )` calls `get_instance()`, class-wp-post.php 423-424), unset `filter` included, and copies a `stdClass` into a new
/// `WP_Post` (post.php 1153-1159), so the binding sends those as [`PostArg::Ref`] with the id, or as an object with
/// no handle ([`PostObject::new`]).
///
/// An empty argument falls back to the global `$post`. [`crate::CurrentPost`] reads it as a value with no handle, so
/// to keep its identity the binding passes the global `WP_Post` as [`PostArg::Object`] with its handle; it passes
/// [`PostRef::Current`] only when it has no object to pass (the global is unset, or holds something other than a
/// `WP_Post`).
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
    ///
    /// WordPress returns nothing. The answer is the object the hooks received, its handle set: `post`'s own, or the
    /// one the binding built from `post` and its properties when it had none. A use case passes that object to the
    /// hooks it fires afterwards (`edit_post`, `save_post`, `wp_insert_post`, `wp_after_insert_post`), so they share
    /// one object as WordPress's do.
    fn transition(
        &self,
        new: &PostStatusName,
        old: &PostStatusName,
        post: &PostObject,
    ) -> Result<PostObject, PostError>;

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
