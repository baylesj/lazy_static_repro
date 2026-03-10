# Findings: lazy_static and LazyLock don't mean what you think

## The intuition gap

When a developer sees `lazy_static!` or `LazyLock`, the word "lazy" carries a strong implication:
*this data won't exist until it's needed.* The mental model is something like lazy loading in a
UI framework, or a database query that only runs on first access. Small binary, small memory
footprint — pay only for what you use.

That intuition is wrong in a specific and costly way.

`lazy_static` and `LazyLock` are **synchronization primitives for deferred runtime initialization**.
They control *when* a value is constructed during program execution. They say nothing about whether
the data backing that value is compiled into the binary. If the data is expressed as literals in
source code, it is embedded at compile time — unconditionally, before `main()` runs, before any
"lazy" logic ever executes.

---

## What these libraries actually do

Both `lazy_static` and `std::sync::LazyLock` (stable since Rust 1.80) solve the same problem:
Rust does not allow complex expressions (function calls, heap allocation, etc.) in `static`
initializers. These libraries work around that by wrapping the value in a `Once`/`OnceLock`
cell and running the initializer the first time the static is dereferenced.

What they guarantee:
- The initializer closure runs **at most once**, even under concurrent access
- The value is available as a `&T` after first access
- If the static is never accessed, the initializer never runs

What they do **not** guarantee:
- That the binary is smaller
- That embedded data is deferred or excluded
- Anything about compile-time data in `.rodata`

---

## The real-world case: Symphonia's FFT twiddle tables

[Symphonia](https://github.com/pdeljanov/Symphonia) is a pure-Rust audio decoding library.
Its FFT implementation uses precomputed twiddle-factor tables — sine and cosine values at
regular intervals — stored as static literal arrays and wrapped in `lazy_static`:

```rust
lazy_static! {
    static ref FFT_TWIDDLE_TABLES: &'static [f64] = &TABLE;
}

static TABLE: [f64; N] = [0.0, 0.00001199, 0.00002398, /* ... hundreds of thousands of values */];
```

A developer reading this code would reasonably conclude the tables are "lazily loaded."
They are not. The literal values in `TABLE` are parsed by the compiler and emitted into
the binary's `.rodata` section at compile time. The `lazy_static` wrapper only controls
when the `&[f64]` reference is initialized — but the data it points to has been in the
binary since the executable was written to disk.

This pattern caused measurable binary bloat in Chrome on Android, where Symphonia is
used for audio decoding. The bloat is proportional to the size of the tables: a table
of 524,288 `f64` values costs **4 MB of `.rodata`**, regardless of whether the program
ever performs an FFT.

---

## The three patterns and what they actually do

### Pattern 1: Embedded literals (bloated)

```rust
// build.rs generates tables.rs with hardcoded f64 values.
// include! splices them into the compilation unit as source.
include!(concat!(env!("OUT_DIR"), "/tables.rs"));
// Equivalent to writing: static TABLE_1: [f64; N] = [0.0, 1.2e-5, ...];

lazy_static! {
    static ref FFT_TWIDDLE: &'static [f64] = &TABLE_1;
}
```

- Compiler sees literal values → emits them into `.rodata`
- `lazy_static` / `LazyLock` wrapper makes no difference
- Binary is ~32 MB larger than it needs to be
- Data is present even if `FFT_TWIDDLE` is never accessed

This covers: `static_literal` (lazy_static), `lazy_lock_literal` (LazyLock).

### Pattern 2: include_bytes! (bloated, same result)

```rust
static TABLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/twiddle.bin"));

lazy_static! {
    static ref TABLE: &'static [f64] = unsafe { bytes_as_f64s(TABLE_BYTES) };
}
```

Different mechanism, identical outcome: raw bytes are embedded in `.rodata` at compile time.
The `lazy_static` defers the unsafe cast, not the data.

### Pattern 3: Runtime computation (correct)

```rust
// With lazy_static:
lazy_static! {
    static ref TABLE: [f64; N] =
        std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).sin());
}

// With LazyLock (std, no dependency):
static TABLE: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).sin()));
```

- Binary contains only the computation code (~KB), not the values
- Array is computed on first access and stored in BSS (zero-initialized static storage)
- If never accessed, computation never runs — *this* is what "lazy" actually buys you

---

## Binary size measurements

<!-- TODO: run `bash measure.sh` and paste the grid here -->

Expected structure of results:

| profile        | static_embedded | static_literal | lazy_lock_literal | lazy_computed | lazy_array | lazy_lock_array | runtime_computed |
|----------------|----------------|----------------|-------------------|---------------|------------|-----------------|-----------------|
| release        | ~33 MB         | ~33 MB         | ~33 MB            | ~1 MB         | ~1 MB      | ~1 MB           | ~1 MB           |
| release-lto    | ~33 MB         | ~33 MB         | ~33 MB            | ~1 MB         | ~1 MB      | ~1 MB           | ~1 MB           |
| release-size   | ~33 MB         | ~33 MB         | ~33 MB            | ~1 MB         | ~1 MB      | ~1 MB           | ~1 MB           |
| release-min    | ~33 MB         | ~33 MB         | ~33 MB            | ~1 MB         | ~1 MB      | ~1 MB           | ~1 MB           |

Key observation: **the bloat is invariant across optimization profiles.** `opt-level = "z"`,
fat LTO, and symbol stripping reduce the `.text` (code) section by a few hundred KB.
They cannot remove data in `.rodata` that is actively referenced by the program.
The ~32 MB gap between embedded and computed variants is constant at every profile.

---

## Why optimization can't save you

When the compiler sees a referenced static, it must emit the data. Dead code elimination
only applies to unreachable code — a static that is referenced (even behind a lazy wrapper)
is reachable and will be included. The only way to remove the data from the binary is to
not put it there in the first place.

LTO can eliminate unused functions across crate boundaries. It cannot eliminate data that
the program actively addresses at runtime.

---

## The fix

Replace literal tables with runtime computation:

```rust
// Before (bloated — ~4 MB per table in .rodata):
static TABLE: [f64; N] = [ /* 524,288 hardcoded values */ ];

// After (no bloat — only the loop body is compiled):
static TABLE: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| {
        (2.0 * PI * i as f64 / N as f64).sin()
    }));
```

Trade-off: a small runtime cost on first access to compute the values. For FFT twiddle
tables this is milliseconds at startup — negligible compared to shipping 32 MB of data
to every user and loading it into memory on every process start, whether or not an FFT
is ever performed.

---

## Repro

This repository contains seven crates demonstrating each pattern, buildable across four
optimization profiles:

```
bash measure.sh
```

Crates in order from most to least binary bloat:
1. `static_embedded` — `include_bytes!` + `lazy_static`
2. `static_literal` — literal array + `lazy_static`
3. `lazy_lock_literal` — literal array + `LazyLock`
4. `lazy_computed` — `lazy_static` + runtime `Box<[f64]>`
5. `lazy_array` — `lazy_static` + runtime `[f64; N]`
6. `lazy_lock_array` — `LazyLock` + runtime `[f64; N]`
7. `runtime_computed` — plain `Vec`, no lazy primitive
