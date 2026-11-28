//! The options surface compiles, stays object-safe, reads typed values as the server's facade did and converts its
//! errors.

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use speedyspec_wp_hooks_contracts::{HookError, HookValue};
use speedyspec_wp_object_cache_contracts::ObjectCacheError;
use speedyspec_wp_options_contracts::{
    Autoload, AutoloadRequest, ConditionalOptions, ConditionalWrite, FromOptionValue, Options,
    OptionsError, TypedOptions, php_empty, php_string,
};

/// Options held in memory: enough to drive the facade and the compare-and-swap shape.
#[derive(Default)]
struct Dummy {
    rows: RefCell<Vec<(String, HookValue)>>,
}

impl Dummy {
    fn with(rows: &[(&str, HookValue)]) -> Dummy {
        Dummy {
            rows: RefCell::new(
                rows.iter()
                    .map(|(n, v)| (n.to_string(), v.clone()))
                    .collect(),
            ),
        }
    }

    fn bytes(value: &HookValue) -> Vec<u8> {
        php_string(value).unwrap_or_default()
    }
}

impl Options for Dummy {
    fn get_option(
        &self,
        name: &str,
        default: Option<HookValue>,
    ) -> Result<HookValue, OptionsError> {
        if name == "throws" {
            return Err(HookError::message("callback failed").into());
        }
        Ok(self
            .rows
            .borrow()
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| default.unwrap_or(HookValue::Bool(false))))
    }

    fn add_option(
        &self,
        name: &str,
        value: HookValue,
        _deprecated: HookValue,
        _autoload: HookValue,
    ) -> Result<bool, OptionsError> {
        if self.stored_bytes(name)?.is_some() {
            return Ok(false);
        }
        self.rows.borrow_mut().push((name.to_string(), value));
        Ok(true)
    }

    fn update_option(
        &self,
        name: &str,
        value: HookValue,
        autoload: HookValue,
    ) -> Result<bool, OptionsError> {
        let mut rows = self.rows.borrow_mut();
        match rows.iter_mut().find(|(n, _)| n == name) {
            Some(slot) if slot.1 == value => Ok(false),
            Some(slot) => {
                slot.1 = value;
                Ok(true)
            }
            None => {
                drop(rows);
                self.add_option(name, value, HookValue::Str(String::new()), autoload)
            }
        }
    }

    fn delete_option(&self, name: &str) -> Result<bool, OptionsError> {
        let mut rows = self.rows.borrow_mut();
        let before = rows.len();
        rows.retain(|(n, _)| n != name);
        Ok(rows.len() != before)
    }
}

impl ConditionalOptions for Dummy {
    fn update_option_if(
        &self,
        name: &str,
        expected: Option<&[u8]>,
        value: HookValue,
        autoload: HookValue,
    ) -> Result<ConditionalWrite, OptionsError> {
        if self.stored_bytes(name)?.as_deref() != expected {
            return Ok(ConditionalWrite::Conflict);
        }
        Ok(if self.update_option(name, value, autoload)? {
            ConditionalWrite::Written
        } else {
            ConditionalWrite::Unchanged
        })
    }

    fn stored_bytes(&self, name: &str) -> Result<Option<Vec<u8>>, OptionsError> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| Dummy::bytes(v)))
    }

    fn forget_cached(&self, _name: &str) -> Result<(), OptionsError> {
        Ok(())
    }

    fn serialized_form(&self, value: &HookValue) -> Result<Vec<u8>, OptionsError> {
        Ok(Dummy::bytes(value))
    }
}

fn typed(rows: &[(&str, HookValue)]) -> TypedOptions {
    TypedOptions::new(Rc::new(Dummy::with(rows)) as Rc<dyn Options>)
}

#[test]
fn the_trait_is_object_safe() {
    let options: Rc<dyn Options> =
        Rc::new(Dummy::with(&[("blogname", HookValue::Str("A".into()))]));
    assert_eq!(
        options.get_option("blogname", None).unwrap(),
        HookValue::Str("A".into())
    );
    assert_eq!(
        options
            .get_option("missing", Some(HookValue::Int(7)))
            .unwrap(),
        HookValue::Int(7)
    );
    assert!(
        options
            .add_option(
                "new",
                HookValue::Int(1),
                HookValue::Str(String::new()),
                HookValue::Null
            )
            .unwrap()
    );
    assert!(
        options
            .update_option("new", HookValue::Int(2), HookValue::Null)
            .unwrap()
    );
    assert!(options.delete_option("new").unwrap());
    assert!(!options.delete_option("new").unwrap());
}

#[test]
fn a_missing_or_false_option_reads_as_none_for_every_type() {
    let options = typed(&[("off", HookValue::Bool(false))]);
    for name in ["missing", "off"] {
        assert!(options.get::<HookValue>(name).unwrap().is_none());
        assert!(options.get::<String>(name).unwrap().is_none());
        assert!(options.get::<i64>(name).unwrap().is_none());
        assert!(options.get::<bool>(name).unwrap().is_none());
    }
}

#[test]
fn an_int_option_reads_as_each_type() {
    let options = typed(&[("n", HookValue::Int(42)), ("zero", HookValue::Int(0))]);
    assert_eq!(
        options.get::<HookValue>("n").unwrap(),
        Some(HookValue::Int(42))
    );
    assert_eq!(options.get::<String>("n").unwrap().as_deref(), Some("42"));
    assert_eq!(options.get::<i64>("n").unwrap(), Some(42));
    assert_eq!(options.get::<bool>("n").unwrap(), Some(true));
    assert_eq!(options.get::<bool>("zero").unwrap(), Some(false));
}

#[test]
fn a_string_option_reads_as_each_type() {
    let options = typed(&[
        ("num", HookValue::Str(" \t12\n".into())),
        ("word", HookValue::Str("on".into())),
        ("empty", HookValue::Str(String::new())),
        ("zero", HookValue::Str("0".into())),
    ]);
    assert_eq!(options.get::<i64>("num").unwrap(), Some(12));
    assert_eq!(options.get::<i64>("word").unwrap(), None);
    assert_eq!(
        options.get::<String>("word").unwrap().as_deref(),
        Some("on")
    );
    assert_eq!(options.get::<String>("empty").unwrap().as_deref(), Some(""));
    assert_eq!(options.get::<bool>("word").unwrap(), Some(true));
    assert_eq!(options.get::<bool>("empty").unwrap(), Some(false));
    assert_eq!(options.get::<bool>("zero").unwrap(), Some(false));
}

#[test]
fn conversions_match_php_casts() {
    assert_eq!(
        String::from_option_value(HookValue::Bool(true)).as_deref(),
        Some("1")
    );
    assert_eq!(
        String::from_option_value(HookValue::Float(0.1)).as_deref(),
        Some("0.1")
    );
    assert_eq!(
        String::from_option_value(HookValue::Array(Vec::new())),
        None
    );
    assert_eq!(i64::from_option_value(HookValue::Float(1.0)), None);
    assert_eq!(
        bool::from_option_value(HookValue::Array(Vec::new())),
        Some(false)
    );
    assert!(php_empty(&HookValue::Null));
    assert!(!php_empty(&HookValue::Str("a".into())));
}

#[test]
fn a_read_error_reaches_the_typed_reader() {
    let options = typed(&[]);
    assert!(matches!(
        options.get::<String>("throws"),
        Err(OptionsError::Hook(_))
    ));
}

#[test]
fn errors_convert_and_keep_their_source() {
    let hook: OptionsError = HookError::message("boom").into();
    assert!(matches!(&hook, OptionsError::Hook(e) if e.message == "boom"));
    assert_eq!(hook.to_string(), "a hook failed: boom");
    assert_eq!(hook.source().unwrap().to_string(), "boom");

    let cache: OptionsError = ObjectCacheError::IllegalOffset("array".into()).into();
    assert!(matches!(cache, OptionsError::Cache(_)));
    assert!(cache.source().is_some());

    let db = OptionsError::Db("gone".into());
    assert_eq!(db.to_string(), "the database failed: gone");
    assert!(db.source().is_none());

    let php = OptionsError::NeedsPhp {
        function: "get_option",
        option: "blacklist_keys".into(),
    };
    assert_eq!(
        php.to_string(),
        "get_option() for the option blacklist_keys needs PHP"
    );
}

#[test]
fn conditional_write_equality_is_unchanged() {
    use ConditionalWrite::{Conflict, Unchanged, Written};
    let all = [Written, Unchanged, Conflict];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(a == b, i == j);
        }
    }
    let copied = Written;
    let again = copied;
    assert_eq!(copied, again);
}

#[test]
fn the_conditional_trait_is_object_safe_and_extends_options() {
    let conditional: Rc<dyn ConditionalOptions> = Rc::new(Dummy::with(&[]));
    conditional.forget_cached("blogname").unwrap();
    assert_eq!(conditional.stored_bytes("blogname").unwrap(), None);
    // A compare-and-swap writer also reads and writes through the plain option API.
    assert!(
        conditional
            .update_option("blogname", HookValue::Str("A".into()), HookValue::Null)
            .unwrap()
    );
    let plain: Rc<dyn Options> = Rc::new(Dummy::with(&[]));
    let _ = TypedOptions::new(plain);
}

#[test]
fn a_compare_and_swap_write_conflicts_only_when_the_bytes_moved() {
    let options: Rc<dyn ConditionalOptions> =
        Rc::new(Dummy::with(&[("cron", HookValue::Str("a".into()))]));
    let read = options.get_option("cron", None).unwrap();
    let version = options.serialized_form(&read).unwrap();
    assert_eq!(options.stored_bytes("cron").unwrap(), Some(version.clone()));

    // Unchanged since the read: the write goes through, then the same value again is no write.
    let written = options
        .update_option_if(
            "cron",
            Some(&version),
            HookValue::Str("b".into()),
            HookValue::Bool(true),
        )
        .unwrap();
    assert_eq!(written, ConditionalWrite::Written);
    let b = options
        .serialized_form(&HookValue::Str("b".into()))
        .unwrap();
    let same = options
        .update_option_if(
            "cron",
            Some(&b),
            HookValue::Str("b".into()),
            HookValue::Bool(true),
        )
        .unwrap();
    assert_eq!(same, ConditionalWrite::Unchanged);

    // Changed since the read (the old version), or expected absent while a row exists: a conflict.
    let stale = options
        .update_option_if(
            "cron",
            Some(&version),
            HookValue::Str("c".into()),
            HookValue::Bool(true),
        )
        .unwrap();
    assert_eq!(stale, ConditionalWrite::Conflict);
    let absent = options
        .update_option_if(
            "cron",
            None,
            HookValue::Str("c".into()),
            HookValue::Bool(true),
        )
        .unwrap();
    assert_eq!(absent, ConditionalWrite::Conflict);
}

#[test]
fn autoload_values_round_trip() {
    for text in [
        "on", "off", "auto", "auto-on", "auto-off", "yes", "no", "other",
    ] {
        assert_eq!(Autoload::parse(text).as_str(), text);
    }
    assert_eq!(
        AutoloadRequest::from_value(&HookValue::Null),
        AutoloadRequest::Default
    );
    assert_eq!(
        AutoloadRequest::from_value(&HookValue::Bool(false)),
        AutoloadRequest::Bool(false)
    );
}
