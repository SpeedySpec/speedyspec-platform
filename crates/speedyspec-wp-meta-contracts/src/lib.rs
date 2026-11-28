#![doc = include_str!("../README.md")]
//!
//! ## In detail
//!
//! WordPress's metadata API contracts: the meta values, the entity and its errors.

#[cfg(feature = "conformance")]
pub mod conformance;
mod entity;
mod error;
mod keys;
mod metadata;
mod ports;
mod values;

pub use entity::{Meta, MetaRow};
pub use error::{MetaError, MetadataError};
pub use keys::{MetaKeys, RegisteredMetaKey};
pub use metadata::{Metadata, UpdateOutcome};
pub use ports::{
    Callables, DefaultSchemaCheck, Installing, MetaRepositories, MetaRepository, ObjectSubtypes,
    ProtectedMeta, RevisionSupport,
};
pub use values::{MetaId, MetaKey, MetaType, ObjectId, SerializedValue};
