#!/bin/zsh
# The aliasing matrix for decisions.md R-BLEP: every candidate, both models, one process at a
# time (nice 19). `cargo build --release -p ca72-lab` first; `matrix.sh <out.tsv>`.
cd "$(dirname "$0")/../.."
L="nice -n 19 target/release/ca72-lab"
OUT=$1
: > $OUT
run() { eval $L alias "$@" >> $OUT; }
run --os 1
run --os 2
run --os 4
for b in poly2+blamp lagrange4+blamp bspline4+blamp sinc8+blamp sinc16+blamp sinc32+blamp min8+blamp min16+blamp min32+blamp; do run --os 1 --blep $b; done
for b in poly2+blamp sinc16+blamp sinc32+blamp; do run --os 4 --blep $b; done
run --os 2 --blep poly2+blamp
for b in poly2 poly2+blamp lagrange4+blamp bspline4+blamp sinc8+blamp sinc16+blamp sinc32+blamp; do run --model light --blep $b; done
run --model light
