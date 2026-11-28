//! The meta rows as plain values that know nothing of how they are stored.

use crate::{MetaId, MetaKey, ObjectId, SerializedValue};

/// One row of a meta table.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Meta {
    pub mid: MetaId,
    pub object_id: ObjectId,
    pub key: MetaKey,
    /// `None` is SQL `NULL`: the column is nullable and `update_metadata_by_mid()` can write it.
    pub value: Option<SerializedValue>,
}

/// One row as `update_meta_cache()` reads it: the object, key and value, without the meta id, because WordPress's
/// cache-priming statement does not select the id column.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MetaRow {
    pub object_id: ObjectId,
    pub key: MetaKey,
    /// `None` is SQL `NULL`.
    pub value: Option<SerializedValue>,
}
