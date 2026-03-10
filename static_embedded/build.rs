/// build.rs for static_embedded
///
/// Pre-computes 8 twiddle-factor tables at BUILD TIME and writes them as raw
/// little-endian f64 binary files into OUT_DIR. The main crate then uses
/// `include_bytes!` to bake every byte into the binary's .rodata section.
///
/// This is the pattern that causes massive binary bloat: the compiler doesn't
/// know these bytes are twiddle factors — it just sees a giant byte literal and
/// dutifully emits it all into the executable.

use std::f64::consts::PI;
use std::io::Write;
use std::path::PathBuf;

/// Number of f64 entries per table. 524_288 × 8 bytes = 4 MB per table.
/// Eight tables → ~32 MB baked into .rodata.
/// (Use a power-of-two so the FFT framing is realistic.)
const N: usize = 524_288; // 2^19

fn main() {
    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();

    // Regenerate whenever this file changes.
    println!("cargo:rerun-if-changed=build.rs");

    let configs: &[(u32, bool)] = &[
        (1, true),  // sin(2π·i/N)  — forward FFT, imaginary twiddles
        (1, false), // cos(2π·i/N)  — forward FFT, real twiddles
        (2, true),  // sin(4π·i/N)  — 2nd harmonic
        (2, false), // cos(4π·i/N)
        (3, true),  // sin(6π·i/N)  — 3rd harmonic
        (3, false), // cos(6π·i/N)
        (4, true),  // sin(8π·i/N)  — 4th harmonic
        (4, false), // cos(8π·i/N)
    ];

    for (idx, &(harmonic, use_sin)) in configs.iter().enumerate() {
        let path = out_dir.join(format!("twiddle_{}.bin", idx + 1));
        let mut file = std::fs::File::create(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));

        for i in 0..N {
            let angle = harmonic as f64 * 2.0 * PI * i as f64 / N as f64;
            let val: f64 = if use_sin { angle.sin() } else { angle.cos() };
            file.write_all(&val.to_le_bytes())
                .expect("write failed");
        }

        eprintln!(
            "build.rs: wrote {} ({:.1} MB)",
            path.display(),
            (N * 8) as f64 / 1_048_576.0
        );
    }
}
