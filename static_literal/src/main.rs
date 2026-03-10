/// static_literal/src/main.rs
///
/// THE SYMPHONIA PATTERN — literal static arrays + lazy_static references
///
/// build.rs generates tables.rs, which contains 8 arrays of hardcoded f64
/// literals. `include!` splices that source directly into this compilation
/// unit, as if you had typed all 4 million values by hand — which is exactly
/// what real-world crates like Symphonia do (hand-authored or codegen'd).
///
/// `lazy_static` then wraps each static as a &[f64] — matching the API shape
/// of FFT_TWIDDLE_TABLES in Symphonia. This is the critical misconception:
///
///   Developers assume lazy_static = "lazy loading" = small binary.
///   Reality: lazy_static only defers reference initialization.
///             The literal bytes are already in .rodata before main() runs.
///
/// Binary size will match static_embedded (~32 MB .rodata) because the
/// compiler emits the same data regardless of whether it came from
/// include_bytes! or from parsed f64 literals.

// Pull in the generated source: defines TABLE_1 … TABLE_8 as [f64; N] statics.
include!(concat!(env!("OUT_DIR"), "/tables.rs"));

lazy_static::lazy_static! {
    // These references look cheap — and at runtime they are.
    // But the data they point to is permanently resident in .rodata.
    static ref FFT_TWIDDLE_1: &'static [f64] = &TABLE_1; // sin(2π·i/N)
    static ref FFT_TWIDDLE_2: &'static [f64] = &TABLE_2; // cos(2π·i/N)
    static ref FFT_TWIDDLE_3: &'static [f64] = &TABLE_3; // sin(4π·i/N)
    static ref FFT_TWIDDLE_4: &'static [f64] = &TABLE_4; // cos(4π·i/N)
    static ref FFT_TWIDDLE_5: &'static [f64] = &TABLE_5; // sin(6π·i/N)
    static ref FFT_TWIDDLE_6: &'static [f64] = &TABLE_6; // cos(6π·i/N)
    static ref FFT_TWIDDLE_7: &'static [f64] = &TABLE_7; // sin(8π·i/N)
    static ref FFT_TWIDDLE_8: &'static [f64] = &TABLE_8; // cos(8π·i/N)
}

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
