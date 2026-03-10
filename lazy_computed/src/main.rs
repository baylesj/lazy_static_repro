/// lazy_computed/src/main.rs
///
/// THE CORRECT LAZY VERSION
///
/// Same 8 twiddle tables, same math, but computed at RUNTIME the first time
/// each table is accessed. The binary contains only the computation code
/// (~a few KB) — zero table data is embedded.
///
/// Each lazy_static initialization closure is its own function in .text.
/// With 8 tables, you get 8 separate initialization functions. This does add
/// some code overhead compared to a single shared helper, but it's in the
/// range of tens of kilobytes, not megabytes.
///
/// The data lives on the heap and is allocated only when first accessed.
/// If a table is never used, it's never computed.

use std::f64::consts::PI;

const N: usize = 524_288; // 2^19 — must match build.rs in static_embedded

lazy_static::lazy_static! {
    // Each closure is compiled as its own function in .text.
    // With codegen-units=16 (the default), these end up in separate codegen
    // units, so LLVM cannot merge or deduplicate them across units.
    static ref TABLE_1: Box<[f64]> = {
        (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).sin()).collect()
    };
    static ref TABLE_2: Box<[f64]> = {
        (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).cos()).collect()
    };
    static ref TABLE_3: Box<[f64]> = {
        (0..N).map(|i| (4.0 * PI * i as f64 / N as f64).sin()).collect()
    };
    static ref TABLE_4: Box<[f64]> = {
        (0..N).map(|i| (4.0 * PI * i as f64 / N as f64).cos()).collect()
    };
    static ref TABLE_5: Box<[f64]> = {
        (0..N).map(|i| (6.0 * PI * i as f64 / N as f64).sin()).collect()
    };
    static ref TABLE_6: Box<[f64]> = {
        (0..N).map(|i| (6.0 * PI * i as f64 / N as f64).cos()).collect()
    };
    static ref TABLE_7: Box<[f64]> = {
        (0..N).map(|i| (8.0 * PI * i as f64 / N as f64).sin()).collect()
    };
    static ref TABLE_8: Box<[f64]> = {
        (0..N).map(|i| (8.0 * PI * i as f64 / N as f64).cos()).collect()
    };
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
