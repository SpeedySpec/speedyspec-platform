//! Benchmarks: each SIMD scan against its scalar twin. `wp_text_diff()` on the recorded cases is benchmarked in
//! speedyspec-server, beside the cases.

use criterion::{Criterion, criterion_group, criterion_main};
use speedyspec_wp_text_diff::{html_special_chars, simd, split_words};
use std::hint::black_box;

fn text() -> Vec<u8> {
    let mut s = Vec::new();
    for i in 0..2000 {
        s.extend_from_slice(b"The quick brown fox jumps over the lazy dog, again and again ");
        if i % 50 == 0 {
            s.extend_from_slice(b"<b>&amp;</b> ");
        }
    }
    s
}

fn scans(c: &mut Criterion) {
    let s = text();
    // 4 KB with nothing to find: each scan reads all of it.
    let clean = b"The quick brown fox jumps over the lazy dog again ".repeat(80);
    c.bench_function("find_escaped/simd", |b| {
        b.iter(|| simd::find_escaped(black_box(&clean)))
    });
    c.bench_function("find_escaped/scalar", |b| {
        b.iter(|| simd::scalar::find_escaped(black_box(&clean)))
    });
    let words =
        b"Supercalifragilisticexpialidocious_and_more_words_without_spaces_0123456789".repeat(54);
    c.bench_function("ascii_word_run/simd", |b| {
        b.iter(|| simd::ascii_word_run(black_box(&words)))
    });
    c.bench_function("ascii_word_run/scalar", |b| {
        b.iter(|| simd::scalar::ascii_word_run(black_box(&words)))
    });
    c.bench_function("html_special_chars", |b| {
        b.iter(|| html_special_chars(black_box(&s)))
    });
    c.bench_function("split_words", |b| {
        b.iter(|| split_words(black_box(&s)).map(|w| w.pieces().len()))
    });
}

criterion_group!(benches, scans);
criterion_main!(benches);
