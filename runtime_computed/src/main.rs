/// runtime_computed/src/main.rs
///
/// THE BASELINE: no lazy_static, plain runtime allocation
///
/// Allocates and computes all 8 twiddle tables directly in main().
/// Uses a single shared helper function so the compiler only needs ONE copy
/// of the loop body, regardless of how many tables we create.
///
/// This is the tightest possible binary: one loop body, called 8 times with
/// different parameters. LLVM can't inline it (due to #[inline(never)]) and
/// won't duplicate it.

use std::f64::consts::PI;

const N: usize = 524_288; // 2^19 — same as the other two crates

/// Compute a single twiddle-factor table.
///
/// `freq_mult` — frequency multiplier (1 = 2π/N, 2 = 4π/N, …)
/// `use_sin`   — sin if true, cos if false
///
/// Marked `#[inline(never)]` so the compiler emits ONE copy of this function
/// rather than inlining 8 separate copies into main().
#[inline(never)]
fn compute_twiddle_table(freq_mult: f64, use_sin: bool) -> Vec<f64> {
    (0..N)
        .map(|i| {
            let angle = freq_mult * 2.0 * PI * i as f64 / N as f64;
            if use_sin { angle.sin() } else { angle.cos() }
        })
        .collect()
}

fn main() {
    let table_1 = compute_twiddle_table(1.0, true);  // sin(2π·i/N)
    let table_2 = compute_twiddle_table(1.0, false); // cos(2π·i/N)
    let table_3 = compute_twiddle_table(2.0, true);  // sin(4π·i/N)
    let table_4 = compute_twiddle_table(2.0, false); // cos(4π·i/N)
    let table_5 = compute_twiddle_table(3.0, true);  // sin(6π·i/N)
    let table_6 = compute_twiddle_table(3.0, false); // cos(6π·i/N)
    let table_7 = compute_twiddle_table(4.0, true);  // sin(8π·i/N)
    let table_8 = compute_twiddle_table(4.0, false); // cos(8π·i/N)

    // Touch every table to prevent DCE.
    let sum = table_1[0]
        + table_2[0]
        + table_3[0]
        + table_4[0]
        + table_5[0]
        + table_6[0]
        + table_7[0]
        + table_8[0];

    println!("sum of first elements: {sum}");
    println!("each table has {} entries", table_1.len());
}
