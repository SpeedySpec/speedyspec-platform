//! The ports of the metadata API: the repository per meta type and the seams its owners bind.
//!
//! Every repository method is one `$wpdb` call `meta.php` makes, so query counts match. A failure `$wpdb` hides is
//! `Ok` with the empty answer; `Err` is only for failures it would not hide.

use std::rc::Rc;

use speedyspec_php_serialize::PhpValue;

use crate::{Meta, MetaError, MetaId, MetaKey, MetaRow, MetaType, ObjectId, SerializedValue};

/// One meta table (Repository per meta type). Rows are compared byte for byte; collation is the SQL adapter's.
pub trait MetaRepository {
    /// The name of the engine behind this repository (Strategy per engine).
    fn engine(&self) -> String;
    /// The meta type whose table this repository reads and writes.
    fn meta_type(&self) -> &MetaType;
    /// `add_metadata()`: `SELECT COUNT(*) ... WHERE meta_key = %s AND {object} = %d`.
    fn count_for(&self, object: ObjectId, key: &MetaKey) -> Result<u64, MetaError>;
    /// `add_metadata()`: `$wpdb->insert`; the new id, `None` when `$wpdb` returns false.
    fn add(
        &self,
        object: ObjectId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<Option<MetaId>, MetaError>;
    /// `update_metadata()`: `SELECT {id} ... WHERE meta_key = %s AND {object} = %d`; the ids come back in id order.
    fn ids_for(&self, object: ObjectId, key: &MetaKey) -> Result<Vec<MetaId>, MetaError>;
    /// `update_metadata()`: `wpdb::update` of `meta_value` `WHERE {object} = %d AND meta_key = %s [AND meta_value = %s]`.
    fn update_by_key(
        &self,
        object: ObjectId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
        previous: Option<&SerializedValue>,
    ) -> Result<u64, MetaError>;
    /// `delete_metadata()`: `SELECT {id} ... WHERE meta_key = %s [AND {object} = %d] [AND meta_value = %s]`.
    fn ids_matching(
        &self,
        key: &MetaKey,
        object: Option<ObjectId>,
        value: Option<&SerializedValue>,
    ) -> Result<Vec<MetaId>, MetaError>;
    /// `delete_metadata()` with `delete_all`: `SELECT {object} ... WHERE meta_key = %s [AND meta_value = %s]`.
    fn objects_with(
        &self,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<Vec<ObjectId>, MetaError>;
    /// `delete_metadata()`: `DELETE ... WHERE {id} IN (..)`; the rows deleted.
    fn delete_ids(&self, ids: &[MetaId]) -> Result<u64, MetaError>;
    /// `get_metadata_by_mid()`: `SELECT * ... WHERE {id} = %d`.
    fn by_mid(&self, mid: MetaId) -> Result<Option<Meta>, MetaError>;
    /// `update_metadata_by_mid()`: `wpdb::update` of `meta_value`, `meta_key` `WHERE {id} = %d`.
    fn update_by_mid(
        &self,
        mid: MetaId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<u64, MetaError>;
    /// `delete_metadata_by_mid()`: `wpdb::delete WHERE {id} = %d`.
    fn delete_by_mid(&self, mid: MetaId) -> Result<u64, MetaError>;
    /// `update_meta_cache()`'s query (meta.php 1268), with `{id_list}` the object ids joined with `,`:
    ///
    /// `SELECT $column, meta_key, meta_value FROM $table WHERE $column IN ($id_list) ORDER BY $id_column ASC`
    ///
    /// So for posts: `SELECT post_id, meta_key, meta_value FROM wp_postmeta WHERE post_id IN (1,2) ORDER BY meta_id
    /// ASC`. The rows come back in id order without the id column (hence [`MetaRow`], not [`Meta`]); an empty list
    /// runs no statement and is empty.
    fn for_objects(&self, objects: &[ObjectId]) -> Result<Vec<MetaRow>, MetaError>;
    /// The meta ids of one object, as the functions that delete an object read them before deleting its meta, with
    /// `get_col( prepare( .., $object_id ) )`. The statement is `SELECT {id} FROM {table} WHERE {object} = %d`,
    /// with the trailing space some callers leave before the closing quote; [`MetaType::object_ids_query`] builds
    /// it byte for byte:
    ///
    /// - `wp_delete_post()` (post.php 4129) and `wp_delete_attachment()` (post.php 7121):
    ///   `"SELECT meta_id FROM $wpdb->postmeta WHERE post_id = %d "`
    /// - `wp_delete_term()` (taxonomy.php 2272): `"SELECT meta_id FROM $wpdb->termmeta WHERE term_id = %d "`
    /// - `wp_delete_site()` (ms-site.php 264): `"SELECT meta_id FROM $wpdb->blogmeta WHERE blog_id = %d "`
    /// - `wp_delete_comment()` (comment.php 1634): `"SELECT meta_id FROM $wpdb->commentmeta WHERE comment_id = %d"`
    /// - `wp_delete_user()` (wp-admin/includes/user.php 444) and `wpmu_delete_user()` (wp-admin/includes/ms.php 201):
    ///   `"SELECT umeta_id FROM $wpdb->usermeta WHERE user_id = %d"`
    ///
    /// No `ORDER BY`, so the ids come back in the table's order.
    fn ids_of_object(&self, object: ObjectId) -> Result<Vec<MetaId>, MetaError>;
}

/// `_get_meta_table()` as a factory (Abstract Factory): `None` when the type has no table.
pub trait MetaRepositories {
    /// The repository for `meta_type`, shared by every caller; `None` when the type has no table.
    fn for_type(&self, meta_type: &MetaType) -> Option<Rc<dyn MetaRepository>>;
}

/// `get_object_subtype()`, bound per meta type by the seam that owns the objects (`""` when unknown).
pub trait ObjectSubtypes {
    fn subtype_of(&self, meta_type: &MetaType, object: ObjectId) -> String;
}

/// `post_type_supports($subtype, 'revisions')`, for `register_meta()`'s `revisions_enabled` check.
pub trait RevisionSupport {
    fn supports_revisions(&self, post_type: &str) -> bool;
}

/// `rest_validate_value_from_schema($default, $schema)` is not a `WP_Error`.
pub trait DefaultSchemaCheck {
    fn default_matches(&self, default: &PhpValue, schema: &PhpValue) -> bool;
}

/// `is_callable()`, for `register_meta()`'s callback arguments (a PHP adapter answers for real callables).
pub trait Callables {
    fn is_callable(&self, value: &PhpValue) -> bool;
}

/// `is_protected_meta()`'s answer, bound by the seam that applies the `is_protected_meta` filter.
pub trait ProtectedMeta {
    /// Whether `key` is protected for `meta_type` (the object type, as `register_meta()` passes it).
    fn is_protected(&self, key: &MetaKey, meta_type: &str) -> bool;
}

/// `wp_installing()`: whether WordPress is installing (some meta lookups skip the cache or the registry then).
pub trait Installing {
    fn installing(&self) -> bool;
}
