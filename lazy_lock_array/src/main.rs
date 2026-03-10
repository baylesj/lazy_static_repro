/// lazy_lock_array/src/main.rs
///
/// RUNTIME-COMPUTED [f64; N] with std::sync::LazyLock — no bloat
///
/// Same type shape as lazy_lock_literal ([f64; N] behind LazyLock) but
/// values are computed at runtime via std::array::from_fn. No data is
/// embedded in the binary; LazyLock fills the BSS-backed storage on first
/// access, just like lazy_array does with lazy_static.

use std::f64::consts::PI;
use std::sync::LazyLock;

const N: usize = 524_288; // 2^19 — same as all other crates

static TABLE_1: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).sin()));
static TABLE_2: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).cos()));
static TABLE_3: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (4.0 * PI * i as f64 / N as f64).sin()));
static TABLE_4: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (4.0 * PI * i as f64 / N as f64).cos()));
static TABLE_5: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (6.0 * PI * i as f64 / N as f64).sin()));
static TABLE_6: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (6.0 * PI * i as f64 / N as f64).cos()));
static TABLE_7: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (8.0 * PI * i as f64 / N as f64).sin()));
static TABLE_8: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (8.0 * PI * i as f64 / N as f64).cos()));

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
