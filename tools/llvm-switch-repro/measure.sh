#!/usr/bin/env bash
#
# Sweep the FAITHFUL reproducer (--shape interp: an #[inline(always)] N-arm switch
# inlined into N handlers) over N and build it three ways with rustc:
#
#   default : no flag                       (LLVM default pipeline)
#   on      : -switch-to-lookup=true        (forces the lookup table)
#   off     : -switch-to-lookup=false       (forces it off -> worst case)
#
# Records compile time and .s size. Also: one clang DEFAULT data point per N to show
# the slow compile reproduces at the LLVM level too. (Apple clang 17 lacks the
# -switch-to-lookup option, so the on/off toggle is rustc-only here; use an upstream
# clang/opt >= the LLVM that added the option to toggle via clang.)
#
# Finally compares against --shape table (the static-array fix), which is
# flag-independent, and dumps asm for one N.
#
# Usage:  bash measure.sh
#         NS="256 512 1024" ASM_N=1024 bash measure.sh
# Env:    NS, ASM_N, RUSTC, CLANG, RUST_EDITION
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"; GEN="$HERE/gen.py"
NS="${NS:-256 512 1024 1536}"
ASM_N="${ASM_N:-1024}"
RUSTC="${RUSTC:-rustc}"; CLANG="${CLANG:-clang}"; RUST_EDITION="${RUST_EDITION:-2021}"
OUT="${OUTDIR:-$(mktemp -d 2>/dev/null || mktemp -d -t llvmrepro)}"
have(){ command -v "$1" >/dev/null 2>&1; }

ct(){ /usr/bin/time -p "$@" >/dev/null 2>"$OUT/t"; awk '/^real/{print $2}' "$OUT/t"; }
sz(){ wc -c < "$1" | tr -d ' '; }

rs(){ # shape N mode -> "time / size"
  local shape=$1 n=$2 mode=$3 f=() s o
  python3 "$GEN" --lang rust --shape "$shape" --count "$n" --out "$OUT/r.rs" || { echo GENERR; return; }
  case $mode in on) f=(-Cllvm-args=--switch-to-lookup=true);; off) f=(-Cllvm-args=--switch-to-lookup=false);; esac
  o="$OUT/r_${shape}_${n}_${mode}.s"
  s=$(ct "$RUSTC" --edition "$RUST_EDITION" -O --crate-type=lib --emit=asm ${f[@]+"${f[@]}"} "$OUT/r.rs" -o "$o")
  echo "${s}s / $(sz "$o")B"
}
cl(){ # N -> clang default "time / size" (or n/a)
  have "$CLANG" || { echo "n/a"; return; }
  python3 "$GEN" --lang c --shape interp --count "$1" --out "$OUT/c.c" || { echo GENERR; return; }
  local o="$OUT/c_$1.s" s; s=$(ct "$CLANG" -O2 -S "$OUT/c.c" -o "$o"); echo "${s}s / $(sz "$o")B"
}

echo "# faithful (interp) reproducer measurements"
echo; echo "rustc: $($RUSTC --version)"
have "$CLANG" && echo "clang: $($CLANG --version | head -1)"
echo "outdir: $OUT"; echo
echo "Each cell: compile-time(real) / .s size."
echo
echo "| N | rustc default | rustc on (=true) | rustc off (=false) | clang default | table-fix default |"
echo "| ---: | --- | --- | --- | --- | --- |"
for n in $NS; do
  echo "| $n | $(rs interp "$n" default) | $(rs interp "$n" on) | $(rs interp "$n" off) | $(cl "$n") | $(rs table "$n" default) |"
done

echo
echo "## asm dump for N=$ASM_N (default vs on vs table) in $OUT"
python3 "$GEN" --lang rust --shape interp --count "$ASM_N" --out "$OUT/asm.rs"
"$RUSTC" --edition "$RUST_EDITION" -O --crate-type=lib --emit=asm "$OUT/asm.rs" -o "$OUT/interp_default.s"
"$RUSTC" --edition "$RUST_EDITION" -O --crate-type=lib --emit=asm -Cllvm-args=--switch-to-lookup=true "$OUT/asm.rs" -o "$OUT/interp_on.s"
echo "- $OUT/interp_default.s  (slow to produce)"
echo "- $OUT/interp_on.s       (fast; same shape)"
echo
echo "Note: rustc 'off' explodes in size at large N — keep N modest for that column."
