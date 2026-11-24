//! Service providers and the composition root's helper.
//!
//! Each seam's `-php` crate ships a [`Provider`]: it binds the seam's implementations under its contracts' keys and
//! builds the seam's native area from proxies. The runtime lists providers and calls [`compose`]; it is the only
//! place that names implementations of more than one seam.

use std::rc::Rc;

use speedyspec_container::Container;
use speedyspec_php::NativeArea;

use crate::ContainerArea;

/// One seam's registrations and native area.
pub trait Provider {
    /// The seam id (the guide's), for logs.
    fn seam(&self) -> &'static str;

    /// Binds this seam's implementations under its contracts' keys, with worker lifetime. Binds only, never
    /// resolves, so the order providers run in does not matter.
    fn register(&self, container: &Container);

    /// Keys of this seam's stateful ports (ADR-017): PHP may bind them only before they are used in a request.
    fn stateful(&self) -> &'static [&'static str] {
        &[]
    }

    /// This seam's native area, built from proxies, after every provider has registered.
    fn area(&self, container: &Rc<Container>) -> Result<Rc<dyn NativeArea>, ProviderError>;
}

/// Why a provider has no area.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    /// The seam cannot run in this configuration (for example, no database credentials); it is left out.
    #[error("{seam} is unavailable: {reason}")]
    Unavailable { seam: &'static str, reason: String },
    /// The seam is configured but cannot start; the worker does not start either.
    #[error("{seam} failed to start: {reason}")]
    Failed { seam: &'static str, reason: String },
}

/// Registers every provider, marks their stateful keys on `area`, then builds their areas in the order given. An
/// [`ProviderError::Unavailable`] seam is left out and returned in the third element; a
/// [`ProviderError::Failed`] one aborts.
#[allow(clippy::type_complexity)]
pub fn compose(
    container: &Rc<Container>,
    mut area: ContainerArea,
    providers: &[Rc<dyn Provider>],
) -> Result<(ContainerArea, Vec<Rc<dyn NativeArea>>, Vec<ProviderError>), ProviderError> {
    for provider in providers {
        provider.register(container);
        for key in provider.stateful() {
            area = area.stateful(key);
        }
    }
    let mut areas = Vec::new();
    let mut unavailable = Vec::new();
    for provider in providers {
        match provider.area(container) {
            Ok(built) => areas.push(built),
            Err(err @ ProviderError::Unavailable { .. }) => unavailable.push(err),
            Err(err @ ProviderError::Failed { .. }) => return Err(err),
        }
    }
    Ok((area, areas, unavailable))
}
