//! The container from PHP: bindings, autowiring, PHP overrides of Rust ports and their end with the request.
//!
//! PHP boots once per process, so this test has its own `main()` (`harness = false`) and runs each case in order.

use std::rc::Rc;

use speedyspec_container::{Container, Resolved};
use speedyspec_container_php::ContainerArea;
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, NativeArea, Php};
use speedyspec_php_ext::{Value, boundary_crossings, php_port, twins_area};

const GREETER: &str = "SpeedySpec\\Testing\\Greeter";
const COUNTER: &str = "SpeedySpec\\Testing\\Counter";
const CLOCK: &str = "SpeedySpec\\Testing\\Clock";

/// An edge port declared the way a contracts crate does, its twin generated the way a `-php` crate does.
#[speedyspec_port_macros::port(key = "SpeedySpec\\Testing\\Speaker", edge)]
pub trait Speaker {
    fn speak(&self, word: String) -> String;
}

__speedyspec_port_Speaker!(::speedyspec_php_port_macros::php_twin);

struct RustSpeaker;

impl Speaker for RustSpeaker {
    fn speak(&self, _word: String) -> String {
        "rust".to_string()
    }
}

/// A port bound only to test the stateful refusal through a proxy.
#[speedyspec_port_macros::port(key = "SpeedySpec\\Testing\\Ledger", edge)]
pub trait Ledger {
    fn total(&self) -> i64;
}

__speedyspec_port_Ledger!(::speedyspec_php_port_macros::php_twin);

struct RustLedger;

impl Ledger for RustLedger {
    fn total(&self) -> i64 {
        1
    }
}

#[php_port(name = "SpeedySpec\\Testing\\Greeter")]
pub trait Greeter {
    fn greet(&self, name: String) -> String;
}

struct RustGreeter;

impl Greeter for RustGreeter {
    fn greet(&self, name: String) -> String {
        format!("hello {name}")
    }
}

fn check(name: &str, ok: bool, detail: impl std::fmt::Debug) {
    if ok {
        println!("ok: {name}");
    } else {
        eprintln!("FAIL: {name}: {detail:?}");
        std::process::exit(1);
    }
}

fn is(result: &Result<Value, speedyspec_php::PhpError>, expected: Value) -> bool {
    matches!(result, Ok(value) if *value == expected)
}

fn greet(container: &Container) -> String {
    container
        .make_port::<dyn Greeter>(GREETER)
        .expect("the greeter resolves")
        .greet("ada".to_string())
}

const OVERRIDE: &str = r#"
    \SpeedySpec\Container\Container::getInstance()->bind(\SpeedySpec\Testing\Greeter::class, fn () => new class implements \SpeedySpec\Testing\Greeter {
        public function greet(string $name): string { return "hi $name"; }
    });
"#;

fn main() {
    let container = Rc::new(Container::new());
    container.bind_port::<dyn Greeter>(GREETER, |_, _| Ok(Rc::new(RustGreeter) as Rc<dyn Greeter>));
    container.singleton(COUNTER, |_, _| Ok(Rc::new(0_i64) as Resolved));
    container.singleton_port::<dyn Speaker>(SPEAKER_KEY, |_, _| {
        Ok(Rc::new(RustSpeaker) as Rc<dyn Speaker>)
    });
    container
        .singleton_port::<dyn Ledger>(LEDGER_KEY, |_, _| Ok(Rc::new(RustLedger) as Rc<dyn Ledger>));

    let area = ContainerArea::new("demo", container.clone())
        .port::<dyn Greeter>(GREETER, |object| Rc::new(PhpGreeter::new(object)))
        .stateful(COUNTER)
        .stateful(LEDGER_KEY)
        .bindings(vec![(CLOCK.to_string(), "FixedClock".to_string())]);
    let areas: Vec<Rc<dyn NativeArea>> = vec![twins_area(), Rc::new(area)];
    let php = Php::boot(BootOptions {
        hooks: Rc::new(MemoryHooks::new()),
        preload: &[],
        areas,
    })
    .expect("PHP boots");

    // php_bindings_installed_before_php_runs
    let first = php.eval(
        "t",
        r"return \SpeedySpec\Container\Container::getInstance()->has('SpeedySpec\Testing\Clock');",
    );
    check(
        "php_bindings_installed_before_php_runs",
        is(&first, Value::Bool(true)),
        &first,
    );
    check(
        "php_bindings_are_foreign",
        container.is_foreign(CLOCK),
        CLOCK,
    );
    let clock = php.eval(
        "t",
        r"final class FixedClock { public function now(): int { return 1700000000; } }
          return \SpeedySpec\Container\Container::getInstance()->make('SpeedySpec\Testing\Clock')->now();",
    );
    check(
        "php_binding_builds_its_class",
        is(&clock, Value::Int(1_700_000_000)),
        &clock,
    );

    // php_binds_and_makes
    let made = php.eval(
        "t",
        r"$c = \SpeedySpec\Container\Container::getInstance();
          $c->bind('answer', fn () => 42);
          $c->singleton('shared', fn () => new \stdClass());
          $c->instance('greeting', 'hello');
          $c->alias('greeting', 'salutation');
          $c->bind('sum', fn ($c, array $p) => $p['a'] + $p['b']);
          $shared = $c->make('shared');
          return [
              $c->make('answer'),
              $shared === $c->make('shared'),
              $c->make('salutation'),
              $c->makeWith('sum', ['a' => 2, 'b' => 3]),
              $c->bound('answer') && !$c->has('nothing-here'),
              $c->factory('answer')(),
          ];",
    );
    let expected = Value::Array(vec![
        (speedyspec_php_ext::Key::Int(0), Value::Int(42)),
        (speedyspec_php_ext::Key::Int(1), Value::Bool(true)),
        (speedyspec_php_ext::Key::Int(2), Value::Str("hello".into())),
        (speedyspec_php_ext::Key::Int(3), Value::Int(5)),
        (speedyspec_php_ext::Key::Int(4), Value::Bool(true)),
        (speedyspec_php_ext::Key::Int(5), Value::Int(42)),
    ]);
    check("php_binds_and_makes", is(&made, expected), &made);

    // php_autowires_constructor_dependencies
    let wired = php.eval(
        "t",
        r"interface Engine { public function name(): string; }
          final class V6 implements Engine { public function name(): string { return 'v6'; } }
          final class V8 implements Engine { public function name(): string { return 'v8'; } }
          final class Wheels {}
          final class Car { public function __construct(public Engine $engine, public Wheels $wheels, public int $count = 4) {} }
          final class Truck { public function __construct(public Engine $engine) {} }
          $c = \SpeedySpec\Container\Container::getInstance();
          $c->bind(Engine::class, V6::class);
          $c->when(Truck::class)->needs(Engine::class)->give(V8::class);
          $car = $c->make(Car::class);
          return [$car->engine->name(), $car->wheels instanceof Wheels, $car->count, $c->make(Truck::class)->engine->name(),
                  $c->call(fn (Wheels $w, int $n = 7) => $n, [])];",
    );
    let expected = Value::Array(vec![
        (speedyspec_php_ext::Key::Int(0), Value::Str("v6".into())),
        (speedyspec_php_ext::Key::Int(1), Value::Bool(true)),
        (speedyspec_php_ext::Key::Int(2), Value::Int(4)),
        (speedyspec_php_ext::Key::Int(3), Value::Str("v8".into())),
        (speedyspec_php_ext::Key::Int(4), Value::Int(7)),
    ]);
    check(
        "php_autowires_constructor_dependencies",
        is(&wired, expected),
        &wired,
    );

    // rust_binding_resolves_without_entering_php
    let before = boundary_crossings();
    let greeting = greet(&container);
    check(
        "rust_binding_resolves_without_entering_php",
        greeting == "hello ada" && boundary_crossings() == before,
        (greeting, boundary_crossings() - before),
    );

    // php_binding_overrides_a_port_for_rust_callers
    php.eval("t", OVERRIDE).expect("PHP binds the port");
    let greeting = greet(&container);
    check(
        "php_binding_overrides_a_port_for_rust_callers",
        greeting == "hi ada" && container.is_foreign(GREETER),
        greeting,
    );
    let from_php = php.eval(
        "t",
        r"return \SpeedySpec\Container\Container::getInstance()->make(\SpeedySpec\Testing\Greeter::class)->greet('bo');",
    );
    check(
        "php_makes_its_own_override",
        is(&from_php, Value::Str("hi bo".into())),
        &from_php,
    );

    // override_is_gone_next_request (Review Focus 4)
    php.restart_request().expect("the next request starts");
    let greeting = greet(&container);
    check(
        "override_is_gone_next_request",
        greeting == "hello ada" && !container.is_foreign(GREETER),
        greeting,
    );
    let answer = php.eval(
        "t",
        r"return \SpeedySpec\Container\Container::getInstance()->bound('answer');",
    );
    check(
        "php_registrations_end_with_the_request",
        is(&answer, Value::Bool(false)),
        &answer,
    );

    // override_is_gone_after_a_fatal_error (Review Focus 1)
    php.eval("t", OVERRIDE).expect("PHP binds the port");
    let fatal = php.eval("t", "undefined_function_here();");
    check("fatal_error_reported", fatal.is_err(), &fatal);
    php.restart_request().expect("the next request starts");
    let greeting = greet(&container);
    check(
        "override_is_gone_after_a_fatal_error",
        greeting == "hello ada",
        greeting,
    );

    // stateful_port_bound_after_use_is_refused
    container.make(COUNTER).expect("the counter resolves");
    let refused = php.eval(
        "t",
        r"try {
              \SpeedySpec\Container\Container::getInstance()->bind('SpeedySpec\Testing\Counter', fn () => 1);
              return 'bound';
          } catch (\SpeedySpec\Container\ContainerException $e) {
              return $e->getMessage();
          }",
    );
    check(
        "stateful_port_bound_after_use_is_refused",
        is(&refused, Value::Str(
                "SpeedySpec\\Testing\\Counter holds state and was already used in this request; bind it in app.demo.php.bindings instead"
                    .into(),
            )),
        &refused,
    );
    php.restart_request().expect("the next request starts");
    let early = php.eval(
        "t",
        r"\SpeedySpec\Container\Container::getInstance()->bind('SpeedySpec\Testing\Counter', fn () => 1);
          return \SpeedySpec\Container\Container::getInstance()->make('SpeedySpec\Testing\Counter');",
    );
    check(
        "stateful_port_bound_before_use_is_allowed",
        is(&early, Value::Int(1)),
        &early,
    );

    // php_binding_of_an_edge_port_reaches_rust_through_its_proxy_and_reverts: no `.port()` call registered the
    // Speaker adapter; php_twin! did.
    php.restart_request().expect("the next request starts");
    let speaker = ProxySpeaker::new(&container);
    let before = speaker.speak("hi".into());
    php.eval(
        "t",
        r"class Loud implements \SpeedySpec\Testing\Speaker {
              public function speak(string $word): string { return strtoupper($word); }
          }
          \SpeedySpec\Container\Container::getInstance()->bind(\SpeedySpec\Testing\Speaker::class, fn () => new Loud());",
    )
    .expect("PHP binds the edge port");
    let during = speaker.speak("hi".into());
    php.restart_request().expect("the next request starts");
    let after = speaker.speak("hi".into());
    check(
        "php_binding_of_an_edge_port_reaches_rust_through_its_proxy_and_reverts",
        before == "rust" && during == "HI" && after == "rust",
        (before, during, after),
    );

    // stateful_edge_port_rebind_after_use_is_refused_in_a_later_request (Review Focus 1)
    let ledger = ProxyLedger::new(&container);
    ledger.total();
    php.restart_request().expect("the next request starts");
    ledger.total();
    let refused = php.eval(
        "t",
        r"try {
              \SpeedySpec\Container\Container::getInstance()->bind(\SpeedySpec\Testing\Ledger::class, fn () => null);
              return 'bound';
          } catch (\SpeedySpec\Container\ContainerException $e) {
              return $e->getMessage();
          }",
    );
    check(
        "stateful_edge_port_rebind_after_use_is_refused_in_a_later_request",
        matches!(&refused, Ok(Value::Str(m)) if m.contains("SpeedySpec\\Testing\\Ledger holds state")),
        &refused,
    );
}
