//! [`PortCell`]: the core of a generated port proxy.
//!
//! A seam reaches another seam's port through a proxy that forwards to whatever the container binds under the
//! port's key now. The cell caches the resolved port against the container's [`Container::epoch`], so a call with
//! nothing rebound costs one compare. Any binding change, and every request start and end, moves the epoch. The
//! next call then resolves through the container again, which serves a shared or scoped value from its own cache
//! and records the key as resolved in this request (so ADR-017's refusal to rebind a used stateful port holds). A
//! transient binding is never cached: it builds on every call, as its meaning requires.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::{Container, ContainerError, Sharing};

struct Cached<P: ?Sized> {
    epoch: u64,
    port: Rc<P>,
}

/// A handle on the port `P` bound under a key. Holds the container weakly, so a container that owns, through an
/// area or a handler, a value holding this cell is not kept alive by it.
pub struct PortCell<P: ?Sized + 'static> {
    container: Weak<Container>,
    epoch: Rc<Cell<u64>>,
    key: &'static str,
    cache: RefCell<Option<Cached<P>>>,
}

impl<P: ?Sized + 'static> PortCell<P> {
    pub fn new(container: &Rc<Container>, key: &'static str) -> PortCell<P> {
        PortCell {
            container: Rc::downgrade(container),
            epoch: container.epoch_cell(),
            key,
            cache: RefCell::new(None),
        }
    }

    /// The port's key.
    pub fn key(&self) -> &'static str {
        self.key
    }

    /// What the container binds under the key now. Panics, naming the key, when the container is gone or the key
    /// cannot be resolved: a provider binds every port its seam owns, so either is a configuration error.
    pub fn get(&self) -> Rc<P> {
        if self.container.strong_count() == 0 {
            panic!("port {}: the container was dropped", self.key);
        }
        match self.try_get() {
            Ok(port) => port,
            Err(err) => panic!("port {}: {err}", self.key),
        }
    }

    /// Calls `f` with what the container binds under the key now; the hot path of a generated proxy. With nothing
    /// rebound since the last call it reads one counter and borrows the cache: no reference counting. Panics as
    /// [`PortCell::get`] does.
    pub fn with<R>(&self, f: impl FnOnce(&P) -> R) -> R {
        {
            let cache = self.cache.borrow();
            if let Some(hit) = cache.as_ref()
                && hit.epoch == self.epoch.get()
            {
                return f(&hit.port);
            }
        }
        let port = self.get();
        f(&port)
    }

    /// As [`PortCell::get`], with the failure returned instead.
    pub fn try_get(&self) -> Result<Rc<P>, ContainerError> {
        let Some(container) = self.container.upgrade() else {
            return Err(ContainerError::Unresolvable(
                self.key.to_string(),
                "the container was dropped".to_string(),
            ));
        };
        let epoch = self.epoch.get();
        if let Some(hit) = self.cache.borrow().as_ref()
            && hit.epoch == epoch
        {
            return Ok(hit.port.clone());
        }
        let port = container.make_port::<P>(self.key)?;
        let cached = match container.sharing(self.key) {
            Some(Sharing::Transient) | None => None,
            Some(_) => Some(Cached {
                // Resolving never moves the epoch, so the one read before resolving still holds.
                epoch,
                port: port.clone(),
            }),
        };
        // A re-entrant call (a port method that calls back into the same port while a binding changed) finds the
        // cache borrowed by the outer call: it answers without caching, and the outer call's next use refreshes it.
        if let Ok(mut cache) = self.cache.try_borrow_mut() {
            *cache = cached;
        }
        Ok(port)
    }
}
