//! The metadata surface compiles, converts its errors and stays object-safe.

use std::rc::Rc;

use speedyspec_php_serialize::PhpValue;
use speedyspec_wp_hooks_contracts::HookError;
use speedyspec_wp_meta_contracts::{
    Installing, Meta, MetaId, MetaKey, MetaKeys, MetaType, Metadata, MetadataError, ObjectId,
    RegisteredMetaKey, UpdateOutcome,
};
use speedyspec_wp_object_cache_contracts::ObjectCacheError;

struct Dummy;

impl Metadata for Dummy {
    fn add(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: &MetaKey,
        _: PhpValue,
        _: bool,
    ) -> Result<Option<MetaId>, MetadataError> {
        Ok(Some(MetaId(1)))
    }
    fn update(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: &MetaKey,
        _: PhpValue,
        _: Option<PhpValue>,
    ) -> Result<UpdateOutcome, MetadataError> {
        Ok(UpdateOutcome::Added(MetaId(2)))
    }
    fn delete(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: &MetaKey,
        _: Option<PhpValue>,
        _: bool,
    ) -> Result<bool, MetadataError> {
        Ok(true)
    }
    fn get(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: Option<&MetaKey>,
        _: bool,
    ) -> Result<PhpValue, MetadataError> {
        Ok(PhpValue::Null)
    }
    fn get_raw(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: Option<&MetaKey>,
        _: bool,
    ) -> Result<Option<PhpValue>, MetadataError> {
        Ok(None)
    }
    fn get_default(
        &self,
        _: &MetaType,
        _: ObjectId,
        _: &MetaKey,
        _: bool,
    ) -> Result<PhpValue, MetadataError> {
        Ok(PhpValue::Null)
    }
    fn exists(&self, _: &MetaType, _: ObjectId, _: &MetaKey) -> Result<bool, MetadataError> {
        Ok(false)
    }
    fn get_by_mid(&self, _: &MetaType, _: MetaId) -> Result<Option<Meta>, MetadataError> {
        Ok(None)
    }
    fn update_by_mid(
        &self,
        _: &MetaType,
        _: MetaId,
        _: PhpValue,
        _: Option<&MetaKey>,
    ) -> Result<bool, MetadataError> {
        Ok(false)
    }
    fn delete_by_mid(&self, _: &MetaType, _: MetaId) -> Result<bool, MetadataError> {
        Ok(false)
    }
    fn update_cache(
        &self,
        _: &MetaType,
        _: &[ObjectId],
    ) -> Result<Option<PhpValue>, MetadataError> {
        Ok(None)
    }
}

impl MetaKeys for Dummy {
    fn keys(&self, _: &str, _: &str) -> Vec<RegisteredMetaKey> {
        Vec::new()
    }
    fn find(&self, object_type: &str, subtype: &str, key: &MetaKey) -> Option<RegisteredMetaKey> {
        Some(RegisteredMetaKey {
            object_type: object_type.to_string(),
            subtype: subtype.to_string(),
            key: key.clone(),
            args: Vec::new(),
        })
    }
}

impl Installing for Dummy {
    fn installing(&self) -> bool {
        false
    }
}

#[test]
fn the_traits_are_object_safe_and_callable() {
    let metadata: Rc<dyn Metadata> = Rc::new(Dummy);
    let keys: Rc<dyn MetaKeys> = Rc::new(Dummy);
    let installing: Rc<dyn Installing> = Rc::new(Dummy);
    let ty = MetaType::new("post").unwrap();
    let key = MetaKey::new("k");
    assert_eq!(
        metadata
            .update(&ty, ObjectId(1), &key, PhpValue::Null, None)
            .unwrap(),
        UpdateOutcome::Added(MetaId(2))
    );
    assert!(keys.find("post", "", &key).is_some());
    assert!(!installing.installing());
}

#[test]
fn errors_convert_into_their_variants() {
    assert!(matches!(
        MetadataError::from(HookError::message("boom")),
        MetadataError::Hook(_)
    ));
    assert!(matches!(
        MetadataError::from(ObjectCacheError::Unsupported("k".into())),
        MetadataError::Cache(_)
    ));
    assert!(matches!(
        MetadataError::from(speedyspec_wp_meta_contracts::MetaError::Db("x".into())),
        MetadataError::Db(_)
    ));
}

#[test]
fn a_registered_key_reads_its_args() {
    let k = MetaKeys::find(&Dummy, "post", "page", &MetaKey::new("k")).unwrap();
    assert!(!k.single());
    assert!(k.default().is_none());
}
