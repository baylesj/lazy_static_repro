/// static_embedded/src/main.rs
///
/// THE BLOATED VERSION
///
/// Each twiddle table was pre-computed at BUILD TIME by build.rs and written
/// to OUT_DIR as a raw binary file. `include_bytes!` splices those bytes
/// directly into the binary's .rodata section at compile time.
///
/// The result: ~32 MB of twiddle-factor data lives permanently inside the
/// executable, whether or not it's ever used. The binary is shipped to every
/// user, loaded into memory on every process start, and competes with your
/// actual code in the CPU's instruction and data caches.
///
/// lazy_static here provides a safe &[f64] view over the embedded bytes.
/// It does NOT reduce binary size — the bytes are already compiled in.
///
/// Section breakdown you'll see after building:
///   __TEXT / .text      ~200 KB   (actual code)
///   __DATA / .rodata    ~32 MB    ← all eight tables live here

use std::mem;

// Safety helper: reinterpret a &[u8] as &[f64].
// build.rs writes native-endian f64 bytes, so this is valid as long as
// the pointer is f64-aligned. The linker guarantees that static byte
// arrays are aligned to at least 8 bytes on all common targets.
unsafe fn bytes_as_f64s(b: &'static [u8]) -> &'static [f64] {
    debug_assert!(
        b.as_ptr() as usize % mem::align_of::<f64>() == 0,
        "alignment violated"
    );
    std::slice::from_raw_parts(b.as_ptr() as *const f64, b.len() / mem::size_of::<f64>())
}

lazy_static::lazy_static! {
    // include_bytes! is a compiler built-in that accepts a concat!() expression
    // as its argument — no intermediate macro needed.
    // Each call embeds a 4 MB blob of pre-computed f64 data into .rodata.

    static ref TABLE_1: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_1.bin")))
    }; // sin(2π·i/N)

    static ref TABLE_2: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_2.bin")))
    }; // cos(2π·i/N)

    static ref TABLE_3: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_3.bin")))
    }; // sin(4π·i/N)

    static ref TABLE_4: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_4.bin")))
    }; // cos(4π·i/N)

    static ref TABLE_5: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_5.bin")))
    }; // sin(6π·i/N)

    static ref TABLE_6: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_6.bin")))
    }; // cos(6π·i/N)

    static ref TABLE_7: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_7.bin")))
    }; // sin(8π·i/N)

    static ref TABLE_8: &'static [f64] = unsafe {
        bytes_as_f64s(include_bytes!(concat!(env!("OUT_DIR"), "/twiddle_8.bin")))
    }; // cos(8π·i/N)
}

fn main() {
    // Touch every table so nothing is dead-code-eliminated.
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
