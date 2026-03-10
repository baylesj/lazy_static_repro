/// lazy_array/src/main.rs
///
/// RUNTIME-COMPUTED, ARRAY-TYPED lazy_static — the Symphonia shape without the bloat
///
/// This variant matches the TYPE SIGNATURE of the Symphonia pattern:
///   static ref TABLE: [f64; N] = ...
///
/// The key difference from static_literal / static_embedded:
///   - The VALUES are computed at runtime via std::array::from_fn, not embedded
///   - The binary contains only computation CODE (~a few KB), zero table data
///   - lazy_static stores the array in BSS (zero-initialized, not in the file)
///     and fills it in on first access
///
/// Compare with lazy_computed (Box<[f64]>):
///   - Box stores an 8-byte pointer in the static; data lives on the heap
///   - [f64; N] stores the array inline in the static; data lives in BSS
///   - Either way: NOT in .rodata, NOT in the binary file on disk
///
/// This demonstrates that the bloat in static_literal comes from the LITERAL
/// VALUES in source, not from the use of lazy_static or the array type itself.

use std::f64::consts::PI;

const N: usize = 524_288; // 2^19 — same as all other crates

lazy_static::lazy_static! {
    // std::array::from_fn computes each element at runtime on first access.
    // The resulting [f64; N] is stored in the lazy_static's BSS-backed storage,
    // not in .rodata. Binary size is unaffected.
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
