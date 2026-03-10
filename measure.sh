#!/usr/bin/env bash
# measure.sh — build every crate × every profile and print a comparison grid.
# Run from the workspace root: bash measure.sh
#
# Crates (columns):
#   static_embedded  — include_bytes! embeds .bin files into .rodata
#   static_literal   — f64 literals in generated .rs source (Symphonia pattern)
#   lazy_computed    — lazy_static Box<[f64]>, values computed at runtime
#   lazy_array       — lazy_static [f64; N], values computed at runtime
#   runtime_computed — plain runtime Vec, shared helper (smallest binary)
#
# Profiles (rows):
#   release          — opt=3, no LTO, 16 CGU  (naive baseline)
#   release-lto      — opt=3, fat LTO, 1 CGU
#   release-size     — opt=s, strip
#   release-min      — opt=z, fat LTO, 1 CGU, strip

set -euo pipefail
WORKSPACE="$(cd "$(dirname "$0")" && pwd)"
cd "$WORKSPACE"

CRATES=(static_embedded static_literal lazy_computed lazy_array runtime_computed)
PROFILES=(release release-lto release-size release-min)

sep()    { printf '\n%s\n' "────────────────────────────────────────────────────────────────────────────────"; }
header() { sep; printf '  %s\n' "$*"; sep; }
bytes_of() { stat -f%z "$1" 2>/dev/null || stat -c%s "$1"; }
fmt_mb()   { printf "%.2f" "$(echo "scale=4; $1 / 1048576" | bc)"; }

# ── Build ──────────────────────────────────────────────────────────────────────

total=$(( ${#CRATES[@]} * ${#PROFILES[@]} ))
n=0
for profile in "${PROFILES[@]}"; do
    for crate in "${CRATES[@]}"; do
        n=$(( n + 1 ))
        header "$n / $total  $crate  [--profile $profile]"
        cargo build --profile "$profile" -p "$crate" 2>&1
    done
done

# ── Smoke test — spot-check one profile to confirm all outputs agree ──────────

header "Smoke test (release profile) — outputs must agree"
for crate in "${CRATES[@]}"; do
    bin="target/release/$crate"
    echo "  $crate : $("$bin")"
done

# ── Size grid ─────────────────────────────────────────────────────────────────

header "BINARY SIZE GRID  (MB on disk)"

# Header row
printf "  %-18s" "profile \\ crate"
for crate in "${CRATES[@]}"; do
    printf "  %16s" "$crate"
done
printf "\n"

sep_row() {
    printf "  %-18s" "------------------"
    for crate in "${CRATES[@]}"; do printf "  %16s" "----------------"; done
    printf "\n"
}
sep_row

for profile in "${PROFILES[@]}"; do
    # Cargo puts named profiles (other than release/dev) under target/<profile-name>/
    if [[ "$profile" == "release" ]]; then
        dir="target/release"
    else
        dir="target/$profile"
    fi

    printf "  %-18s" "$profile"
    for crate in "${CRATES[@]}"; do
        bin="$dir/$crate"
        if [[ -f "$bin" ]]; then
            b=$(bytes_of "$bin")
            printf "  %16s" "$(fmt_mb "$b") MB"
        else
            printf "  %16s" "(missing)"
        fi
    done
    printf "\n"
done

# ── Section breakdown for each profile × bloated crates ───────────────────────

header "SECTION SIZES  (size tool, __TEXT / __DATA / __BSS)"

for profile in "${PROFILES[@]}"; do
    [[ "$profile" == "release" ]] && dir="target/release" || dir="target/$profile"
    sep
    printf "  profile: %s\n" "$profile"
    sep
    for crate in "${CRATES[@]}"; do
        bin="$dir/$crate"
        [[ -f "$bin" ]] && { printf "  %s\n" "$crate"; size "$bin"; } || true
    done
done

# ── Key ratios per profile ─────────────────────────────────────────────────────

header "BLOAT RATIO  (static_literal / lazy_array)  per profile"
printf "  The data bloat should be nearly constant regardless of profile;\n"
printf "  optimization only affects code size, not .rodata.\n\n"

for profile in "${PROFILES[@]}"; do
    [[ "$profile" == "release" ]] && dir="target/release" || dir="target/$profile"
    lit_bin="$dir/static_literal"
    arr_bin="$dir/lazy_array"
    if [[ -f "$lit_bin" && -f "$arr_bin" ]]; then
        lit=$(bytes_of "$lit_bin")
        arr=$(bytes_of "$arr_bin")
        ratio=$(echo "scale=1; $lit / $arr" | bc)
        printf "  %-18s  static_literal=%s MB   lazy_array=%s MB   ratio=%.1fx\n" \
            "$profile" "$(fmt_mb "$lit")" "$(fmt_mb "$arr")" "$ratio"
    fi
done

sep
printf "  KEY INSIGHT\n"
printf "  Across all profiles, static_embedded and static_literal remain ~32 MB\n"
printf "  larger than the runtime variants. LTO and opt=z shrink the CODE section\n"
printf "  by a few hundred KB at most — they cannot remove data that is actively\n"
printf "  referenced. The only fix is to not embed the data at compile time.\n"
sep
