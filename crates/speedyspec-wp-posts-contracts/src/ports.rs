//! The ports the post entity, post type and status rules read and write through; adapters implement them at the
//! edge.
//!
//! Every [`PostRepository`] method is one `$wpdb` call `post.php` makes, so query counts match. A failure `$wpdb`
//! hides is `Ok` with the empty answer; `Err` is only for failures it would not hide. Rows are compared byte for
//! byte; collation is the SQL adapter's.
//!
//! The statement text each method documents is WordPress 7.2-RC2's, whitespace included. In the multi-line ones each
//! `⇥` stands for one tab, as `post.php` indents them; the lines are joined with newlines (`\n`), the first line is
//! empty, and nothing follows the last. The SQL repository sends exactly that text, because the `query` filter sees
//! it. Values are spliced as WordPress splices them: `%s` and `%d` by `$wpdb->prepare()`, quoted lists with
//! `esc_sql()` on each item, id lists as plain integers. The statements `$wpdb->insert`, `$wpdb->update` and
//! `$wpdb->delete` send are the text `wpdb` builds for them: the table and columns in backticks, `%d` for the columns
//! `$wpdb->field_types` lists and `%s` for the rest, then `prepare()`d.

use speedyspec_php_serialize::PhpValue;

use crate::{
    LastPostField, MimeFilter, MimeType, Post, PostDateTime, PostError, PostId, PostParent,
    PostRow, PostStatusName, PostStatusObject, PostType, PostTypeName, Slug, Timezone,
    WriteOutcome,
};

/// One row `get_page_by_path()` selects: `ID, post_name, post_parent, post_type`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRow {
    /// `ID`.
    pub id: PostId,
    /// `post_name`.
    pub name: Slug,
    /// `post_parent`.
    pub parent: PostId,
    /// `post_type`.
    pub post_type: PostTypeName,
}

/// `wp_posts` (Repository). `{posts}` is `$wpdb->posts`.
pub trait PostRepository {
    /// The name of the engine behind this repository (Strategy per engine).
    fn engine(&self) -> String;
    /// `WP_Post::get_instance()`: `SELECT * FROM {posts} WHERE ID = %d LIMIT 1`.
    fn find(&self, id: PostId) -> Result<Option<Post>, PostError>;
    /// `wp_insert_post()`'s `$wpdb->insert( $wpdb->posts, $data )` (post.php 5268): `row` is the filtered `$data`
    /// ([`PostRow`]), its keys the statement's columns in order. [`WriteOutcome::Written`] with the new id
    /// (`$wpdb->insert_id`), or [`WriteOutcome::Failed`] with `$wpdb->last_error` where `$wpdb->insert` returns false:
    ///
    /// - before any statement, when `process_fields()` fails: a [`crate::RowValue::Str`] that is invalid text for its
    ///   column's charset or longer than the column, or a failed charset lookup;
    /// - at the engine: a duplicate `ID`, an unknown column, an empty row, or `NULL` in any column but `ID`, which
    ///   MySQL rejects in an `INSERT` even outside strict mode (error 1048). `ID` is `AUTO_INCREMENT`, so `NULL` there
    ///   takes the next id, as `0` and a negative value do.
    ///
    /// `wpdb::_insert_replace_helper()` builds it: each key in backticks, joined with `` `, ` ``; each value's
    /// placeholder by [`crate::column_format`] (`%d` or `%s`), or `NULL` for a null value, joined with `, `; then
    /// `prepare()`s it, printing each value as [`crate::RowValue`] says. A statement with no placeholder (the empty
    /// row, or a row of nulls) makes `prepare()` call `_doing_it_wrong( 'wpdb::prepare', … must have a placeholder.,
    /// '3.9.0' )` first, whose hooks may throw ([`PostError::Hook`]). With the row [`PostRow::from_post`] builds,
    /// unchanged by any callback:
    ///
    /// ```text
    /// INSERT INTO `{posts}` (`post_author`, `post_date`, `post_date_gmt`, `post_content`, `post_content_filtered`, `post_title`, `post_excerpt`, `post_status`, `post_type`, `comment_status`, `ping_status`, `post_password`, `post_name`, `to_ping`, `pinged`, `post_modified`, `post_modified_gmt`, `post_parent`, `menu_order`, `post_mime_type`, `guid`) VALUES (%d, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %d, %d, %s, %s)
    /// ```
    ///
    /// A column the row leaves out takes the engine's default (MySQL: `0`, `''`, `0000-00-00 00:00:00`, and the
    /// schema's `publish`, `open`, `open` and `post` for `post_status`, `comment_status`, `ping_status` and
    /// `post_type`; the same as [`Post::default`]). A value the column cannot hold is stored as MySQL converts it under
    /// WordPress's `sql_mode`: a negative `post_parent` or `post_author` as `0`, a negative or zero `ID` as the next
    /// auto-increment id ([`PostParent::stored`]).
    ///
    /// `ID` is in the row when a callback added it, or when `wp_insert_post()` set `$data['ID']`: it casts a non-empty
    /// `import_id` with `(int)` (so `'abc'` is `0` and `-5` is `-5`; `0`, `'0'` and `''` are empty and set nothing),
    /// asks [`PostRepository::import_id_taken`] with the cast value, and sets `ID` when no row has it, replacing a
    /// callback's `ID` in its place. A row that holds `comment_count` writes it.
    fn insert(&self, row: &PostRow) -> Result<WriteOutcome<PostId>, PostError>;
    /// `wp_unique_post_slug()`, attachments:
    /// `SELECT post_name FROM {posts} WHERE post_name = %s AND ID != %d LIMIT 1`.
    fn attachment_slug_owner(&self, slug: &Slug, except: PostId)
    -> Result<Option<Slug>, PostError>;
    /// `wp_unique_post_slug()`, hierarchical types: `SELECT post_name FROM {posts} WHERE post_name = %s AND
    /// post_type IN ( %s, 'attachment' ) AND ID != %d AND post_parent = %d LIMIT 1`.
    ///
    /// A negative `parent` is sent as it is (`post_parent = -5`) and matches no row, since the column is unsigned; a
    /// [`PostParent::Null`] one is sent as `0` ([`PostParent::prepared`]).
    fn hierarchical_slug_owner(
        &self,
        slug: &Slug,
        post_type: &PostTypeName,
        except: PostId,
        parent: PostParent,
    ) -> Result<Option<Slug>, PostError>;
    /// `wp_unique_post_slug()`, flat types:
    /// `SELECT post_name FROM {posts} WHERE post_name = %s AND post_type = %s AND ID != %d LIMIT 1`.
    fn flat_slug_owner(
        &self,
        slug: &Slug,
        post_type: &PostTypeName,
        except: PostId,
    ) -> Result<Option<Slug>, PostError>;

    /// `_prime_post_caches()` (post.php 8741): `SELECT {posts}.* FROM {posts} WHERE ID IN (%s)`, where `%s` is the
    /// ids joined with `,` (`sprintf()`, not `prepare()`).
    ///
    /// The rows for the ids that exist, once each; there is no `ORDER BY`, so the order is the engine's. Empty `ids`
    /// sends no statement (WordPress never sends `IN ()`) and answers empty.
    fn find_many(&self, ids: &[PostId]) -> Result<Vec<Post>, PostError>;
    /// `_prime_post_parent_id_caches()` (post.php 8794):
    /// `SELECT {posts}.ID, {posts}.post_parent FROM {posts} WHERE ID IN (%s)`, the ids joined with `,`.
    ///
    /// `(ID, post_parent)` for each id that exists, in the engine's order. Empty `ids` sends no statement.
    fn parent_ids(&self, ids: &[PostId]) -> Result<Vec<(PostId, PostId)>, PostError>;
    /// `get_page_by_path()` (post.php 6532-6540), `$wpdb->get_results( $sql, OBJECT_K )`:
    ///
    /// ```text
    ///
    /// ⇥⇥SELECT ID, post_name, post_parent, post_type
    /// ⇥⇥FROM {posts}
    /// ⇥⇥WHERE post_name IN ({names})
    /// ⇥⇥AND post_type IN ({types})
    /// ⇥⇥ORDER BY
    /// ⇥⇥⇥post_status = 'publish' DESC,
    /// ⇥⇥⇥post_status IN ('draft', 'pending', 'auto-draft') ASC, ID ASC
    /// ⇥
    /// ```
    ///
    /// `{names}` is `'` + the names, each through `esc_sql()`, joined with `','` + `'`; `{types}` the same for the
    /// post types. The caller has already `sanitize_title_for_query()`'d the names and expanded a single post type to
    /// `[ $post_type, 'attachment' ]`.
    ///
    /// Rows in the statement's order: published first, then the other statuses, then `draft`, `pending` and
    /// `auto-draft`, each by `ID`. Whether `post_name` matches ignoring case is the engine's collation: the SQL
    /// repository on stock tables does (`utf8mb4_unicode_520_ci`, checked by `conformance::mysql_suite`), the
    /// memory repository compares bytes. Empty `names` or `post_types` sends no statement and answers empty
    /// (WordPress always has at least one of each).
    fn pages_by_names(
        &self,
        names: &[Slug],
        post_types: &[PostTypeName],
    ) -> Result<Vec<PageRow>, PostError>;
    /// `get_all_page_ids()` (post.php 6435): `SELECT ID FROM {posts} WHERE post_type = 'page'`.
    ///
    /// Every page's id, whatever its status, in the engine's order.
    fn all_page_ids(&self) -> Result<Vec<PostId>, PostError>;
    /// `wp_count_posts()` (post.php 3611-3635), through `$wpdb->prepare()`. With `reader` as `None`, `%s` the post
    /// type; this is the statement for every `$perm` but `readable`, for a logged-out reader, and for a reader who may
    /// read private posts:
    ///
    /// ```text
    ///
    /// ⇥⇥⇥SELECT post_status, COUNT(*) AS num_posts
    /// ⇥⇥⇥FROM {posts}
    /// ⇥⇥⇥WHERE post_type = %s
    /// ⇥⇥ GROUP BY post_status
    /// ```
    ///
    /// With `reader` as `Some( user_id )`, for a logged-in reader who cannot read private posts under `readable`, the
    /// first `%s` and the second the post type and `%d` the reader's user id:
    ///
    /// ```text
    ///
    /// ⇥⇥⇥SELECT post_status, COUNT(*) AS num_posts
    /// ⇥⇥⇥FROM (
    /// ⇥⇥⇥⇥SELECT post_status
    /// ⇥⇥⇥⇥FROM {posts}
    /// ⇥⇥⇥⇥WHERE post_type = %s AND post_status != 'private'
    /// ⇥⇥⇥⇥UNION ALL
    /// ⇥⇥⇥⇥SELECT post_status
    /// ⇥⇥⇥⇥FROM {posts}
    /// ⇥⇥⇥⇥WHERE post_type = %s AND post_status = 'private' AND post_author = %d
    /// ⇥⇥⇥) AS filtered_posts
    /// ⇥⇥ GROUP BY post_status
    /// ```
    ///
    /// The status that branch hides unless the reader wrote the post is `private`, hard-coded in the statement as
    /// WordPress 7.2-RC2 has it; a registered status with `private => true` is not hidden.
    ///
    /// Each status that has posts with its count; no `ORDER BY`, so the engine's order. A type with no posts answers
    /// empty.
    fn count_by_status(
        &self,
        post_type: &PostTypeName,
        reader: Option<u64>,
    ) -> Result<Vec<(PostStatusName, u64)>, PostError>;
    /// `wp_count_attachments()` (post.php 3730 and 3736): two statements, in this order, where `$and` is
    /// `wp_post_mime_type_where( $mime_type )`:
    ///
    /// - `SELECT post_mime_type, COUNT( * ) AS num_posts FROM {posts} WHERE post_type = 'attachment' AND post_status
    ///   != 'trash' $and GROUP BY post_mime_type`
    /// - `SELECT COUNT( * ) FROM {posts} WHERE post_type = 'attachment' AND post_status = 'trash' $and`
    ///
    /// With no filter `$and` is empty, so `'trash'` and `GROUP BY` have two spaces between them, and the second
    /// statement ends in a space. Otherwise `$and` is ` AND (` + the clauses joined with ` OR ` + `) `.
    ///
    /// The SQL repository builds `$and` by `wp_post_mime_type_where()`'s rule, and the memory repository matches as
    /// those clauses would. The items are [`MimeFilter`]'s (a string split on `,`, trimmed, empties dropped; an array
    /// as given). For each item in turn, with its whitespace removed:
    ///
    /// - if the item itself is `''`, `%` or `%/%`, the whole `$and` is `''` (everything matches), even when earlier
    ///   items have produced clauses;
    /// - otherwise it becomes a pattern: `group/subgroup` keeps `-*.a-zA-Z0-9` in the group and `-*.+a-zA-Z0-9` in the
    ///   subgroup, an empty subgroup becoming `*`; an item with no `/` keeps `-*.a-zA-Z0-9` and gains `/*` unless it
    ///   holds a `*`; then every run of `*` becomes `%`;
    /// - a pattern holding `%` gives `post_mime_type LIKE '{pattern}'`, any other `post_mime_type = '{pattern}'`.
    ///
    /// So `image` gives `LIKE 'image/%'`, `image/png` gives `= 'image/png'`, and `*` gives `LIKE '%'` (not an empty
    /// `$and`: only the raw item is compared with the wildcards). `None`, or a filter with no items left, is an empty
    /// `$and`.
    ///
    /// The untrashed counts by mime type, in the engine's order, and the trashed count.
    fn count_attachments(
        &self,
        mime_type: Option<&MimeFilter>,
    ) -> Result<(Vec<(MimeType, u64)>, u64), PostError>;
    /// `_get_last_post_time()` (post.php 8125-8132), `{field}` being `date` or `modified` and `{types}` `'` + the
    /// post types, each through `esc_sql()`, joined with `', '` + `'` (`''` when there are none):
    ///
    /// - [`Timezone::Gmt`]: `SELECT post_{field}_gmt FROM {posts} WHERE post_status = 'publish' AND post_type IN
    ///   ({types}) ORDER BY post_{field}_gmt DESC LIMIT 1`
    /// - [`Timezone::Blog`]: `SELECT post_{field} FROM {posts} WHERE post_status = 'publish' AND post_type IN
    ///   ({types}) ORDER BY post_{field}_gmt DESC LIMIT 1`
    /// - [`Timezone::Server`]: `SELECT DATE_ADD(post_{field}_gmt, INTERVAL '0' SECOND) FROM {posts} WHERE
    ///   post_status = 'publish' AND post_type IN ({types}) ORDER BY post_{field}_gmt DESC LIMIT 1`. The `0` is
    ///   PHP's `gmdate( 'Z' )`, which is always `0`.
    ///
    /// The newest published post's date, ordered by the GMT column whichever column is read. For
    /// [`Timezone::Server`] it is `DATE_ADD`'s text, which carries microseconds (`2024-03-01 20:00:00.000000`)
    /// because the interval is a string; WordPress returns that unchanged. A zero date
    /// (`0000-00-00 00:00:00`) comes back as stored: WordPress caches and returns it. `None` when no row matches or
    /// the value is `NULL` (MySQL's `DATE_ADD` on a zero date, checked by `conformance::mysql_suite`).
    fn last_post_time(
        &self,
        field: LastPostField,
        post_types: &[PostTypeName],
        timezone: Timezone,
    ) -> Result<Option<PostDateTime>, PostError>;
    /// `get_available_post_mime_types()` (post.php 8903), through `$wpdb->prepare()`, `%s` the post type:
    /// `SELECT DISTINCT post_mime_type FROM {posts} WHERE post_type = %s AND post_mime_type != ''`.
    ///
    /// Each non-empty mime type once, in the engine's order.
    fn available_mime_types(&self, post_type: &PostTypeName) -> Result<Vec<MimeType>, PostError>;

    /// `wp_insert_post()`'s `$wpdb->update( $wpdb->posts, $data, $where )` (post.php 5236): `row` is the filtered
    /// `$data` ([`PostRow`]), its keys the columns set, in order, where `ID` is `id`. A column the row leaves out keeps
    /// its stored value.
    ///
    /// `wpdb::update()` builds it: `` `column` = `` and the column's placeholder ([`crate::column_format`]), or
    /// `` `column` = NULL `` for a null value, joined with `, `, then ``WHERE `ID` = %d``, and `prepare()`s it,
    /// printing each value as [`crate::RowValue`] says. With the row [`PostRow::from_post`] builds, unchanged by any
    /// callback:
    ///
    /// ```text
    /// UPDATE `{posts}` SET `post_author` = %d, `post_date` = %s, `post_date_gmt` = %s, `post_content` = %s, `post_content_filtered` = %s, `post_title` = %s, `post_excerpt` = %s, `post_status` = %s, `post_type` = %s, `comment_status` = %s, `ping_status` = %s, `post_password` = %s, `post_name` = %s, `to_ping` = %s, `pinged` = %s, `post_modified` = %s, `post_modified_gmt` = %s, `post_parent` = %d, `menu_order` = %d, `post_mime_type` = %s, `guid` = %s WHERE `ID` = %d
    /// ```
    ///
    /// A negative `post_parent` is stored as `0`, and so is `NULL` in any column, which an `UPDATE` stores as the
    /// column's implicit default (`0`, `''`, the zero date) with a warning outside strict mode, unlike an `INSERT`. An
    /// empty row (``UPDATE `{posts}` SET  WHERE `ID` = %d``, two spaces) or an unknown column makes MySQL reject it, and
    /// a failed `process_fields()` sends no statement, as for [`PostRepository::insert`]. A row holding `ID` sets it
    /// (``SET `ID` = %d … WHERE `ID` = %d``).
    ///
    /// [`WriteOutcome::Written`] with the rows changed, as MySQL counts them: an update that matches a row but changes
    /// nothing is `0`, and so is one that matches none. [`WriteOutcome::Failed`] with `$wpdb->last_error` where
    /// `$wpdb->update` returns false.
    fn update(&self, id: PostId, row: &PostRow) -> Result<WriteOutcome<u64>, PostError>;
    /// `$wpdb->update( $wpdb->posts, array( 'post_name' => $name ), array( 'ID' => $id ) )`:
    /// ``UPDATE `{posts}` SET `post_name` = %s WHERE `ID` = %d``.
    ///
    /// `wp_insert_post()` after the insert or update, for a post with no slug that is not a draft (post.php 5291), and
    /// `wp_add_trashed_suffix_to_post_name_for_post()` for the `__trashed` suffix (8865). The rows changed, as
    /// [`PostRepository::update`] counts them.
    fn update_name(&self, id: PostId, name: &Slug) -> Result<Option<u64>, PostError>;
    /// `$wpdb->update( $wpdb->posts, array( 'guid' => get_permalink( $id ) ), array( 'ID' => $id ) )`:
    /// ``UPDATE `{posts}` SET `guid` = %s WHERE `ID` = %d``.
    ///
    /// `wp_insert_post()` for a new post with no GUID (post.php 5362), and `_transition_post_status()` for a post
    /// first published with none (8402). `get_permalink()`'s `false` is written as `''`, as `%s` prints it. The rows
    /// changed.
    fn update_guid(&self, id: PostId, guid: &[u8]) -> Result<Option<u64>, PostError>;
    /// `wp_publish_post()`'s `$wpdb->update( $wpdb->posts, array( 'post_status' => 'publish' ), array( 'ID' =>
    /// $post->ID ) )` (post.php 5687): ``UPDATE `{posts}` SET `post_status` = %s WHERE `ID` = %d``. The rows changed.
    fn update_status(&self, id: PostId, status: &PostStatusName) -> Result<Option<u64>, PostError>;
    /// `add_ping()`'s `$wpdb->update( $wpdb->posts, array( 'pinged' => $new ), array( 'ID' => $post->ID ) )`
    /// (post.php 6285): ``UPDATE `{posts}` SET `pinged` = %s WHERE `ID` = %d``. The rows changed, which `add_ping()`
    /// returns.
    fn update_pinged(&self, id: PostId, pinged: &[u8]) -> Result<Option<u64>, PostError>;
    /// `set_post_type()`'s `$wpdb->update( $wpdb->posts, array( 'post_type' => $post_type ), array( 'ID' => $post_id
    /// ) )` (post.php 2475): ``UPDATE `{posts}` SET `post_type` = %s WHERE `ID` = %d``. The rows changed, which
    /// `set_post_type()` returns. `id` is `set_post_type()`'s `$post_id` as `%d` prints it, so `-3` sends
    /// ``WHERE `ID` = -3``.
    fn update_post_type(&self, id: i64, post_type: &PostTypeName)
    -> Result<Option<u64>, PostError>;
    /// `wp_insert_post()`'s `import_id` check (post.php 5254), through `$wpdb->get_var()`:
    /// `SELECT ID FROM {posts} WHERE ID = %d`, `%d` being the `(int)` cast `import_id`, `0` and negative values
    /// included (no row has them). Whether a row has the id, so the `import_id` is dropped.
    fn import_id_taken(&self, id: i64) -> Result<bool, PostError>;
    /// `wp_delete_post()`'s first read (post.php 4034), through `$wpdb->get_row()`:
    /// `SELECT * FROM {posts} WHERE ID = %d`, with no `LIMIT` (unlike [`PostRepository::find`]). The row, or `None`.
    fn raw_row(&self, id: PostId) -> Result<Option<Post>, PostError>;
    /// `wp_delete_post()`'s children of a hierarchical post (post.php 4090-4096), through `$wpdb->get_results()`:
    /// `SELECT * FROM {posts} WHERE post_parent = %d AND post_type = %s`. The rows, in the engine's order.
    fn children_of_type(
        &self,
        parent: PostId,
        post_type: &PostTypeName,
    ) -> Result<Vec<Post>, PostError>;
    /// `wp_delete_post()`'s `$wpdb->update( $wpdb->posts, $parent_data, $parent_where + array( 'post_type' =>
    /// $post->post_type ) )` (post.php 4099), when [`PostRepository::children_of_type`] found children:
    /// ``UPDATE `{posts}` SET `post_parent` = %d WHERE `post_parent` = %d AND `post_type` = %s``, the first `%d`
    /// `to` (the deleted post's own parent) and the second `from` (the deleted post). The rows changed.
    fn reparent_children(
        &self,
        from: PostId,
        to: PostParent,
        post_type: &PostTypeName,
    ) -> Result<Option<u64>, PostError>;
    /// `wp_delete_post()`'s revision ids (post.php 4105), through `$wpdb->get_col()`:
    /// `SELECT ID FROM {posts} WHERE post_parent = %d AND post_type = 'revision'`. In the engine's order.
    fn revision_ids(&self, parent: PostId) -> Result<Vec<PostId>, PostError>;
    /// `wp_delete_post()`'s `$wpdb->update( $wpdb->posts, $parent_data, $parent_where + array( 'post_type' =>
    /// 'attachment' ) )` (post.php 4114), always sent: the statement of [`PostRepository::reparent_children`] with
    /// `'attachment'` for the `%s`. The rows changed.
    fn reparent_attachments(&self, from: PostId, to: PostParent) -> Result<Option<u64>, PostError>;
    /// `wp_delete_post()`'s `$wpdb->delete( $wpdb->posts, array( 'ID' => $post_id ) )` (post.php 4160), which
    /// `wpdb::delete()` builds: ``DELETE FROM `{posts}` WHERE `ID` = %d``. The rows deleted; `None` where
    /// `$wpdb->delete` returns false. `wp_delete_post()` returns false for `Some(0)` too.
    fn remove(&self, id: PostId) -> Result<Option<u64>, PostError>;
    /// `wp_delete_auto_drafts()` (post.php 8619), through `$wpdb->get_col()`, sent as it is (no `prepare()`):
    /// `SELECT ID FROM {posts} WHERE post_status = 'auto-draft' AND DATE_SUB( NOW(), INTERVAL 7 DAY ) > post_date`.
    ///
    /// `NOW()` is the database's clock in its session time zone; a memory repository uses its own. A zero
    /// `post_date` (`0000-00-00 00:00:00`) is older than any date, so its auto-draft is listed. In the engine's order.
    fn auto_drafts_older_than_a_week(&self) -> Result<Vec<PostId>, PostError>;
}

/// `get_post( $id )`: through whatever cache the binding keeps. Phase 2b's binding has none (`speedyspec-wp-posts`'
/// `Uncached`); phase 4's reads the `posts` group first.
pub trait PostReader {
    /// The post, or `None` when there is none.
    fn post(&self, id: PostId) -> Result<Option<Post>, PostError>;
}

/// The global `$post` (Adapter): what `get_post()` falls back to when its argument is empty.
pub trait CurrentPost {
    /// `$GLOBALS['post']` as PHP holds it (a `WP_Post`, an id, or whatever a plugin put there); `None` when it is
    /// not set.
    fn current(&self) -> Option<PhpValue>;
}

/// `WP_Query` (Adapter), opaque until the query seam is ported: the post functions that query through it
/// (`get_posts()`, `get_children()`, `get_pages()`) stay WordPress's own until then, and this names the call for them.
pub trait PostQuery {
    /// `( new WP_Query() )->query( $args )`: the posts it returns, as PHP holds them.
    fn query(&self, args: PhpValue) -> Result<PhpValue, PostError>;
    /// `get_posts( $args )`, its defaults and `suppress_filters` included: the posts it returns, as PHP holds them.
    /// `wp_insert_post()` finds the trashed posts holding a wanted slug through it
    /// (`wp_add_trashed_suffix_to_post_name_for_trashed_posts()`).
    fn get_posts(&self, args: PhpValue) -> Result<PhpValue, PostError>;
}

/// The time functions WordPress's dates use (a table in tests; Rust and PHP adapters in phase 5).
///
/// The local-time methods read `wp_timezone()` or `get_option( 'gmt_offset' )`, whose option filters run; a callback
/// that throws answers [`PostError::Hook`]. The GMT and timestamp methods run no callback.
pub trait Clock {
    /// `current_time( 'mysql' )`.
    fn now_local(&self) -> Result<String, PostError>;
    /// `current_time( 'mysql', true )`, which is also `gmdate( 'Y-m-d H:i:s' )`.
    fn now_gmt(&self) -> String;
    /// `get_date_from_gmt()`.
    fn date_from_gmt(&self, gmt: &[u8]) -> Result<Vec<u8>, PostError>;
    /// `get_gmt_from_date()`.
    fn gmt_from_date(&self, local: &[u8]) -> Result<Vec<u8>, PostError>;
    /// `strtotime()` in UTC; `None` is false.
    ///
    /// PHP's `time()` (`wp_trash_post()`'s `_wp_trash_meta_time`, `check_and_publish_future_post()`'s comparison) is
    /// `timestamp( now_gmt() )`: the clock has no method of its own for it.
    fn timestamp(&self, datetime: &[u8]) -> Option<i64>;
}

/// `wp_truncate_slug( $slug, $length )` (formatting.php, the text seam's); phase 4's provider binds
/// `speedyspec-wp-text`'s rule.
pub trait SlugTruncation {
    /// The slug, cut to `length` bytes as WordPress cuts it.
    fn truncate(&self, slug: &[u8], length: usize) -> Vec<u8>;
}

/// The `wp_unique_post_slug_is_bad_{attachment,hierarchical,flat}_slug` filters (Strategy), bound in phase 4.
///
/// A callback may throw: the method answers [`PostError::Hook`], and the use case stops before it checks the slug,
/// as WordPress's exception would.
pub trait BadSlugs {
    /// `wp_unique_post_slug_is_bad_attachment_slug`.
    fn bad_attachment_slug(&self, slug: &[u8]) -> Result<bool, PostError>;
    /// `wp_unique_post_slug_is_bad_hierarchical_slug`.
    fn bad_hierarchical_slug(
        &self,
        slug: &[u8],
        post_type: &str,
        parent: PostParent,
    ) -> Result<bool, PostError>;
    /// `wp_unique_post_slug_is_bad_flat_slug`.
    fn bad_flat_slug(&self, slug: &[u8], post_type: &str) -> Result<bool, PostError>;
}

/// `__()` and `_x()` in the `default` domain (WordPress's English when nothing translates).
///
/// The `gettext` and `gettext_with_context` filters (and their per-domain forms) run; a callback that throws answers
/// [`PostError::Hook`].
pub trait Translator {
    /// The translation of `text`, in `context` when there is one.
    fn translate(&self, text: &str, context: Option<&str>) -> Result<String, PostError>;
}

/// `set_props()`' `is_admin() || get_option( 'permalink_structure' )`.
///
/// `get_option( 'permalink_structure' )` runs the option filters; a callback that throws answers [`PostError::Hook`].
pub trait RewriteContext {
    /// Whether rewrite arguments are resolved.
    fn rewrite_applies(&self) -> Result<bool, PostError>;
}

/// `sanitize_title_with_dashes()` for a string `query_var` (the text seam's rule; a PHP adapter in phase 4).
pub trait TitleSanitizer {
    /// The sanitized title.
    fn sanitize_title_with_dashes(&self, title: &[u8]) -> Vec<u8>;
}

/// `$wp_post_types`, read live by phase 4's PHP adapter. Owned values: the adapter converts PHP objects anyway.
pub trait PostTypes {
    /// One registered post type.
    fn post_type(&self, name: &str) -> Option<PostType>;
    /// Every registered post type, in registration order.
    fn post_types(&self) -> Vec<PostType>;
}

/// `$_wp_post_type_features`: post type, then feature, then `true` or the args array, in insertion order.
pub trait PostTypeSupports {
    /// `post_type_supports()`.
    fn supports(&self, post_type: &str, feature: &str) -> bool;
    /// Every post type's features.
    fn features(&self) -> Vec<(String, Vec<(String, PhpValue)>)>;
}

/// `$wp_post_statuses`.
///
/// Lookups take UTF-8 names. PHP keys statuses by bytes, so the adapter must decide how a non-UTF-8 name maps (no
/// stored status has one when every name went through `sanitize_key()`, which keeps only ASCII).
pub trait PostStatuses {
    /// One registered post status.
    fn status(&self, name: &str) -> Option<PostStatusObject>;
    /// Every registered post status, in registration order.
    fn statuses(&self) -> Vec<PostStatusObject>;
}
