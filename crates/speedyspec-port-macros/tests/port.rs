//! `#[port]`: the key, the proxy, and the edge signature.

use std::rc::Rc;

use speedyspec_container::Container;
use speedyspec_port_macros::port;

#[port(key = "T\\Greeter", edge)]
pub trait Greeter {
    fn greet(&self, name: String) -> String;
    fn count(&self) -> i64;
}

#[port(key = "T\\HookApi")]
pub trait HookApi {
    fn fire(&self, hook: &str) -> bool;
}

struct Hello(&'static str);

impl Greeter for Hello {
    fn greet(&self, name: String) -> String {
        format!("{} {name}", self.0)
    }
    fn count(&self) -> i64 {
        7
    }
}

fn bind(c: &Container, word: &'static str) {
    c.singleton_port::<dyn Greeter>(GREETER_KEY, move |_, _| Ok(Rc::new(Hello(word))));
}

#[test]
fn key_const_is_generated() {
    assert_eq!(GREETER_KEY, "T\\Greeter");
    assert_eq!(HOOK_API_KEY, "T\\HookApi");
}

#[test]
fn proxy_forwards_to_the_current_binding() {
    let c = Rc::new(Container::new());
    bind(&c, "hello");
    let proxy = ProxyGreeter::new(&c);
    assert_eq!(proxy.greet("ann".into()), "hello ann");
    assert_eq!(proxy.count(), 7);
    bind(&c, "hi");
    assert_eq!(proxy.greet("bo".into()), "hi bo");
}

#[test]
fn a_proxy_is_usable_as_the_port() {
    let c = Rc::new(Container::new());
    bind(&c, "hey");
    let port: Rc<dyn Greeter> = Rc::new(ProxyGreeter::new(&c));
    assert_eq!(port.greet("cy".into()), "hey cy");
}

#[test]
fn a_non_edge_port_has_a_proxy_with_borrowed_arguments() {
    struct Fires;
    impl HookApi for Fires {
        fn fire(&self, hook: &str) -> bool {
            hook == "init"
        }
    }
    let c = Rc::new(Container::new());
    c.singleton_port::<dyn HookApi>(HOOK_API_KEY, |_, _| Ok(Rc::new(Fires)));
    assert!(ProxyHookApi::new(&c).fire("init"));
}

macro_rules! capture {
    (key = $key:literal, path = $path:path, trait $name:ident { $($body:tt)* }) => {
        const CAPTURED_KEY: &str = $key;
        const CAPTURED_NAME: &str = stringify!($name);
        const CAPTURED_BODY: &str = stringify!($($body)*);
        fn captured_path_is_the_trait(port: &dyn $path) -> i64 {
            port.count()
        }
    };
}

__speedyspec_port_Greeter!(capture);

#[test]
fn edge_emits_the_signature_macro() {
    assert_eq!(CAPTURED_KEY, "T\\Greeter");
    assert_eq!(CAPTURED_NAME, "Greeter");
    assert!(CAPTURED_BODY.contains("fn greet"), "{CAPTURED_BODY}");
    assert!(CAPTURED_BODY.contains("fn count"), "{CAPTURED_BODY}");
    assert_eq!(captured_path_is_the_trait(&Hello("x")), 7);
}

#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
