#!/usr/bin/env bash
# Deletes the build output (target/) of CA-72 checkouts nobody has built in for a while
# (AGENTS.md: target/ is disposable). Run weekly on the owner's Mac and Linux machines.
#
#   clean-targets.sh [-n] [-d DAYS] [MAIN_CHECKOUT]
#
#   -n       list what would go, delete nothing
#   -d DAYS  idle for at least DAYS days (default 3)
#
# A checkout is every worktree of MAIN_CHECKOUT (default ~/Code/ca-72, else
# ~/Projects/Code/ca-72), wherever it lives, plus any other ca-72/ca72 folder in the code
# folder. Its target/ goes when it holds a Cargo.toml, its target/ is a real folder, nothing
# in target/ (three levels down) changed in DAYS days, and no cargo, rustc or rust-analyzer
# runs inside it. Sources, branches and worktrees are never touched.
set -u

dry=0
days=3
while getopts nd: opt; do
  case $opt in
    n) dry=1 ;;
    d) days=$OPTARG ;;
    *) echo "usage: $0 [-n] [-d DAYS] [MAIN_CHECKOUT]" >&2; exit 2 ;;
  esac
done
shift $((OPTIND - 1))

main=${1:-}
if [ -z "$main" ]; then
  for m in "$HOME/Code/ca-72" "$HOME/Projects/Code/ca-72"; do
    [ -d "$m/.git" ] && main=$m && break
  done
fi
[ -n "$main" ] && [ -d "$main/.git" ] || { echo "no CA-72 checkout found" >&2; exit 1; }
code=$(dirname "$main")

# The working folders of running Rust tools: a checkout holding one is busy.
busy=$(
  if [ -d /proc ] && [ "$(uname -s)" = Linux ]; then
    for p in $(pgrep -x cargo; pgrep -x rustc; pgrep -f rust-analyzer); do
      readlink "/proc/$p/cwd" 2>/dev/null
    done
  else
    lsof -a -c cargo -c rustc -c rust-analyzer -d cwd -Fn 2>/dev/null | sed -n 's/^n//p'
  fi | sort -u
)

{
  git -C "$main" worktree list --porcelain | sed -n 's/^worktree //p'
  for d in "$code"/ca-72* "$code"/ca72*; do [ -d "$d" ] && echo "$d"; done
} | sort -u | {
  freed=0
  while IFS= read -r w; do
    t=$w/target
    [ -f "$w/Cargo.toml" ] && [ -d "$t" ] && [ ! -L "$t" ] || continue
    if [ -n "$(find "$t" -maxdepth 3 -mtime -"$days" -print 2>/dev/null | head -n 1)" ]; then
      continue
    fi
    if printf '%s\n' "$busy" | awk -v w="$w/" 'index($0 "/", w) == 1 { f = 1 } END { exit !f }'; then
      echo "busy, kept: $t"
      continue
    fi
    kb=$(du -sk "$t" 2>/dev/null | cut -f1)
    kb=${kb:-0}
    if [ $dry = 1 ]; then
      echo "would delete $((kb / 1024)) MB: $t"
    elif rm -rf "$t" 2>/dev/null; then
      echo "deleted $((kb / 1024)) MB: $t"
    else
      # The Mac's privacy protection keeps a LaunchAgent out of ~/Documents and the like.
      echo "could not delete: $t"
      continue
    fi
    freed=$((freed + kb))
  done
  echo "$(date '+%F %T') $([ $dry = 1 ] && echo 'would free' || echo freed) $((freed / 1048576)) GB (idle >= $days days, $main)"
}
