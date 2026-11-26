//! What the metadata API reports when it cannot do as WordPress does.

use std::fmt;

use speedyspec_wp_hooks_contracts::HookError;
use speedyspec_wp_object_cache_contracts::ObjectCacheError;

/// A failure the metadata API cannot hide, as `$wpdb` hides none of these.
#[derive(Debug)]
pub enum MetaError {
    /// The database reported a failure.
    Db(String),
}

impl fmt::Display for MetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaError::Db(e) => write!(f, "the database failed: {e}"),
        }
    }
}

impl std::error::Error for MetaError {}

/// A failure the metadata API reports to its caller (a use case, not a repository).
#[derive(Debug)]
pub enum MetadataError {
    /// A hook callback failed, or a hook argument could not be used.
    Hook(HookError),
    /// The object cache rejected a key or group.
    Cache(ObjectCacheError),
    /// The database reported a failure.
    Db(String),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::Hook(e) => write!(f, "a hook failed: {e}"),
            MetadataError::Cache(e) => write!(f, "the object cache failed: {e}"),
            MetadataError::Db(e) => write!(f, "the database failed: {e}"),
        }
    }
}

impl std::error::Error for MetadataError {}

impl From<HookError> for MetadataError {
    fn from(e: HookError) -> Self {
        MetadataError::Hook(e)
    }
}

impl From<ObjectCacheError> for MetadataError {
    fn from(e: ObjectCacheError) -> Self {
        MetadataError::Cache(e)
    }
}

impl From<MetaError> for MetadataError {
    fn from(e: MetaError) -> Self {
        match e {
            MetaError::Db(m) => MetadataError::Db(m),
        }
    }
}
