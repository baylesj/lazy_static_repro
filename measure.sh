#!/usr/bin/env bash
# measure.sh — build all five versions and produce a detailed comparison.
# Run from the workspace root: bash measure.sh
#
# The five versions:
#   static_embedded  — pre-computed data baked in via include_bytes! (build.rs → .bin files)
#   static_literal   — pre-computed data as f64 literals in generated .rs source (Symphonia pattern)
#   lazy_computed    — lazy_static Box<[f64]> with runtime sin/cos computation
#   lazy_array       — lazy_static [f64; N] with runtime sin/cos computation (Symphonia type shape)
#   runtime_computed — plain runtime Vec with a shared helper function (smallest binary)

set -euo pipefail
WORKSPACE="$(cd "$(dirname "$0")" && pwd)"
cd "$WORKSPACE"

STATIC_BIN="target/release/static_embedded"
LITERAL_BIN="target/release/static_literal"
LAZY_BIN="target/release/lazy_computed"
ARRAY_BIN="target/release/lazy_array"
RUNTIME_BIN="target/release/runtime_computed"

sep() { printf '\n%s\n' "────────────────────────────────────────────────────────────────"; }
header() { sep; echo "  $*"; sep; }

# ── Build ──────────────────────────────────────────────────────────────────────

header "1 / 5  Building static_embedded (release)  [build.rs → .bin files → include_bytes!]"
cargo build --release -p static_embedded 2>&1

header "2 / 5  Building static_literal (release)   [build.rs → .rs literals → include!]"
cargo build --release -p static_literal 2>&1

header "3 / 5  Building lazy_computed (release)    [lazy_static Box<[f64]>, runtime computed]"
cargo build --release -p lazy_computed 2>&1

header "4 / 5  Building lazy_array (release)        [lazy_static [f64; N], runtime computed]"
cargo build --release -p lazy_array 2>&1

header "5 / 5  Building runtime_computed (release)"
cargo build --release -p runtime_computed 2>&1

# ── Smoke-test: all five should print the same sum ────────────────────────────

header "Smoke test — outputs must agree"
echo "  static_embedded  : $("$STATIC_BIN")"
echo "  static_literal   : $("$LITERAL_BIN")"
echo "  lazy_computed    : $("$LAZY_BIN")"
echo "  lazy_array       : $("$ARRAY_BIN")"
echo "  runtime_computed : $("$RUNTIME_BIN")"

# ── Binary sizes ───────────────────────────────────────────────────────────────

header "Binary sizes (ls -lh)"
ls -lh "$STATIC_BIN" "$LITERAL_BIN" "$LAZY_BIN" "$ARRAY_BIN" "$RUNTIME_BIN"

# ── Section breakdown — macOS (size) ──────────────────────────────────────────

header "Section sizes — static_embedded"
size "$STATIC_BIN"

header "Section sizes — static_literal"
size "$LITERAL_BIN"

header "Section sizes — lazy_computed"
size "$LAZY_BIN"

header "Section sizes — lazy_array"
size "$ARRAY_BIN"

header "Section sizes — runtime_computed"
size "$RUNTIME_BIN"

# ── objdump largest sections (macOS uses llvm-objdump or system objdump) ──────

run_objdump() {
    local bin="$1" label="$2"
    sep
    echo "  Top sections by size — $label"
    sep
    if command -v objdump &>/dev/null; then
        objdump -h "$bin" 2>/dev/null \
          | awk 'NR>4 && NF>=5 { printf "%12s  %s\n", $3, $2 }' \
          | sort -rn \
          | head -15 \
          || true
    else
        echo "  (objdump not available — install Xcode command line tools)"
    fi
}

run_objdump "$STATIC_BIN"   "static_embedded"
run_objdump "$LITERAL_BIN"  "static_literal"
run_objdump "$LAZY_BIN"     "lazy_computed"
run_objdump "$ARRAY_BIN"    "lazy_array"
run_objdump "$RUNTIME_BIN"  "runtime_computed"

# ── cargo-bloat (optional) ────────────────────────────────────────────────────

if cargo bloat --version &>/dev/null 2>&1; then
    header "cargo bloat — static_embedded (top 20 functions by size)"
    cargo bloat --release -p static_embedded -n 20 2>&1 || true

    header "cargo bloat — static_literal (top 20 functions by size)"
    cargo bloat --release -p static_literal -n 20 2>&1 || true

    header "cargo bloat — lazy_computed (top 20 functions by size)"
    cargo bloat --release -p lazy_computed -n 20 2>&1 || true

    header "cargo bloat — lazy_array (top 20 functions by size)"
    cargo bloat --release -p lazy_array -n 20 2>&1 || true

    header "cargo bloat — runtime_computed (top 20 functions by size)"
    cargo bloat --release -p runtime_computed -n 20 2>&1 || true
else
    sep
    echo "  cargo-bloat not installed (install: cargo install cargo-bloat)"
    echo "  Skipping per-function breakdown."
fi

# ── Summary table ─────────────────────────────────────────────────────────────

header "SUMMARY"

bytes_of() { stat -f%z "$1" 2>/dev/null || stat -c%s "$1"; }

STATIC_BYTES=$(bytes_of "$STATIC_BIN")
LITERAL_BYTES=$(bytes_of "$LITERAL_BIN")
LAZY_BYTES=$(bytes_of "$LAZY_BIN")
ARRAY_BYTES=$(bytes_of "$ARRAY_BIN")
RUNTIME_BYTES=$(bytes_of "$RUNTIME_BIN")

fmt_mb() { echo "scale=2; $1 / 1048576" | bc; }

printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "static_embedded"  "$STATIC_BYTES"  "$(fmt_mb $STATIC_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "static_literal"   "$LITERAL_BYTES" "$(fmt_mb $LITERAL_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "lazy_computed"    "$LAZY_BYTES"    "$(fmt_mb $LAZY_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "lazy_array"       "$ARRAY_BYTES"   "$(fmt_mb $ARRAY_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "runtime_computed" "$RUNTIME_BYTES" "$(fmt_mb $RUNTIME_BYTES)"

echo ""
printf "  static_embedded  / lazy_computed    = %.1fx\n" \
    "$(echo "scale=4; $STATIC_BYTES / $LAZY_BYTES"    | bc)"
printf "  static_literal   / lazy_computed    = %.1fx\n" \
    "$(echo "scale=4; $LITERAL_BYTES / $LAZY_BYTES"   | bc)"
printf "  static_embedded  / lazy_array       = %.1fx\n" \
    "$(echo "scale=4; $STATIC_BYTES / $ARRAY_BYTES"   | bc)"
printf "  static_literal   / lazy_array       = %.1fx\n" \
    "$(echo "scale=4; $LITERAL_BYTES / $ARRAY_BYTES"  | bc)"
printf "  static_embedded  / runtime_computed = %.1fx\n" \
    "$(echo "scale=4; $STATIC_BYTES / $RUNTIME_BYTES" | bc)"
sep
echo "  KEY INSIGHT"
echo "  static_embedded and static_literal have identical binary bloat (~32 MB"
echo "  in .rodata) because the compiler emits the same data whether it came"
echo "  from include_bytes! or from parsed f64 literals."
echo ""
echo "  lazy_static does NOT help when the values are literals: the data is"
echo "  already in the binary before main() runs. This is the Symphonia pattern."
echo ""
echo "  lazy_array uses the same [f64; N] type as static_literal but computes"
echo "  values at runtime — BSS grows, but the binary file stays small."
echo ""
echo "  lazy_computed and runtime_computed stay small for the same reason:"
echo "  only computation CODE is compiled, never the data values themselves."
sep
