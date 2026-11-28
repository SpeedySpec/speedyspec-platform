//! The PHP casts an option's value goes through: `(string)` and `empty()`.

use speedyspec_wp_hooks_contracts::HookValue;

/// PHP's `(string)` cast; `None` for arrays and objects, which have none here.
pub fn php_string(value: &HookValue) -> Option<Vec<u8>> {
    match value {
        HookValue::Null | HookValue::Bool(false) => Some(Vec::new()),
        HookValue::Bool(true) => Some(b"1".to_vec()),
        HookValue::Int(i) => Some(i.to_string().into_bytes()),
        HookValue::Float(f) => Some(float_string(*f).into_bytes()),
        HookValue::Str(s) => Some(s.as_bytes().to_vec()),
        HookValue::Bytes(b) => Some(b.clone()),
        _ => None,
    }
}

/// A float as PHP's string cast writes it (`precision = 14`): 14 significant digits, trailing zeros dropped, and
/// the exponent form outside decimal exponents -4 to 13.
fn float_string(f: f64) -> String {
    if f.is_nan() {
        return "NAN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "INF".into() } else { "-INF".into() };
    }
    if f == 0.0 {
        return if f.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    let sci = format!("{:.13e}", f.abs());
    let (mantissa, exp) = sci.split_once('e').expect("{:e} has an exponent");
    let exp: i32 = exp.parse().expect("an integer exponent");
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let n = digits.len() as i32;
    let sign = if f < 0.0 { "-" } else { "" };
    if (-4..14).contains(&exp) {
        let body = if exp >= 0 {
            if n > exp + 1 {
                format!(
                    "{}.{}",
                    &digits[..(exp + 1) as usize],
                    &digits[(exp + 1) as usize..]
                )
            } else {
                format!("{digits}{}", "0".repeat((exp + 1 - n) as usize))
            }
        } else {
            format!("0.{}{digits}", "0".repeat((-exp - 1) as usize))
        };
        format!("{sign}{body}")
    } else {
        let frac = if n > 1 { &digits[1..] } else { "0" };
        format!(
            "{sign}{}.{frac}E{}{}",
            &digits[..1],
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

/// PHP's `empty()`.
pub fn php_empty(value: &HookValue) -> bool {
    match value {
        HookValue::Null | HookValue::Bool(false) | HookValue::Int(0) => true,
        HookValue::Float(f) => *f == 0.0,
        HookValue::Str(s) => s.is_empty() || s == "0",
        HookValue::Bytes(b) => b.is_empty() || b == b"0",
        HookValue::Array(items) => items.is_empty(),
        HookValue::Opaque(_) => match value.read() {
            std::borrow::Cow::Owned(read) if !matches!(read, HookValue::Opaque(_)) => {
                php_empty(&read)
            }
            std::borrow::Cow::Owned(_) => false,
            std::borrow::Cow::Borrowed(_) => false,
        },
        _ => false,
    }
}
