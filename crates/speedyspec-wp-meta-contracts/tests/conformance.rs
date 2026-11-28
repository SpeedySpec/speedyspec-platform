//! The conformance suites hold against a plain in-memory table, so the checks agree with one another.
//! Run with `--features conformance`.
#![cfg(feature = "conformance")]

use std::cell::RefCell;

use speedyspec_wp_meta_contracts::conformance::{binary_safe_suite, meta_repository_suite};
use speedyspec_wp_meta_contracts::{
    Meta, MetaError, MetaId, MetaKey, MetaRepository, MetaRow, MetaType, ObjectId, SerializedValue,
};

struct Table {
    ty: MetaType,
    rows: RefCell<Vec<Meta>>,
    next: RefCell<u64>,
}

impl Table {
    fn new() -> Table {
        Table {
            ty: MetaType::new("post").unwrap(),
            rows: RefCell::new(Vec::new()),
            next: RefCell::new(1),
        }
    }
}

impl MetaRepository for Table {
    fn engine(&self) -> String {
        "test".into()
    }
    fn meta_type(&self) -> &MetaType {
        &self.ty
    }
    fn count_for(&self, object: ObjectId, key: &MetaKey) -> Result<u64, MetaError> {
        let rows = self.rows.borrow();
        Ok(rows
            .iter()
            .filter(|m| m.object_id == object && &m.key == key)
            .count() as u64)
    }
    fn add(
        &self,
        object: ObjectId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<Option<MetaId>, MetaError> {
        let mut next = self.next.borrow_mut();
        let mid = MetaId(*next);
        *next += 1;
        self.rows.borrow_mut().push(Meta {
            mid,
            object_id: object,
            key: key.clone(),
            value: value.cloned(),
        });
        Ok(Some(mid))
    }
    fn ids_for(&self, object: ObjectId, key: &MetaKey) -> Result<Vec<MetaId>, MetaError> {
        self.ids_matching(key, Some(object), None)
    }
    fn update_by_key(
        &self,
        object: ObjectId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
        previous: Option<&SerializedValue>,
    ) -> Result<u64, MetaError> {
        let mut changed = 0;
        for m in self.rows.borrow_mut().iter_mut() {
            if m.object_id == object
                && &m.key == key
                && previous.is_none_or(|p| m.value.as_ref() == Some(p))
                && m.value.as_ref() != value
            {
                m.value = value.cloned();
                changed += 1;
            }
        }
        Ok(changed)
    }
    fn ids_matching(
        &self,
        key: &MetaKey,
        object: Option<ObjectId>,
        value: Option<&SerializedValue>,
    ) -> Result<Vec<MetaId>, MetaError> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|m| {
                &m.key == key
                    && object.is_none_or(|o| m.object_id == o)
                    && value.is_none_or(|v| m.value.as_ref() == Some(v))
            })
            .map(|m| m.mid)
            .collect())
    }
    fn objects_with(
        &self,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<Vec<ObjectId>, MetaError> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|m| &m.key == key && value.is_none_or(|v| m.value.as_ref() == Some(v)))
            .map(|m| m.object_id)
            .collect())
    }
    fn delete_ids(&self, ids: &[MetaId]) -> Result<u64, MetaError> {
        let mut rows = self.rows.borrow_mut();
        let before = rows.len();
        rows.retain(|m| !ids.contains(&m.mid));
        Ok((before - rows.len()) as u64)
    }
    fn by_mid(&self, mid: MetaId) -> Result<Option<Meta>, MetaError> {
        Ok(self.rows.borrow().iter().find(|m| m.mid == mid).cloned())
    }
    fn update_by_mid(
        &self,
        mid: MetaId,
        key: &MetaKey,
        value: Option<&SerializedValue>,
    ) -> Result<u64, MetaError> {
        let mut rows = self.rows.borrow_mut();
        let Some(m) = rows.iter_mut().find(|m| m.mid == mid) else {
            return Ok(0);
        };
        if &m.key == key && m.value.as_ref() == value {
            return Ok(0);
        }
        m.key = key.clone();
        m.value = value.cloned();
        Ok(1)
    }
    fn delete_by_mid(&self, mid: MetaId) -> Result<u64, MetaError> {
        self.delete_ids(&[mid])
    }
    fn for_objects(&self, objects: &[ObjectId]) -> Result<Vec<MetaRow>, MetaError> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|m| objects.contains(&m.object_id))
            .map(|m| MetaRow {
                object_id: m.object_id,
                key: m.key.clone(),
                value: m.value.clone(),
            })
            .collect())
    }
    fn ids_of_object(&self, object: ObjectId) -> Result<Vec<MetaId>, MetaError> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|m| m.object_id == object)
            .map(|m| m.mid)
            .collect())
    }
}

#[test]
fn an_in_memory_table_passes_the_repository_suite() {
    meta_repository_suite(&Table::new());
}

#[test]
fn an_in_memory_table_passes_the_binary_safe_suite() {
    binary_safe_suite(&Table::new());
}
