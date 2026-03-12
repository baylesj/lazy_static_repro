/// lazy_lock_array/src/main.rs
///
/// RUNTIME-COMPUTED Box<[f64]> with std::sync::LazyLock — correct, no bloat
///
/// This is the LazyLock equivalent of lazy_computed: values are computed at
/// runtime and heap-allocated via Box. The static holds only an 8-byte pointer.
///
/// WHY Box<[f64]> instead of [f64; N]:
///
/// LazyLock<[f64; N]> has two problems with large arrays:
///
///   1. STACK OVERFLOW: LazyLock::new(|| std::array::from_fn(...)) constructs
///      the [f64; N] return value on the current stack before moving it into
///      LazyLock's internal storage. At N=524288 that is 4 MB per table —
///      far beyond the default stack size. The program crashes on first access.
///      (lazy_static avoids this because it pre-allocates static storage and
///      the init closure writes directly into it via a hidden pointer.)
///
///   2. __DATA BLOAT: even if construction succeeded, LazyLock<[f64; N]>
///      internally holds a OnceLock<[f64; N]>, which reserves N*8 bytes of
///      static storage. Like lazy_static's MaybeUninit, this lands in __DATA
///      (file-backed) rather than __BSS, bloating the binary just as much as
///      embedding the values directly.
///
/// Box<[f64]> sidesteps both: the static stores only a pointer (8 bytes),
/// and heap allocation happens at runtime without touching the stack.
///
/// MEASURED RESULT: binary is ~0.43 MB — matches lazy_computed and
/// runtime_computed.

use std::f64::consts::PI;
use std::sync::LazyLock;

const N: usize = 524_288; // 2^19 — same as all other crates

static TABLE_1: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).sin()).collect());
static TABLE_2: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).cos()).collect());
static TABLE_3: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (4.0 * PI * i as f64 / N as f64).sin()).collect());
static TABLE_4: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (4.0 * PI * i as f64 / N as f64).cos()).collect());
static TABLE_5: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (6.0 * PI * i as f64 / N as f64).sin()).collect());
static TABLE_6: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (6.0 * PI * i as f64 / N as f64).cos()).collect());
static TABLE_7: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (8.0 * PI * i as f64 / N as f64).sin()).collect());
static TABLE_8: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (8.0 * PI * i as f64 / N as f64).cos()).collect());

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
