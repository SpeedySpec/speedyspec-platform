//! `Provider` and `compose`: every provider registers before any area is built; unavailable seams are left out.

use std::rc::Rc;

use speedyspec_container::{Container, Resolved};
use speedyspec_container_php::{ContainerArea, Provider, ProviderError, compose};
use speedyspec_php::NativeArea;

struct Area;
impl NativeArea for Area {}

/// Registers `"a.value"`.
struct A;
impl Provider for A {
    fn seam(&self) -> &'static str {
        "a"
    }
    fn register(&self, container: &Container) {
        container.singleton("a.value", |_, _| Ok(Rc::new(41_i64) as Resolved));
    }
    fn area(&self, _: &Rc<Container>) -> Result<Rc<dyn NativeArea>, ProviderError> {
        Ok(Rc::new(Area))
    }
}

/// Its area needs `"a.value"`.
struct B;
impl Provider for B {
    fn seam(&self) -> &'static str {
        "b"
    }
    fn register(&self, _: &Container) {}
    fn area(&self, container: &Rc<Container>) -> Result<Rc<dyn NativeArea>, ProviderError> {
        let value = container
            .make("a.value")
            .map_err(|err| ProviderError::Failed {
                seam: "b",
                reason: err.to_string(),
            })?;
        assert_eq!(value.downcast_ref::<i64>(), Some(&41));
        Ok(Rc::new(Area))
    }
}

struct Missing;
impl Provider for Missing {
    fn seam(&self) -> &'static str {
        "missing"
    }
    fn register(&self, _: &Container) {}
    fn area(&self, _: &Rc<Container>) -> Result<Rc<dyn NativeArea>, ProviderError> {
        Err(ProviderError::Unavailable {
            seam: "missing",
            reason: "no database credentials".into(),
        })
    }
}

struct Broken;
impl Provider for Broken {
    fn seam(&self) -> &'static str {
        "broken"
    }
    fn register(&self, _: &Container) {}
    fn area(&self, _: &Rc<Container>) -> Result<Rc<dyn NativeArea>, ProviderError> {
        Err(ProviderError::Failed {
            seam: "broken",
            reason: "bad config".into(),
        })
    }
}

fn setup() -> (Rc<Container>, ContainerArea) {
    let container = Rc::new(Container::new());
    let area = ContainerArea::new("demo", container.clone());
    (container, area)
}

#[test]
fn providers_register_before_any_area_is_built() {
    let (container, area) = setup();
    let providers: Vec<Rc<dyn Provider>> = vec![Rc::new(B), Rc::new(A)];
    let (_, areas, unavailable) = compose(&container, area, &providers).expect("composes");
    assert_eq!(areas.len(), 2);
    assert!(unavailable.is_empty());
}

#[test]
fn unavailable_seam_is_left_out_and_reported() {
    let (container, area) = setup();
    let providers: Vec<Rc<dyn Provider>> = vec![Rc::new(A), Rc::new(Missing)];
    let (_, areas, unavailable) = compose(&container, area, &providers).expect("composes");
    assert_eq!(areas.len(), 1);
    assert_eq!(unavailable.len(), 1);
    assert_eq!(
        unavailable[0].to_string(),
        "missing is unavailable: no database credentials"
    );
}

#[test]
fn failed_seam_aborts_composition() {
    let (container, area) = setup();
    let providers: Vec<Rc<dyn Provider>> = vec![Rc::new(A), Rc::new(Broken)];
    let err = compose(&container, area, &providers).err().expect("fails");
    assert_eq!(err.to_string(), "broken failed to start: bad config");
}
