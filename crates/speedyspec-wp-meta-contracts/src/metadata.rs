//! The metadata API as one contract: a method per function of WordPress's `meta.php`.
//!
//! Command/CQRS: each method is one handler. Template Method: the use cases are generic over the [`MetaType`], which
//! supplies the table, column and hook names. Observer: implementations run WordPress's hooks through the hook API.

use speedyspec_php_serialize::PhpValue;

use crate::{Meta, MetaId, MetaKey, MetaType, MetadataError, ObjectId};

/// What `update_metadata()` returns: `true`, the new meta id, or `false`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpdateOutcome {
    /// `true`: the value of an existing row changed.
    Updated,
    /// The key had no row, so the call added one: WordPress returns the new id.
    Added(MetaId),
    /// `false`: nothing changed (the value was the same, or a filter short-circuited with false).
    Unchanged,
    /// `false`: the update, or the add it fell back to, failed.
    Failed,
}

/// The metadata API (`add_metadata()` and the functions beside it).
///
/// Every method takes the meta type first. Values are PHP values: the implementation serializes them as
/// `maybe_serialize()` does and unserializes what it reads.
pub trait Metadata {
    /// `add_metadata()`: the new meta id, `None` when WordPress returns `false` (a filter, a unique key that
    /// exists, or a failed insert).
    fn add(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: &MetaKey,
        value: PhpValue,
        unique: bool,
    ) -> Result<Option<MetaId>, MetadataError>;

    /// `update_metadata()`; `prev` is `$prev_value`, the value the row must have.
    fn update(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: &MetaKey,
        value: PhpValue,
        prev: Option<PhpValue>,
    ) -> Result<UpdateOutcome, MetadataError>;

    /// `delete_metadata()`: whether a row was deleted. `value` is `$meta_value`; `delete_all` deletes the key
    /// from every object.
    fn delete(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: &MetaKey,
        value: Option<PhpValue>,
        delete_all: bool,
    ) -> Result<bool, MetadataError>;

    /// `get_metadata()`: with no key, every key's values; with a key, the value (`single`) or the list. A missing
    /// key gives `''` for `single`, an empty array otherwise, or the registered default.
    fn get(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: Option<&MetaKey>,
        single: bool,
    ) -> Result<PhpValue, MetadataError>;

    /// `get_metadata_raw()`: as `get()` but with no default. `None` where WordPress returns `null` (nothing stored,
    /// and no short-circuit answer); `Some(PhpValue::Bool(false))` where it returns `false` (an empty type or a
    /// non-positive object id).
    fn get_raw(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: Option<&MetaKey>,
        single: bool,
    ) -> Result<Option<PhpValue>, MetadataError>;

    /// `get_metadata_default()`: the registered default for the key, or the empty value `get()` falls back to.
    fn get_default(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: &MetaKey,
        single: bool,
    ) -> Result<PhpValue, MetadataError>;

    /// `metadata_exists()`: whether the object has the key.
    fn exists(
        &self,
        meta_type: &MetaType,
        object: ObjectId,
        key: &MetaKey,
    ) -> Result<bool, MetadataError>;

    /// `get_metadata_by_mid()`: the row, `None` when there is none.
    fn get_by_mid(&self, meta_type: &MetaType, mid: MetaId) -> Result<Option<Meta>, MetadataError>;

    /// `update_metadata_by_mid()`: `key` is `$meta_key`, which renames the row when given.
    fn update_by_mid(
        &self,
        meta_type: &MetaType,
        mid: MetaId,
        value: PhpValue,
        key: Option<&MetaKey>,
    ) -> Result<bool, MetadataError>;

    /// `delete_metadata_by_mid()`.
    fn delete_by_mid(&self, meta_type: &MetaType, mid: MetaId) -> Result<bool, MetadataError>;

    /// `update_meta_cache()`: fills the cache for `objects`; the cache array WordPress returns, `None` when it
    /// returns `false` (an unknown meta type, or no valid object ids).
    fn update_cache(
        &self,
        meta_type: &MetaType,
        objects: &[ObjectId],
    ) -> Result<Option<PhpValue>, MetadataError>;
}
