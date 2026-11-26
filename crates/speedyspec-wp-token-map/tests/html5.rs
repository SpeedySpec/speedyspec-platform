//! The HTML5 named character references: the lookup the HTML/kses port uses. The tests that check the table against
//! WordPress's `entities.json` live in speedyspec-server, beside that data.

use speedyspec_wp_token_map::named_reference;

#[test]
fn kses_cases() {
    assert_eq!(named_reference(b"amp;x", 0), Some((&b"&"[..], 4)));
    assert_eq!(named_reference(b"ampx", 0), Some((&b"&"[..], 3)));
    assert_eq!(named_reference(b"notin;", 0), Some(("∉".as_bytes(), 6)));
    assert_eq!(named_reference(b"zzz", 0), None);
    assert_eq!(named_reference(b"&lt;", 1), Some((&b"<"[..], 3)));
    assert_eq!(named_reference(b"a&", 2), None);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing needs a release build")]
fn named_reference_is_fast() {
    let names: [&[u8]; 10] = [
        b"amp;", b"lt;", b"gt;", b"quot;", b"nbsp;", b"copy;", b"hellip;", b"notin;", b"mdash;",
        b"zzz",
    ];
    let start = std::time::Instant::now();
    let mut found = 0;
    for _ in 0..10_000 {
        for name in names {
            found += usize::from(named_reference(std::hint::black_box(name), 0).is_some());
        }
    }
    let took = start.elapsed();
    assert_eq!(found, 90_000);
    // 100,000 lookups: well under a microsecond each.
    assert!(took.as_millis() < 30, "100,000 lookups took {took:?}");
}
