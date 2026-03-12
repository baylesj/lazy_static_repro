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

And as our measurements show, even with runtime computation, the *type* you store in the lazy
static determines whether the binary is small. This is the second trap developers fall into after
switching away from literals.

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
- Anything about compile-time data in `.rodata` or `.data`

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

## The patterns, what we expected, and what we measured

We built seven crates across four optimization profiles. All use 8 twiddle-factor tables
of 524,288 `f64` values (4 MB each, 32 MB total if embedded).

### Pattern 1: Embedded literals — bloated (expected)

```rust
// Hardcoded f64 values in source → compiler emits them into .rodata
static TABLE: [f64; N] = [0.0, 1.2e-5, /* ... */];

// With lazy_static:
lazy_static! { static ref T: &'static [f64] = &TABLE; }

// With LazyLock:
static T: LazyLock<&'static [f64]> = LazyLock::new(|| &TABLE);
```

Both `lazy_static` and `LazyLock` wrappers produce identical binary size. The choice of
primitive is irrelevant — the literals are already in `.rodata` before any lazy logic runs.

Crates: `static_literal` (lazy_static), `lazy_lock_literal` (LazyLock), `static_embedded` (include_bytes!).

### Pattern 2: Runtime computation, inline array type — bloated (unexpected)

```rust
// Values computed at runtime — no literals, should be small. Right?
lazy_static! {
    static ref TABLE: [f64; N] =
        std::array::from_fn(|i| (2.0 * PI * i as f64 / N as f64).sin());
}
```

**Measured result: ~32.67 MB — identical to the literal variants.**

`lazy_static` internally stores values in a `static UnsafeCell<MaybeUninit<T>>`. For
`T = [f64; N]`, that reserves N×8 bytes of static storage per table. `MaybeUninit::uninit()`
compiles to `undef` in LLVM IR — not a zero initializer — so the linker places it in
`__DATA` (file-backed initialized data) rather than `__BSS` (zero-initialized, not stored
in the file). The binary carries 32 MB of uninitialized storage on disk even though the
values are computed at runtime.

There is a second problem with `LazyLock<[f64; N]>`: the closure passed to `LazyLock::new`
must return the array by value. The calling convention requires the 4 MB array to be
constructed on the stack before being moved into `LazyLock`'s internal storage — which
immediately overflows the default stack. The program crashes on first access.

Crate: `lazy_array` (lazy_static, bloated but functional), `lazy_lock_array` (LazyLock,
crashes with stack overflow — fixed in repo to use `Box<[f64]>`).

### Pattern 3: Runtime computation, heap-allocated — correct

```rust
// lazy_static with Box:
lazy_static! {
    static ref TABLE: Box<[f64]> =
        (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).sin()).collect();
}

// LazyLock with Box (std, no dependency):
static TABLE: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| (2.0 * PI * i as f64 / N as f64).sin()).collect());
```

The static holds only an 8-byte pointer. The 32 MB of table data is heap-allocated on
first access and never appears in the binary file. Binary is ~0.43 MB.

Crates: `lazy_computed` (lazy_static + Box), `lazy_lock_array` after fix (LazyLock + Box),
`runtime_computed` (plain Vec, no lazy primitive).

---

## Binary size measurements (release profile, macOS)

| crate              | primitive   | value source     | type       | binary size |
|--------------------|-------------|------------------|------------|-------------|
| `static_embedded`  | lazy_static | include_bytes!   | &[f64]     | 32.67 MB    |
| `static_literal`   | lazy_static | f64 literals     | [f64; N]   | 32.67 MB    |
| `lazy_lock_literal`| LazyLock    | f64 literals     | [f64; N]   | 32.67 MB    |
| `lazy_array`       | lazy_static | runtime computed | [f64; N]   | 32.67 MB    |
| `lazy_computed`    | lazy_static | runtime computed | Box<[f64]> | 0.43 MB     |
| `lazy_lock_array`  | LazyLock    | runtime computed | Box<[f64]> | 0.43 MB     |
| `runtime_computed` | none        | runtime computed | Vec<f64>   | 0.42 MB     |

The dividing line is not *lazy primitive* (lazy_static vs LazyLock) and not *value source*
(literals vs computed). It is the **storage type**:

- Inline array `[f64; N]` → static storage in `__DATA` → large binary file
- `Box<[f64]>` / `Vec<f64>` → 8-byte pointer in static → small binary file

---

## Optimization profiles don't help

Full grid across four profiles (MB on disk):

| profile        | static_embedded | static_literal | lazy_lock_literal | lazy_array | lazy_computed | lazy_lock_array | runtime_computed |
|----------------|-----------------|----------------|-------------------|------------|---------------|-----------------|------------------|
| release        | 32.67 MB        | 32.67 MB       | 32.67 MB          | 32.67 MB   | 0.43 MB       | 0.43 MB         | 0.42 MB          |
| release-lto    | 32.61 MB        | 32.61 MB       | 32.61 MB          | 32.62 MB   | 0.37 MB       | 0.37 MB         | 0.36 MB          |
| release-size   | 32.60 MB        | 32.60 MB       | 32.60 MB          | 32.60 MB   | 0.35 MB       | 0.35 MB         | 0.35 MB          |
| release-min    | 32.54 MB        | 32.54 MB       | 32.54 MB          | 32.54 MB   | 0.30 MB       | 0.30 MB         | 0.29 MB          |

`opt-level = "z"`, fat LTO, and symbol stripping each shave a few hundred KB off the code
section. They cannot remove data in `__DATA` or `.rodata` that is actively referenced. The
~32 MB gap is constant at every profile — optimization is not a workaround.

---

## Why optimization can't save you

When the compiler sees a referenced static, it must emit the backing storage. Dead code
elimination only applies to unreachable code paths. A static that is dereferenced anywhere
in the program — even behind a lazy wrapper — will have its storage included in the binary.

LTO can eliminate unused functions across crate boundaries. It cannot eliminate data that
the program actively addresses at runtime, whether that data is literal values in `.rodata`
or `MaybeUninit` storage in `__DATA`.

---

## The fix

Replace inline array types with heap-allocated types:

```rust
// Before — two problems: large binary + stack overflow with LazyLock:
static TABLE: LazyLock<[f64; N]> =
    LazyLock::new(|| std::array::from_fn(|i| compute(i)));

// Also before — large binary despite runtime computation:
lazy_static! {
    static ref TABLE: [f64; N] = std::array::from_fn(|i| compute(i));
}

// After — small binary, no stack overflow, works with either primitive:
static TABLE: LazyLock<Box<[f64]>> =
    LazyLock::new(|| (0..N).map(|i| compute(i)).collect());

lazy_static! {
    static ref TABLE: Box<[f64]> = (0..N).map(|i| compute(i)).collect();
}
```

Trade-off: a small one-time runtime cost to compute the values on first access. For FFT
twiddle tables this is milliseconds at startup — negligible compared to shipping 32 MB of
data to every user and loading it into memory on every process start, whether or not an
FFT is ever performed.

---

## Summary

Three independent variables interact here:

| variable            | effect on binary size                              |
|---------------------|----------------------------------------------------|
| lazy primitive      | none — lazy_static and LazyLock are identical      |
| value source        | literals → `.rodata` bloat; computed → no `.rodata` bloat |
| storage type        | `[T; N]` → `__DATA` bloat; `Box`/`Vec` → no bloat |

The only safe combination is: **runtime-computed values + heap-allocated storage type.**

---

## Repro

```
git clone https://github.com/baylesj/lazy_static_repro
cd lazy_static_repro
bash measure.sh
```
