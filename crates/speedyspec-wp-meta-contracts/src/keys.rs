//! The registered meta keys: the entries of `$wp_meta_keys` and the read port over them.

use speedyspec_php_serialize::semantics::{get, is_empty};
use speedyspec_php_serialize::{PhpKey, PhpValue};

use crate::MetaKey;

/// One entry of `$wp_meta_keys[ $object_type ][ $subtype ][ $key ]`: the parsed args, without `object_subtype`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegisteredMetaKey {
    pub object_type: String,
    pub subtype: String,
    pub key: MetaKey,
    pub args: Vec<(PhpKey, PhpValue)>,
}

impl RegisteredMetaKey {
    /// `$args['single']`, as PHP tests it for truth.
    pub fn single(&self) -> bool {
        get(&self.args, "single").is_some_and(|v| !is_empty(v))
    }

    /// `$args['default']` when the key was registered with one (`array_key_exists`, so a null default counts).
    pub fn default(&self) -> Option<&PhpValue> {
        get(&self.args, "default")
    }

    /// `$args[$name]`.
    pub fn arg(&self, name: &str) -> Option<&PhpValue> {
        get(&self.args, name)
    }
}

/// A read port over the registered keys (Registry): `get_registered_meta_keys()` and `get_registered_metadata()`
/// ask it; `register_meta()` fills it.
pub trait MetaKeys {
    /// `get_registered_meta_keys()`: every key registered for `object_type` and `subtype`.
    fn keys(&self, object_type: &str, subtype: &str) -> Vec<RegisteredMetaKey>;
    /// The entry for one key, `None` when it is not registered for `object_type` and `subtype`.
    fn find(&self, object_type: &str, subtype: &str, key: &MetaKey) -> Option<RegisteredMetaKey>;
}
