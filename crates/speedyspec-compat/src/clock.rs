//! The frozen clock: a scenario's `clock` makes PHP's own sense of "now" a fixed Unix time.
//!
//! PHP code cannot redefine a builtin (`time()` and the rest), and WordPress declares `current_time()` and `wp_date()`
//! without a `function_exists()` guard, so neither the driver nor an area's `setup.php` can replace them. The
//! interpreter can, though: an internal function is a `zend_function` whose `handler` is a C function pointer, and
//! the embedding process may point it elsewhere. [`install`] does that for the builtins WordPress reads the time
//! through (a Decorator over each original handler):
//!
//! - `time()`;
//! - `microtime()`, as `"0.00000000 <clock>"` or the clock as a float;
//! - `date()` and `gmdate()` without a timestamp (or with a null one), formatted at the clock by PHP's own
//!   `php_format_date()`;
//! - `new DateTime()`, `new DateTimeImmutable()`, `date_create()` and `date_create_immutable()` for "now" (no
//!   argument, null, `""` or `"now"`): the object is built by the original handler, then moved to the clock as
//!   `DateTime::setTimestamp()` would move it, keeping its time zone.
//!
//! WordPress's `current_time()` (`time()` and `new DateTime( 'now', ... )`), `current_datetime()`, `wp_date()` and
//! everything above them follow. Relative strings (`"+1 day"`, `"tomorrow"`), `strtotime()` without a base,
//! `mktime()`, `getdate()`, `localtime()` and `hrtime()` still read the real clock.
//!
//! The clock is `$GLOBALS['speedyspec_compat_clock']`, which the driver sets from the scenario's and each step's
//! `clock`. While it is not an int every decorated function calls its original handler unchanged, so the decorators
//! do nothing to a scenario without a clock, and [`install`] runs only for a scenario that has one. Internal
//! functions outlive requests, so once installed the decorators stay for the process.
//!
//! The ports' Rust code does not read PHP's clock: a port that needs the time reads it through PHP (or a port that
//! answers what PHP answers) for its replay to see the frozen clock.

use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use speedyspec_hooks::Value;
use speedyspec_php::ext_php_rs::boxed::ZBox;
use speedyspec_php::ext_php_rs::ffi::{
    HashTable, ext_php_rs_executor_globals, zend_execute_data, zend_function, zend_hash_str_find,
    zend_object, zend_string, zif_handler, zval,
};
use speedyspec_php::ext_php_rs::types::{ZendStr, Zval};

/// The global the driver keeps the frozen clock in.
const CLOCK: &str = "speedyspec_compat_clock";

/// `ZEND_INTERNAL_FUNCTION`.
const INTERNAL: u8 = 1;

unsafe extern "C" {
    /// `PHPAPI zend_string *php_format_date(const char *format, size_t format_len, time_t ts, bool localtime)`.
    fn php_format_date(
        format: *const c_char,
        len: usize,
        ts: i64,
        localtime: bool,
    ) -> *mut zend_string;
    /// `void timelib_unixtime2local(timelib_time *tm, timelib_sll ts)`.
    fn timelib_unixtime2local(time: *mut c_void, ts: i64);
    /// `void timelib_update_ts(timelib_time *time, timelib_tzinfo *tzi)`.
    fn timelib_update_ts(time: *mut c_void, tzi: *mut c_void);
}

/// The head of timelib's `timelib_time`, up to the microseconds.
#[repr(C)]
struct TimelibHead {
    ymd: [i64; 3],
    his: [i64; 3],
    us: i64,
}

/// What each decorated function does when the clock is frozen.
#[derive(Clone, Copy)]
enum Kind {
    Time,
    Microtime,
    Date {
        localtime: bool,
    },
    /// A constructor (the object is `$this`) or a `date_create*()` function (the object is the return value).
    Now {
        constructor: bool,
    },
}

/// One decorated function: where it lives, what it does frozen, and its original handler (0 until installed).
struct Decorated {
    class: Option<&'static str>,
    name: &'static str,
    kind: Kind,
    original: AtomicUsize,
}

const fn decorated(class: Option<&'static str>, name: &'static str, kind: Kind) -> Decorated {
    Decorated {
        class,
        name,
        kind,
        original: AtomicUsize::new(0),
    }
}

static DECORATED: [Decorated; 8] = [
    decorated(None, "time", Kind::Time),
    decorated(None, "microtime", Kind::Microtime),
    decorated(None, "date", Kind::Date { localtime: true }),
    decorated(None, "gmdate", Kind::Date { localtime: false }),
    decorated(None, "date_create", Kind::Now { constructor: false }),
    decorated(
        None,
        "date_create_immutable",
        Kind::Now { constructor: false },
    ),
    decorated(
        Some("datetime"),
        "__construct",
        Kind::Now { constructor: true },
    ),
    decorated(
        Some("datetimeimmutable"),
        "__construct",
        Kind::Now { constructor: true },
    ),
];

/// The handler of each entry of [`DECORATED`], by index.
static HANDLERS: [unsafe extern "C" fn(*mut zend_execute_data, *mut zval); 8] = [
    handler::<0>,
    handler::<1>,
    handler::<2>,
    handler::<3>,
    handler::<4>,
    handler::<5>,
    handler::<6>,
    handler::<7>,
];

/// Points every decorated function at its decorator, once per process (again if PHP re-registered its functions).
/// [`crate::PhpRunner`] calls it inside a request it has just started: the function and class tables are read.
pub(crate) fn install() -> Result<(), String> {
    for (index, entry) in DECORATED.iter().enumerate() {
        // SAFETY: the runner has a request open, so the executor's function and class tables are live.
        let func = unsafe { find(entry.class, entry.name) };
        if func.is_null() {
            return Err(format!(
                "the frozen clock cannot find {}{}",
                entry.class.map(|c| format!("{c}::")).unwrap_or_default(),
                entry.name
            ));
        }
        let ours = HANDLERS[index];
        // SAFETY: `func` is a live function from the engine's tables; only internal functions are changed.
        unsafe {
            if (*func).type_ != INTERNAL {
                return Err(format!("{} is not an internal function", entry.name));
            }
            let current = (*func).internal_function.handler;
            if current.map(|h| h as usize) == Some(ours as usize) {
                continue;
            }
            let Some(original) = current else {
                return Err(format!("{} has no handler", entry.name));
            };
            entry.original.store(original as usize, Ordering::SeqCst);
            (*func).internal_function.handler = Some(ours);
        }
    }
    Ok(())
}

/// A function in the function table, or a method of a class in the class table; null when there is none.
///
/// # Safety
///
/// A request is open.
unsafe fn find(class: Option<&str>, name: &str) -> *mut zend_function {
    // SAFETY: the caller keeps a request open; the tables hold IS_PTR zvals.
    unsafe {
        let globals = ext_php_rs_executor_globals();
        let table: *mut HashTable = match class {
            None => (*globals).function_table,
            Some(class) => {
                let zv =
                    zend_hash_str_find((*globals).class_table, class.as_ptr().cast(), class.len());
                if zv.is_null() {
                    return std::ptr::null_mut();
                }
                &raw mut (*(*zv).value.ce).function_table
            }
        };
        let zv = zend_hash_str_find(table, name.as_ptr().cast(), name.len());
        if zv.is_null() {
            std::ptr::null_mut()
        } else {
            (*zv).value.func
        }
    }
}

/// The frozen clock, when the driver has set one.
fn frozen() -> Option<i64> {
    match speedyspec_php::global(CLOCK) {
        Some(Value::Int(clock)) => Some(clock),
        _ => None,
    }
}

/// The `i`th argument of a call, if it was passed.
///
/// # Safety
///
/// `ex` is the live frame of the call.
unsafe fn arg<'a>(ex: *mut zend_execute_data, i: usize) -> Option<&'a Zval> {
    // SAFETY: the caller's frame is live; `num_args` bounds the slots.
    unsafe {
        if i >= (*ex).This.u2.num_args as usize {
            return None;
        }
        (*ex).zend_call_arg(i).map(|zv| &*zv)
    }
}

/// Whether a `DateTime` "now" argument (the first) means the current time: absent, null, `""` or `"now"`.
fn means_now(arg: Option<&Zval>) -> bool {
    match arg {
        None => true,
        Some(zv) if zv.is_null() => true,
        Some(zv) => zv
            .str()
            .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "" | "now")),
    }
}

/// The decorator of `DECORATED[I]`.
unsafe extern "C" fn handler<const I: usize>(ex: *mut zend_execute_data, ret: *mut zval) {
    let entry = &DECORATED[I];
    let original = entry.original.load(Ordering::SeqCst);
    // SAFETY: `original` was this function's own handler, stored before the decorator was installed.
    let call_original = || unsafe {
        let original: zif_handler = std::mem::transmute::<usize, zif_handler>(original);
        if let Some(original) = original {
            original(ex, ret);
        }
    };
    let Some(clock) = frozen() else {
        return call_original();
    };
    // SAFETY: PHP called the handler with its live frame and return slot.
    unsafe {
        let ret_zv = &mut *ret.cast::<Zval>();
        let count = (*ex).This.u2.num_args as usize;
        match entry.kind {
            Kind::Time if count == 0 => ret_zv.set_long(clock),
            Kind::Microtime if count <= 1 => match arg(ex, 0).map(Zval::bool) {
                None | Some(Some(false)) => {
                    if ret_zv
                        .set_string(&format!("0.00000000 {clock}"), false)
                        .is_err()
                    {
                        call_original();
                    }
                }
                Some(Some(true)) => ret_zv.set_double(clock as f64),
                Some(None) => call_original(),
            },
            Kind::Date { localtime }
                if count == 1 || (count == 2 && arg(ex, 1).is_some_and(Zval::is_null)) =>
            {
                match arg(ex, 0).and_then(Zval::str) {
                    Some(format) => {
                        let text =
                            php_format_date(format.as_ptr().cast(), format.len(), clock, localtime);
                        ret_zv.set_zend_string(ZBox::from_raw(text.cast::<ZendStr>()));
                    }
                    None => call_original(),
                }
            }
            Kind::Now { constructor } if means_now(arg(ex, 0)) => {
                call_original();
                if !(*ext_php_rs_executor_globals()).exception.is_null() {
                    return;
                }
                let object = if constructor {
                    (*ex).get_self().map(|o| &raw mut *o)
                } else {
                    ret_zv.object_mut().map(|o| &raw mut *o)
                };
                if let Some(object) = object {
                    move_to(object.cast::<zend_object>(), clock);
                }
            }
            _ => call_original(),
        }
    }
}

/// Moves a `DateTime` or `DateTimeImmutable` object to `clock`, as `DateTime::setTimestamp()` does
/// (`php_date_timestamp_set`): local fields from the timestamp in the object's own zone, and no microseconds.
///
/// # Safety
///
/// `object` is the `std` member of a live `php_date_obj` (`{ timelib_time *time; zend_object std; }`).
unsafe fn move_to(object: *mut zend_object, clock: i64) {
    // SAFETY: as the caller promises; `time` is the member before `std`.
    unsafe {
        let date = object
            .cast::<u8>()
            .sub(std::mem::size_of::<*mut c_void>())
            .cast::<*mut c_void>();
        let time = *date;
        if time.is_null() {
            return;
        }
        timelib_unixtime2local(time, clock);
        timelib_update_ts(time, std::ptr::null_mut());
        (*time.cast::<TimelibHead>()).us = 0;
    }
}
