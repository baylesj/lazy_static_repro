/// lazy_array/src/main.rs
///
/// RUNTIME-COMPUTED, ARRAY-TYPED lazy_static — an unexpected source of bloat
///
/// This variant uses the same [f64; N] type as static_literal, but computes
/// the values at runtime via std::array::from_fn rather than embedding them
/// as literals. You might expect this to produce a small binary. It does not.
///
/// MEASURED RESULT: binary is ~32.67 MB — identical to the bloated variants.
///
/// WHY: lazy_static internally stores values in a
///   static UnsafeCell<MaybeUninit<T>>
/// For T = [f64; N], that is 4 MB of MaybeUninit per table, 32 MB total.
/// MaybeUninit::uninit() is represented as `undef` in LLVM IR — not a
/// zeroinitializer — so the linker places it in __DATA (file-backed initialized
/// data) rather than __BSS (zero-initialized, not stored in the file).
/// The binary must carry 32 MB of "uninitialized" storage, every byte of which
/// is physically present on disk.
///
/// Contrast with lazy_computed (Box<[f64]>):
///   The static only holds an 8-byte pointer. The 4 MB arrays are heap-
///   allocated on first access and never appear in the binary file at all.
///
/// LESSON: runtime computation is not sufficient to avoid binary bloat.
///         The TYPE stored in the lazy static matters too.
///         Inline arrays ([T; N]) → 32 MB in __DATA → large binary file.
///         Box/Vec               → 8-byte pointer in static → small binary.

use std::f64::consts::PI;

const N: usize = 524_288; // 2^19 — same as all other crates

lazy_static::lazy_static! {
    static ref TABLE_1: [f64; N] =
        std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).sin());
    static ref TABLE_2: [f64; N] =
        std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).cos());
    static ref TABLE_3: [f64; N] =
        std::array::from_fn(|i| (4.0 * PI * i as f64 / N as f64).sin());
    static ref TABLE_4: [f64; N] =
        std::array::from_fn(|i| (4.0 * PI * i as f64 / N as f64).cos());
    static ref TABLE_5: [f64; N] =
        std::array::from_fn(|i| (6.0 * PI * i as f64 / N as f64).sin());
    static ref TABLE_6: [f64; N] =
        std::array::from_fn(|i| (6.0 * PI * i as f64 / N as f64).cos());
    static ref TABLE_7: [f64; N] =
        std::array::from_fn(|i| (8.0 * PI * i as f64 / N as f64).sin());
    static ref TABLE_8: [f64; N] =
        std::array::from_fn(|i| (8.0 * PI * i as f64 / N as f64).cos());
}

fn main() {
    let sum = TABLE_1[0]
        + TABLE_2[0]
        + TABLE_3[0]
        + TABLE_4[0]
        + TABLE_5[0]
        + TABLE_6[0]
        + TABLE_7[0]
        + TABLE_8[0];

    println!("sum of first elements: {sum}");
    println!("each table has {} entries", TABLE_1.len());
}
