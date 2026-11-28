//! The row `wp_insert_post()` hands to `$wpdb->insert` and `$wpdb->update`: the `wp_insert_post_data`-filtered
//! `$data`, column by column, in its order.

use speedyspec_wp_hooks_contracts::HookValue;

use crate::{Post, PostParent};

/// The placeholder `wpdb::process_field_formats()` gives a column: `%d` for a name in `$wpdb->field_types`, `%s` for
/// any other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColumnFormat {
    /// `%d`: `prepare()` prints the value as PHP's `(int)` reads it.
    Int,
    /// `%s`: `prepare()` prints the value as PHP's `(string)` reads it, quoted and escaped.
    Str,
}

/// `$wpdb->field_types` as WordPress 7.2-RC2 sets it (load.php 751-787): the names that take `%d`. The match is exact,
/// case included, as `isset( $this->field_types[ $field ] )` is.
///
/// A known limitation: a plugin that changes `$wpdb->field_types` at runtime is not followed.
pub const INT_FIELDS: &[&str] = &[
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

/// The placeholder `wpdb` gives the column `name` (its bytes, as PHP keys are).
pub fn column_format(name: impl AsRef<[u8]>) -> ColumnFormat {
    let name = name.as_ref();
    if INT_FIELDS.iter().any(|f| f.as_bytes() == name) {
        ColumnFormat::Int
    } else {
        ColumnFormat::Str
    }
}

/// One value of a [`PostRow`]: the PHP value, by its PHP type, which decides how `wpdb::prepare()` prints it.
///
/// The use case does not convert a value to its column's format: `wpdb` formats it, and its length check
/// (`strip_invalid_text()`) looks only at PHP strings, so a non-string under `%s` is never length-checked. Each
/// variant, under each [`ColumnFormat`]:
///
/// - [`RowValue::Null`]: `NULL` in place of the placeholder, whatever the format.
/// - [`RowValue::Int`]: not escaped; `%d` prints it, `%s` prints its decimal text, quoted.
/// - [`RowValue::Str`]: escaped by `mysqli_real_escape_string()` first, then printed: `%s` quotes the escaped bytes,
///   and `%d` reads the escaped text as PHP's `sprintf( '%d' )` does. So `'4x'` is `4`, but `"\n5"` is `0`, not the
///   `5` PHP's `(int)` gives, because the newline is escaped to `\n` first (a leading `\r` the same). It is the only
///   variant `strip_invalid_text()` checks, under `%s`: invalid text for the column's charset, or a value longer than
///   the column, makes `$wpdb` return false before any statement.
/// - [`RowValue::Php`] with a float: not escaped; `%d` truncates it (`2.9` is `2`), `%s` prints it as `(string)` does,
///   with PHP's `precision` of 14 (`0.1 + 0.2` is `'0.3'`, `-1.2345678901234567e308` is `'-1.2345678901235E+308'`).
/// - [`RowValue::Php`] with a bool: escaped as a string first, so `true` is `'1'` and `false` is `''` under `%s`, and
///   `1` and `0` under `%d`.
/// - [`RowValue::Php`] with anything else (an array, an object): `prepare()` calls `_doing_it_wrong( 'wpdb::prepare',
///   'Unsupported value type (array).', '4.8.2' )`, which runs the `doing_it_wrong_run` action and the
///   `doing_it_wrong_trigger_error` filter, then prints `''` (`0` under `%d`). That happens after `process_fields()`
///   has passed, just before the statement is sent.
///
/// [`RowValue::Php`] never holds a value [`RowValue::Null`], [`RowValue::Int`] or [`RowValue::Str`] expresses: a hook
/// value's `Null`, `Int`, `Str` and `Bytes` (bytes are a PHP string too), and an `Opaque` the bridge reads as one of
/// those, belong in those variants. Build a value from a hook value with [`RowValue::from_hook`], which normalises
/// them, so two repositories never print the same PHP string differently.
///
/// It is `#[non_exhaustive]`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum RowValue {
    /// `null`.
    Null,
    /// A PHP int.
    Int(i64),
    /// A PHP string, as bytes.
    Str(Vec<u8>),
    /// Any other PHP value (a float, a bool, an array, an object), as the hooks carry it; never one of the above.
    Php(HookValue),
}

impl RowValue {
    /// The row value for a PHP value as the hooks carry it: `Null`, `Int`, `Str` and `Bytes` become
    /// [`RowValue::Null`], [`RowValue::Int`] and [`RowValue::Str`]; an `Opaque` is read through the bridge
    /// ([`HookValue::read`]) and normalised the same way when it reads as one of those (a PHP string passed through a
    /// hook untouched). Anything else (a float, a bool, an array, an object the bridge keeps opaque) is
    /// [`RowValue::Php`], as given.
    pub fn from_hook(value: HookValue) -> RowValue {
        fn scalar(value: &HookValue) -> Option<RowValue> {
            match value {
                HookValue::Null => Some(RowValue::Null),
                HookValue::Int(n) => Some(RowValue::Int(*n)),
                HookValue::Str(s) => Some(RowValue::Str(s.as_bytes().to_vec())),
                HookValue::Bytes(b) => Some(RowValue::Str(b.clone())),
                _ => None,
            }
        }
        if let Some(row) = scalar(&value) {
            return row;
        }
        if let HookValue::Opaque(_) = value
            && let Some(row) = scalar(value.read().as_ref())
        {
            return row;
        }
        RowValue::Php(value)
    }
}

/// The `$data` array `wp_insert_post()` passes to `$wpdb->insert( $wpdb->posts, $data )` or `$wpdb->update(
/// $wpdb->posts, $data, $where )`, after the `wp_insert_post_data` (or `wp_insert_attachment_data`) filter and
/// `wp_unslash()`: its keys in order, each with its value.
///
/// The statement's columns are these keys, in this order. A column a callback dropped is not written (an `INSERT`
/// leaves it to the engine's default, an `UPDATE` keeps the stored value), and a key a callback added is written: a
/// name that is not a `wp_posts` column makes MySQL reject the statement ("Unknown column"), so `$wpdb` returns false.
/// Keys are bytes, as PHP's are, so a key that is not UTF-8 reaches the statement (and the `query` filter) exactly; an
/// integer key is its decimal text, as PHP prints it.
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
///   binding raises it (assigning `ID` on a string, `''` included, throws first: "Cannot access offset of type
///   string on string").
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PostRow {
    columns: Vec<(Vec<u8>, RowValue)>,
}

impl PostRow {
    /// An empty row. WordPress sends ``INSERT INTO `{posts}` (``) VALUES ()`` (after `prepare()`'s `_doing_it_wrong`,
    /// since it holds no placeholder) or ``UPDATE `{posts}` SET  WHERE `ID` = %d`` for it, which MySQL rejects, so
    /// `$wpdb` returns false.
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
    pub fn set(&mut self, name: impl AsRef<[u8]>, value: RowValue) {
        let name = name.as_ref();
        match self.columns.iter_mut().find(|(n, _)| n == name) {
            Some((_, slot)) => *slot = value,
            None => self.columns.push((name.to_vec(), value)),
        }
    }

    /// The same row with a column set, as [`PostRow::set`].
    pub fn with(mut self, name: impl AsRef<[u8]>, value: RowValue) -> Self {
        self.set(name, value);
        self
    }

    /// Drops a column, as `unset( $data[ $name ] )` does.
    pub fn remove(&mut self, name: impl AsRef<[u8]>) {
        let name = name.as_ref();
        self.columns.retain(|(n, _)| n != name);
    }

    /// One column's value.
    pub fn get(&self, name: impl AsRef<[u8]>) -> Option<&RowValue> {
        let name = name.as_ref();
        self.columns.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// The columns, in order: each key's bytes and its value.
    pub fn columns(&self) -> &[(Vec<u8>, RowValue)] {
        &self.columns
    }

    /// Whether the row has no column.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

/// What `$wpdb->insert` or `$wpdb->update` answered: the insert id or the rows changed, or false with the
/// `$wpdb->last_error` it left, which `wp_insert_post()` puts in its `db_insert_error` or `db_update_error` as the
/// data (post.php 5244, 5276).
///
/// It is `#[non_exhaustive]`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteOutcome<T> {
    /// The write ran: the new id (`$wpdb->insert_id`), or the rows changed.
    Written(T),
    /// `$wpdb` returned false.
    Failed {
        /// `$wpdb->last_error` afterwards:
        ///
        /// - the engine's message for a statement it rejected (MySQL's "Duplicate entry '100' for key
        ///   'wp_posts.PRIMARY'", "Unknown column 'bogus' in 'field list'", "Column 'post_parent' cannot be null");
        /// - `process_fields()`' message when a value failed `strip_invalid_text()` and no statement was sent
        ///   ("WordPress database error: Processing the value for the following field failed: post_title. The
        ///   supplied value may be too long or contains invalid data.", through `__()`, or the plural form for
        ///   several fields);
        /// - whatever it held before when the charset lookup failed, since no query ran to clear it.
        last_error: String,
    },
}

impl<T> WriteOutcome<T> {
    /// A failed write with this `$wpdb->last_error`.
    pub fn failed(last_error: impl Into<String>) -> Self {
        WriteOutcome::Failed {
            last_error: last_error.into(),
        }
    }

    /// The id or count, `None` for a failed write.
    pub fn written(self) -> Option<T> {
        match self {
            WriteOutcome::Written(value) => Some(value),
            WriteOutcome::Failed { .. } => None,
        }
    }

    /// `$wpdb->last_error` for a failed write, `None` for one that ran.
    pub fn last_error(&self) -> Option<&str> {
        match self {
            WriteOutcome::Written(_) => None,
            WriteOutcome::Failed { last_error } => Some(last_error),
        }
    }
}
