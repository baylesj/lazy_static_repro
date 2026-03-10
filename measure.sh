#!/usr/bin/env bash
# measure.sh — build all three versions and produce a detailed comparison.
# Run from the workspace root: bash measure.sh
#
# The three versions:
#   static_embedded  — pre-computed data baked into binary at compile time
#   lazy_computed    — lazy_static with runtime sin/cos computation
#   runtime_computed — plain runtime Vec with a shared helper function

set -euo pipefail
WORKSPACE="$(cd "$(dirname "$0")" && pwd)"
cd "$WORKSPACE"

STATIC_BIN="target/release/static_embedded"
LAZY_BIN="target/release/lazy_computed"
RUNTIME_BIN="target/release/runtime_computed"

sep() { printf '\n%s\n' "────────────────────────────────────────────────────────────────"; }
header() { sep; echo "  $*"; sep; }

# ── Build ──────────────────────────────────────────────────────────────────────

header "1 / 3  Building static_embedded (release)  [runs build.rs first]"
cargo build --release -p static_embedded 2>&1

header "2 / 3  Building lazy_computed (release)"
cargo build --release -p lazy_computed 2>&1

header "3 / 3  Building runtime_computed (release)"
cargo build --release -p runtime_computed 2>&1

# ── Smoke-test: all three should print the same sum ───────────────────────────

header "Smoke test — outputs must agree"
echo "  static_embedded  : $("$STATIC_BIN")"
echo "  lazy_computed    : $("$LAZY_BIN")"
echo "  runtime_computed : $("$RUNTIME_BIN")"

# ── Binary sizes ───────────────────────────────────────────────────────────────

header "Binary sizes (ls -lh)"
ls -lh "$STATIC_BIN" "$LAZY_BIN" "$RUNTIME_BIN"

# ── Section breakdown — macOS (size) ──────────────────────────────────────────

header "Section sizes — static_embedded"
size "$STATIC_BIN"

header "Section sizes — lazy_computed"
size "$LAZY_BIN"

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
run_objdump "$LAZY_BIN"     "lazy_computed"
run_objdump "$RUNTIME_BIN"  "runtime_computed"

# ── cargo-bloat (optional) ────────────────────────────────────────────────────

if cargo bloat --version &>/dev/null 2>&1; then
    header "cargo bloat — static_embedded (top 20 functions by size)"
    cargo bloat --release -p static_embedded -n 20 2>&1 || true

    header "cargo bloat — lazy_computed (top 20 functions by size)"
    cargo bloat --release -p lazy_computed -n 20 2>&1 || true

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
LAZY_BYTES=$(bytes_of "$LAZY_BIN")
RUNTIME_BYTES=$(bytes_of "$RUNTIME_BIN")

fmt_mb() { echo "scale=2; $1 / 1048576" | bc; }

printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "static_embedded"  "$STATIC_BYTES"  "$(fmt_mb $STATIC_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "lazy_computed"    "$LAZY_BYTES"    "$(fmt_mb $LAZY_BYTES)"
printf "  %-22s  %10s bytes  (%6s MB)\n" \
    "runtime_computed" "$RUNTIME_BYTES" "$(fmt_mb $RUNTIME_BYTES)"

echo ""
RATIO_SL=$(echo "scale=1; $STATIC_BYTES / $LAZY_BYTES"   | bc)
RATIO_SR=$(echo "scale=1; $STATIC_BYTES / $RUNTIME_BYTES" | bc)
RATIO_LR=$(echo "scale=1; $LAZY_BYTES   / $RUNTIME_BYTES" | bc)

printf "  static_embedded  / lazy_computed    = %sx\n" "$RATIO_SL"
printf "  static_embedded  / runtime_computed = %sx\n" "$RATIO_SR"
printf "  lazy_computed    / runtime_computed = %sx\n" "$RATIO_LR"
sep
echo "  KEY INSIGHT"
echo "  The static_embedded binary is larger because ~32 MB of pre-computed"
echo "  twiddle-factor DATA is baked into .rodata at compile time."
echo "  lazy_computed and runtime_computed are nearly the same size because"
echo "  both only compile the COMPUTATION CODE, not the data values."
echo "  The data lives on the heap and is not present in the binary."
sep
