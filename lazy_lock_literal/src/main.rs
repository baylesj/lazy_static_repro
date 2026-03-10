/// lazy_lock_literal/src/main.rs
///
/// THE SYMPHONIA PATTERN — but with std::sync::LazyLock instead of lazy_static
///
/// Identical data to static_literal: build.rs generates Rust source with
/// hardcoded f64 literals, included via include!. The only change is using
/// the std-native LazyLock (stable since Rust 1.80) instead of lazy_static.
///
/// Result: still ~32 MB of .rodata bloat. Switching from lazy_static to
/// LazyLock does not fix the problem — the embedded data comes from the
/// literal values in source, not from the choice of lazy primitive.

use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

static FFT_TWIDDLE_1: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_1);
static FFT_TWIDDLE_2: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_2);
static FFT_TWIDDLE_3: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_3);
static FFT_TWIDDLE_4: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_4);
static FFT_TWIDDLE_5: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_5);
static FFT_TWIDDLE_6: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_6);
static FFT_TWIDDLE_7: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_7);
static FFT_TWIDDLE_8: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE_8);

fn main() {
    let sum = FFT_TWIDDLE_1[0]
        + FFT_TWIDDLE_2[0]
        + FFT_TWIDDLE_3[0]
        + FFT_TWIDDLE_4[0]
        + FFT_TWIDDLE_5[0]
        + FFT_TWIDDLE_6[0]
        + FFT_TWIDDLE_7[0]
        + FFT_TWIDDLE_8[0];

    println!("sum of first elements: {sum}");
    println!("each table has {} entries", FFT_TWIDDLE_1.len());
}
