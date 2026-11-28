//! The row `wp_insert_post()` hands to `$wpdb->insert` and `$wpdb->update`: the `wp_insert_post_data`-filtered
//! `$data`, column by column, in its order.

use crate::{Post, PostParent};

/// The placeholder `wpdb::process_field_formats()` gives a column: `%d` for a name in `$wpdb->field_types`, `%s` for
/// any other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColumnFormat {
    /// `%d`: `prepare()` prints the value as PHP's `(int)` reads it.
    Int,
    /// `%s`: `prepare()` prints the value as PHP's `(string)` reads it, quoted and escaped.
    Str,
}

/// `$wpdb->field_types` as WordPress 7.2-RC2 sets it (load.php 751-787): the names that take `%d`. The match is exact,
/// case included, as `isset( $this->field_types[ $field ] )` is; a plugin that changes `$wpdb->field_types` is not
/// followed.
pub const INT_FIELDS: [&str; 34] = [
    "post_author",
    "post_parent",
    "menu_order",
    "term_id",
    "term_group",
    "term_taxonomy_id",
    "parent",
    "count",
    "object_id",
    "term_order",
    "ID",
    "comment_ID",
    "comment_post_ID",
    "comment_parent",
    "user_id",
    "link_id",
    "link_owner",
    "link_rating",
    "option_id",
    "blog_id",
    "meta_id",
    "post_id",
    "user_status",
    "umeta_id",
    "comment_karma",
    "comment_count",
    "active",
    "cat_id",
    "deleted",
    "lang_id",
    "mature",
    "public",
    "site_id",
    "spam",
];

/// The placeholder `wpdb` gives the column `name`.
pub fn column_format(name: &str) -> ColumnFormat {
    if INT_FIELDS.contains(&name) {
        ColumnFormat::Int
    } else {
        ColumnFormat::Str
    }
}

/// One value of a [`PostRow`], as `wpdb` prints it into the statement.
///
/// The use case converts the PHP value by its column's [`ColumnFormat`]: `(int)` for `%d` (`2.9` is `2`, `'4x'` is
/// `4`) and `(string)` for `%s` (`1.5` is `'1.5'`, `true` is `'1'`, `false` is `''`). `null` is [`RowValue::Null`]
/// whatever the format: `wpdb` writes `NULL` in its place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowValue {
    /// `null`: `NULL` in an `INSERT`'s `VALUES`, `` `column` = NULL `` in an `UPDATE`.
    Null,
    /// A `%d` value.
    Int(i64),
    /// A `%s` value, as bytes.
    Str(Vec<u8>),
}

impl RowValue {
    /// Whether the value may stand in a column of `format`: [`RowValue::Null`] in any, [`RowValue::Int`] under `%d`,
    /// [`RowValue::Str`] under `%s`.
    pub fn fits(&self, format: ColumnFormat) -> bool {
        matches!(
            (self, format),
            (RowValue::Null, _)
                | (RowValue::Int(_), ColumnFormat::Int)
                | (RowValue::Str(_), ColumnFormat::Str)
        )
    }
}

/// The `$data` array `wp_insert_post()` passes to `$wpdb->insert( $wpdb->posts, $data )` or `$wpdb->update(
/// $wpdb->posts, $data, $where )`, after the `wp_insert_post_data` (or `wp_insert_attachment_data`) filter and
/// `wp_unslash()`: its keys in order, each with its value.
///
/// The statement's columns are these keys, in this order. A column a callback dropped is not written (an `INSERT`
/// leaves it to the engine's default, an `UPDATE` keeps the stored value), and a key a callback added is written: a
/// name that is not a `wp_posts` column makes MySQL reject the statement ("Unknown column"), so `$wpdb` returns false.
/// An integer key is its decimal text, as PHP prints it; a key that is not UTF-8 is the binding's to map, and names no
/// column either way.
///
/// For an insert, the row includes `ID` when `wp_insert_post()` set `$data['ID']` from a free `import_id`: appended
/// after the filtered keys, or replacing an `ID` a callback added, in its place, as PHP's assignment does.
///
/// A filter answer that is not an array never becomes a row; the use case handles it before the repository, as
/// WordPress 7.2-RC2 does (checked by running it):
///
/// - an update: `$wpdb->update` returns false at its `is_array()` check without a statement, so `wp_insert_post()`
///   answers `db_update_error` (or `0`);
/// - an insert of `null` or `false` with a free `import_id`: `$data['ID'] = $import_id` makes it an array, and the row
///   is `ID` alone (``INSERT INTO `{posts}` (`ID`) VALUES (900)``);
/// - any other insert: `wpdb::_insert_replace_helper()` calls `array_keys()` on it, which throws a `TypeError`, and the
///   binding raises it (assigning `ID` on a non-empty string throws first).
///
/// Every value must fit its column's format ([`RowValue::fits`]); a repository answers [`crate::PostError::Db`] for
/// one that does not, without a statement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostRow {
    columns: Vec<(String, RowValue)>,
}

impl PostRow {
    /// An empty row. WordPress sends ``INSERT INTO `{posts}` (``) VALUES ()`` or ``UPDATE `{posts}` SET  WHERE `ID` =
    /// %d`` for it, which MySQL rejects, so `$wpdb` returns false.
    pub fn new() -> Self {
        Self::default()
    }

    /// The columns `wp_insert_post()`'s `compact()` builds, in its order, when no callback changed them: the 21
    /// columns of [`Post`] but `ID` and `comment_count`. A [`PostParent::Null`] parent is [`RowValue::Null`].
    pub fn from_post(post: &Post) -> Self {
        let text = |bytes: &[u8]| RowValue::Str(bytes.to_vec());
        let parent = match post.parent {
            PostParent::Int(n) => RowValue::Int(n),
            PostParent::Null => RowValue::Null,
        };
        let author = RowValue::Int(i64::try_from(post.author).unwrap_or(i64::MAX));
        let mut row = Self::new();
        for (name, value) in [
            ("post_author", author),
            ("post_date", text(post.date.local.as_str().as_bytes())),
            ("post_date_gmt", text(post.date.gmt.as_str().as_bytes())),
            ("post_content", text(&post.content)),
            ("post_content_filtered", text(&post.content_filtered)),
            ("post_title", text(&post.title)),
            ("post_excerpt", text(&post.excerpt)),
            ("post_status", text(post.status.as_str().as_bytes())),
            ("post_type", text(post.post_type.as_str().as_bytes())),
            (
                "comment_status",
                text(post.comment_status.as_str().as_bytes()),
            ),
            ("ping_status", text(post.ping_status.as_str().as_bytes())),
            ("post_password", text(&post.password)),
            ("post_name", text(post.name.as_bytes())),
            ("to_ping", text(&post.to_ping)),
            ("pinged", text(&post.pinged)),
            (
                "post_modified",
                text(post.modified.local.as_str().as_bytes()),
            ),
            (
                "post_modified_gmt",
                text(post.modified.gmt.as_str().as_bytes()),
            ),
            ("post_parent", parent),
            ("menu_order", RowValue::Int(post.menu_order)),
            ("post_mime_type", text(post.mime_type.as_bytes())),
            ("guid", text(&post.guid)),
        ] {
            row.set(name, value);
        }
        row
    }

    /// Sets a column as PHP's `$data[ $name ] = $value` does: in place when the key is there, appended otherwise.
    pub fn set(&mut self, name: impl Into<String>, value: RowValue) {
        let name = name.into();
        match self.columns.iter_mut().find(|(n, _)| *n == name) {
            Some((_, slot)) => *slot = value,
            None => self.columns.push((name, value)),
        }
    }

    /// The same row with a column set, as [`PostRow::set`].
    pub fn with(mut self, name: impl Into<String>, value: RowValue) -> Self {
        self.set(name, value);
        self
    }

    /// Drops a column, as `unset( $data[ $name ] )` does.
    pub fn remove(&mut self, name: &str) {
        self.columns.retain(|(n, _)| n != name);
    }

    /// One column's value.
    pub fn get(&self, name: &str) -> Option<&RowValue> {
        self.columns.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// The columns, in order.
    pub fn columns(&self) -> &[(String, RowValue)] {
        &self.columns
    }

    /// Whether the row has no column.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}
