/// build.rs for static_literal
///
/// Generates a Rust source file (tables.rs) containing 8 twiddle-factor tables
/// as hardcoded f64 LITERAL ARRAYS — the exact pattern used in Symphonia's
/// FFT_TWIDDLE_TABLES and similar real-world crates.
///
/// This is the key difference from static_embedded:
///   static_embedded  → `include_bytes!` embeds raw binary bytes into .rodata
///   static_literal   → `include!` embeds Rust source with float literals;
///                       the compiler parses each literal and emits the same
///                       data into .rodata. Binary bloat is identical.
///
/// The generated file looks like:
///   pub static TABLE_1: [f64; 524288] = [0.0, 1.2e-5, …];
///   pub static TABLE_2: [f64; 524288] = [1.0, 9.9e-1, …];
///   …
///
/// `lazy_static` in main.rs then wraps these statics as &[f64] references —
/// exactly mirroring how Symphonia exposes its tables. The lazy_static does
/// NOT prevent the literal data from being baked into the binary; it only
/// controls when the reference wrapper is initialized.

use std::f64::consts::PI;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const N: usize = 524_288; // 2^19 — must match the other crates

fn main() {
    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    println!("cargo:rerun-if-changed=build.rs");

    let path = out_dir.join("tables.rs");
    let file = std::fs::File::create(&path)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
    let mut w = BufWriter::new(file);

    let configs: &[(u32, bool)] = &[
        (1, true),  // sin(2π·i/N)
        (1, false), // cos(2π·i/N)
        (2, true),  // sin(4π·i/N)
        (2, false), // cos(4π·i/N)
        (3, true),  // sin(6π·i/N)
        (3, false), // cos(6π·i/N)
        (4, true),  // sin(8π·i/N)
        (4, false), // cos(8π·i/N)
    ];

    for (idx, &(harmonic, use_sin)) in configs.iter().enumerate() {
        write!(w, "pub static TABLE_{}: [f64; {N}] = [", idx + 1).unwrap();
        for i in 0..N {
            let angle = harmonic as f64 * 2.0 * PI * i as f64 / N as f64;
            let val: f64 = if use_sin { angle.sin() } else { angle.cos() };
            // {:?} produces the shortest round-trip-exact decimal representation.
            write!(w, "{val:?},").unwrap();
        }
        writeln!(w, "];").unwrap();

        eprintln!(
            "build.rs: wrote TABLE_{} ({} entries, ~{:.1} MB of source text)",
            idx + 1,
            N,
            // rough estimate: avg ~18 chars per literal
            (N * 18) as f64 / 1_048_576.0
        );
    }
}
