//! What the options API reports when it cannot do as WordPress does.

use std::fmt;

use speedyspec_data::CommitError;
use speedyspec_wp_hooks_contracts::{HookError, HookValue};
use speedyspec_wp_object_cache_contracts::ObjectCacheError;

/// A failure the options API reports to its caller.
#[derive(Debug)]
pub enum OptionsError {
    /// A hook callback failed; its payload (a PHP exception) is kept whole, to be rethrown.
    Hook(HookError),
    /// The object cache rejected a key or group.
    Cache(ObjectCacheError),
    /// The database reported a failure.
    Db(String),
    /// A value that cannot be stored as it is (not yet serialized).
    Serialize(String),
    /// `delete_option` of a protected option, whose message `wp_die()` shows.
    Protected(String),
    /// A step only PHP can take (a deprecated name's notice, an unported function), asked of a caller without PHP.
    NeedsPhp {
        /// The WordPress function.
        function: &'static str,
        /// The option it was called for.
        option: String,
    },
    /// A PHP exception (an opaque value) a callback threw, to be rethrown.
    Php(HookValue),
    /// A unit of work could not commit the write.
    Commit(CommitError),
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionsError::Hook(e) => write!(f, "a hook failed: {e}"),
            OptionsError::Cache(e) => write!(f, "the object cache failed: {e}"),
            OptionsError::Db(e) => write!(f, "the database failed: {e}"),
            OptionsError::Serialize(e) => write!(f, "a value cannot be stored: {e}"),
            OptionsError::Protected(e) => f.write_str(e),
            OptionsError::NeedsPhp { function, option } => {
                write!(f, "{function}() for the option {option} needs PHP")
            }
            OptionsError::Php(_) => f.write_str("a PHP callback threw an exception"),
            OptionsError::Commit(e) => write!(f, "the write could not be committed: {e}"),
        }
    }
}

impl std::error::Error for OptionsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OptionsError::Hook(e) => Some(e),
            OptionsError::Cache(e) => Some(e),
            OptionsError::Commit(e) => Some(e),
            OptionsError::Db(_)
            | OptionsError::Serialize(_)
            | OptionsError::Protected(_)
            | OptionsError::NeedsPhp { .. }
            | OptionsError::Php(_) => None,
        }
    }
}

impl From<HookError> for OptionsError {
    fn from(e: HookError) -> OptionsError {
        OptionsError::Hook(e)
    }
}

impl From<ObjectCacheError> for OptionsError {
    fn from(e: ObjectCacheError) -> OptionsError {
        OptionsError::Cache(e)
    }
}

impl From<CommitError> for OptionsError {
    fn from(e: CommitError) -> OptionsError {
        OptionsError::Commit(e)
    }
}
