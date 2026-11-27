//! Writing a value into a hash bucket copies its value and type only, never the bucket's hash-chain word (`u2`).
//!
//! A PHP object reaches Rust as `Value::Opaque`, a handle holding a copy of the object's zval. Copying that whole zval
//! into an array bucket or a symbol-table bucket also copied its `u2`, which a bucket uses as the next link in its
//! collision chain: lookups then missed entries or looped forever. Integer keys 9, 1 and 25 collide in an 8-slot
//! table, which is how it was found.
//!
//! PHP boots once per process, so this file has its own `main()` (`harness = false`). A watchdog thread fails the
//! run if any case hangs.

use std::process::ExitCode;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use speedyspec_hooks::{Key, Value};
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, Php, set_global};

type Case = fn(&Php) -> Result<(), String>;

const CASES: &[(&str, Case)] = &[
    (
        "objects_into_colliding_keys_reverse_order",
        case_objects_reverse_order,
    ),
    (
        "objects_into_colliding_keys_same_order",
        case_objects_same_order,
    ),
    (
        "set_global_keeps_symbol_table_chains",
        case_set_global_keeps_chains,
    ),
];

/// How long one case may run before the watchdog calls it hung.
const HANG_AFTER: Duration = Duration::from_secs(20);

fn php_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The objects PHP built for `keys`, in that order, each with an `id` property equal to its key.
fn objects_from_php(php: &Php, keys: &[i64]) -> Result<Vec<(i64, Value)>, String> {
    let list = keys
        .iter()
        .map(|k| format!("{k} => (object) ['id' => {k}]"))
        .collect::<Vec<_>>()
        .join(", ");
    let got = php
        .eval("t", &format!("return [{list}];"))
        .map_err(php_err)?;
    let Value::Array(items) = got else {
        return Err(format!("eval gave {got:?}, want an array"));
    };
    items
        .into_iter()
        .map(|(key, value)| match (key, &value) {
            (Key::Int(k), Value::Opaque(_)) => Ok((k, value)),
            (key, value) => Err(format!("entry {key:?} => {value:?} is not an object")),
        })
        .collect()
}

/// Writes the objects into `$GLOBALS[$name]` keyed in `order`, then has PHP look each one up and iterate the array.
fn write_and_read_back(php: &Php, name: &str, from: &[i64], order: &[i64]) -> Result<(), String> {
    let objects = objects_from_php(php, from)?;
    let array = order
        .iter()
        .map(|k| {
            let (_, object) = objects
                .iter()
                .find(|(key, _)| key == k)
                .ok_or(format!("no object for key {k}"))?;
            Ok((Key::Int(*k), object.clone()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    set_global(name, &Value::Array(array)).map_err(php_err)?;

    let lookups = order
        .iter()
        .map(|k| format!("(isset($a[{k}]) ? $a[{k}]->id : 'missing')"))
        .collect::<Vec<_>>()
        .join(" . ',' . ");
    let code = format!(
        "$a = $GLOBALS['{name}']; $seen = []; foreach ($a as $k => $o) {{ $seen[] = $k . ':' . $o->id; }} \
         $a[99] = 'new'; return {lookups} . '|' . implode(',', $seen) . '|' . count($a);"
    );
    let got = php.eval("t", &code).map_err(php_err)?;
    let ids = order
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let seen = order
        .iter()
        .map(|k| format!("{k}:{k}"))
        .collect::<Vec<_>>()
        .join(",");
    let want = Value::Str(format!("{ids}|{seen}|{}", order.len() + 1));
    if got == want {
        Ok(())
    } else {
        Err(format!("got {got:?}, want {want:?}"))
    }
}

fn case_objects_reverse_order(php: &Php) -> Result<(), String> {
    write_and_read_back(php, "zv_reverse", &[25, 1, 9], &[9, 1, 25])
}

fn case_objects_same_order(php: &Php) -> Result<(), String> {
    write_and_read_back(php, "zv_same", &[9, 1, 25], &[9, 1, 25])?;
    write_and_read_back(php, "zv_shuffled", &[1, 25, 9], &[9, 1, 25])
}

/// Global names that all share one hash under PHP's DJBX33A ("Ez" and "FY" hash alike), so they share one chain.
fn colliding_names() -> Vec<String> {
    (0..16u32)
        .map(|bits| {
            let blocks: String = (0..4)
                .map(|b| if bits >> b & 1 == 1 { "FY" } else { "Ez" })
                .collect();
            format!("zvg{blocks}")
        })
        .collect()
}

/// Overwriting existing globals from Rust leaves every global in their shared hash chain reachable.
fn case_set_global_keeps_chains(php: &Php) -> Result<(), String> {
    let names = colliding_names();
    let quoted = names
        .iter()
        .map(|n| format!("'{n}'"))
        .collect::<Vec<_>>()
        .join(", ");
    // Every variable the check below uses exists first, so the check inserts nothing and never rehashes the table.
    php.eval(
        "t",
        &format!(
            "$zv_names = [{quoted}]; $zv_bad = []; $zv_i = 0; $zv_n = ''; \
             foreach ($zv_names as $zv_i => $zv_n) {{ $GLOBALS[$zv_n] = (object) ['id' => -1]; }}"
        ),
    )
    .map_err(php_err)?;
    let keys = (0..names.len() as i64).collect::<Vec<_>>();
    let objects = objects_from_php(php, &keys)?;
    for (i, object) in &objects {
        set_global(&names[*i as usize], object).map_err(php_err)?;
    }
    let got = php
        .eval(
            "t",
            "$zv_bad = []; foreach ($zv_names as $zv_i => $zv_n) { \
             if (!isset($GLOBALS[$zv_n]) || $GLOBALS[$zv_n]->id !== $zv_i) { $zv_bad[] = $zv_n; } } \
             return implode(',', $zv_bad);",
        )
        .map_err(php_err)?;
    if got == Value::Str(String::new()) {
        Ok(())
    } else {
        Err(format!("globals lost or wrong: {got:?}"))
    }
}

/// Fails the process if one case runs longer than [`HANG_AFTER`]. Each message names the case starting; `None` or a
/// closed channel ends the watch.
fn watchdog() -> mpsc::Sender<Option<&'static str>> {
    let (tx, rx) = mpsc::channel::<Option<&'static str>>();
    thread::spawn(move || {
        let mut current = "boot";
        loop {
            match rx.recv_timeout(HANG_AFTER) {
                Ok(Some(name)) => current = name,
                Ok(None) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    println!("FAILED: {current}: hung for {}s", HANG_AFTER.as_secs());
                    std::process::exit(1);
                }
            }
        }
    });
    tx
}

fn main() -> ExitCode {
    let watch = watchdog();
    let php = match Php::boot(BootOptions {
        hooks: Rc::new(MemoryHooks::default()),
        preload: &[],
        areas: Vec::new(),
    }) {
        Ok(php) => php,
        Err(err) => {
            println!("FAILED: boot: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mut failed = 0;
    for (name, case) in CASES {
        let _ = watch.send(Some(name));
        match case(&php) {
            Ok(()) => println!("ok: {name}"),
            Err(why) => {
                failed += 1;
                println!("FAILED: {name}: {why}");
            }
        }
    }
    let _ = watch.send(Some("shutdown"));
    drop(php);
    let _ = watch.send(None);
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
