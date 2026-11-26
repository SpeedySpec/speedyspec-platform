//! `PortCell`: a port handle that re-resolves only when its binding may have changed.

use std::cell::Cell;
use std::rc::Rc;

use speedyspec_container::{Container, Lifetime, PortCell};

trait Greeter {
    fn greet(&self) -> String;
}

struct Fixed(&'static str);

impl Greeter for Fixed {
    fn greet(&self) -> String {
        self.0.to_string()
    }
}

const KEY: &str = "Test\\Greeter";

fn fixed(word: &'static str) -> Rc<dyn Greeter> {
    Rc::new(Fixed(word))
}

/// Binds `KEY` as a singleton answering `word`, counting how many times its factory ran.
fn singleton(c: &Container, key: &str, word: &'static str) -> Rc<Cell<u32>> {
    let built = Rc::new(Cell::new(0));
    let count = built.clone();
    c.singleton_port::<dyn Greeter>(key, move |_, _| {
        count.set(count.get() + 1);
        Ok(fixed(word))
    });
    built
}

#[test]
fn resolves_lazily_and_caches_a_singleton() {
    let c = Rc::new(Container::new());
    let built = singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    assert_eq!(built.get(), 0, "nothing is resolved before the first call");
    assert_eq!(cell.get().greet(), "a");
    assert_eq!(cell.get().greet(), "a");
    assert_eq!(built.get(), 1);
}

#[test]
fn follows_a_rebind() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    assert_eq!(cell.get().greet(), "a");
    singleton(&c, KEY, "b");
    assert_eq!(cell.get().greet(), "b");
}

#[test]
fn follows_an_alias_retarget() {
    let c = Rc::new(Container::new());
    singleton(&c, "impl.a", "a");
    singleton(&c, "impl.b", "b");
    c.alias("impl.a", KEY);
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    assert_eq!(cell.get().greet(), "a");
    c.alias("impl.b", KEY);
    assert_eq!(cell.get().greet(), "b");
}

#[test]
fn transient_builds_every_call() {
    let c = Rc::new(Container::new());
    let built = Rc::new(Cell::new(0));
    let count = built.clone();
    c.bind_port::<dyn Greeter>(KEY, move |_, _| {
        count.set(count.get() + 1);
        Ok(fixed("t"))
    });
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    for _ in 0..3 {
        assert_eq!(cell.get().greet(), "t");
    }
    assert_eq!(built.get(), 3);
}

#[test]
fn scoped_is_fresh_each_request() {
    let c = Rc::new(Container::new());
    let built = Rc::new(Cell::new(0));
    let count = built.clone();
    c.scoped(KEY, move |_, _| {
        count.set(count.get() + 1);
        Ok(Rc::new(fixed("s")) as speedyspec_container::Resolved)
    });
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    c.start_request();
    cell.get();
    cell.get();
    c.end_request();
    c.start_request();
    cell.get();
    assert_eq!(built.get(), 2);
}

#[test]
fn first_use_in_each_request_marks_the_key_resolved() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    c.start_request();
    cell.get();
    c.end_request();
    c.start_request();
    assert!(!c.resolved_in_request(KEY));
    cell.get();
    assert!(c.resolved_in_request(KEY));
}

#[test]
fn request_bindings_revert_at_end_request() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    c.start_request();
    {
        let _request = c.registering(Lifetime::Request);
        singleton(&c, KEY, "b");
    }
    assert_eq!(cell.get().greet(), "b");
    c.end_request();
    assert_eq!(cell.get().greet(), "a");
}

#[test]
fn try_get_reports_an_unbound_key() {
    let c = Rc::new(Container::new());
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    assert!(cell.try_get().is_err());
    assert_eq!(cell.key(), KEY);
}

#[test]
#[should_panic(expected = "port Test\\Greeter: the container was dropped")]
fn panics_naming_the_key_when_the_container_is_gone() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    drop(c);
    cell.get();
}

#[test]
#[should_panic(expected = "port Test\\Greeter:")]
fn panics_naming_the_key_when_unbound() {
    let c = Rc::new(Container::new());
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    cell.get();
}

/// The cost of a call through a cell against a direct call, inside one request (run with --ignored --nocapture).
#[test]
#[ignore]
fn port_cell_overhead() {
    use std::hint::black_box;
    use std::time::Instant;
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    let direct: Rc<dyn Greeter> = fixed("a");
    c.start_request();
    cell.get();
    const N: u32 = 10_000_000;
    let start = Instant::now();
    for _ in 0..N {
        black_box(black_box(&direct).greet().len());
    }
    let direct_ns = start.elapsed().as_nanos() as f64 / N as f64;
    let start = Instant::now();
    for _ in 0..N {
        black_box(black_box(&cell).get().greet().len());
    }
    let cell_ns = start.elapsed().as_nanos() as f64 / N as f64;
    println!(
        "direct {direct_ns:.2} ns/call, cell {cell_ns:.2} ns/call, overhead {:.2} ns",
        cell_ns - direct_ns
    );
}

#[test]
fn with_calls_through_the_current_binding_and_survives_reentry_across_a_rebind() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = Rc::new(PortCell::<dyn Greeter>::new(&c, KEY));
    assert_eq!(cell.with(|g| g.greet()), "a");
    let inner = cell.with(|_| {
        singleton(&c, KEY, "b");
        cell.with(|g| g.greet())
    });
    assert_eq!(inner, "b");
    assert_eq!(cell.with(|g| g.greet()), "b");
}

/// As `port_cell_overhead`, through `with` (run with --ignored --nocapture).
#[test]
#[ignore]
fn port_cell_with_overhead() {
    use std::hint::black_box;
    use std::time::Instant;
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    let direct: Rc<dyn Greeter> = fixed("a");
    c.start_request();
    cell.get();
    const N: u32 = 10_000_000;
    for round in 0..3 {
        let start = Instant::now();
        for _ in 0..N {
            black_box(black_box(&direct).greet().len());
        }
        let direct_ns = start.elapsed().as_nanos() as f64 / N as f64;
        let start = Instant::now();
        for _ in 0..N {
            black_box(black_box(&cell).with(|g| g.greet().len()));
        }
        let cell_ns = start.elapsed().as_nanos() as f64 / N as f64;
        println!(
            "round {round}: direct {direct_ns:.2} ns/call, with {cell_ns:.2} ns/call, overhead {:.2} ns",
            cell_ns - direct_ns
        );
    }
}

#[test]
#[should_panic(expected = "port Test\\Greeter: the container was dropped")]
fn a_warm_cell_panics_naming_the_key_once_the_container_is_gone() {
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "a");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    assert_eq!(cell.with(|g| g.greet()), "a");
    drop(c);
    cell.with(|g| g.greet());
}

/// A value that records when it is dropped.
struct Watched(Rc<Cell<bool>>);

impl Greeter for Watched {
    fn greet(&self) -> String {
        "watched".to_string()
    }
}

impl Drop for Watched {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

#[test]
fn a_foreign_binding_is_never_held_past_its_request() {
    // A language bridge's value (a PHP object) is valid only for its request: the cell must not keep it after the
    // container lets it go, or it would be released after PHP freed it.
    let c = Rc::new(Container::new());
    singleton(&c, KEY, "rust");
    let cell = PortCell::<dyn Greeter>::new(&c, KEY);
    let dropped = Rc::new(Cell::new(false));
    c.start_request();
    {
        let _request = c.registering(Lifetime::Request);
        let flag = dropped.clone();
        c.singleton_foreign(KEY, move |_, _| {
            Ok(Rc::new(Rc::new(Watched(flag.clone())) as Rc<dyn Greeter>)
                as speedyspec_container::Resolved)
        });
    }
    assert_eq!(cell.with(|g| g.greet()), "watched");
    assert_eq!(cell.with(|g| g.greet()), "watched");
    c.end_request();
    assert!(
        dropped.get(),
        "the request's foreign value outlived the request"
    );
    assert_eq!(cell.with(|g| g.greet()), "rust");
}
