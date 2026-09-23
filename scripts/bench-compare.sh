#!/usr/bin/env bash
# Runs the solver benchmark on a git ref and on the working tree, and prints
# the working tree's results against the ref's.
#
#   scripts/bench-compare.sh <ref> [runner options...]
#
# e.g. `scripts/bench-compare.sh main --tag vct`. Both sides run this tree's
# runner and cases (benches/), so only the solver differs. The ref must have
# `quintet::mate::solve_with_stats`. See docs/07-benchmarks.en.md.
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 <ref> [runner options...]" >&2
  exit 2
fi
ref=$1
shift

root=$(git rev-parse --show-toplevel)
work=$(mktemp -d)
trap 'git -C "$root" worktree remove --force "$work/base" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT

git -C "$root" worktree add --detach --quiet "$work/base" "$ref"
rm -rf "$work/base/benches"
cp -R "$root/benches" "$work/base/benches"
if ! grep -q '^name = "solvers"' "$work/base/Cargo.toml"; then
  printf '\n[[bench]]\nname = "solvers"\nharness = false\n' >> "$work/base/Cargo.toml"
fi

echo "== $ref ($(git -C "$root" rev-parse --short "$ref"))"
# The base's results are only a baseline: a case failing there must not stop
# the comparison.
# Its own target dir, kept between runs, so that the ref is not rebuilt from
# scratch every time nor clobbers the working tree's build.
(cd "$work/base" && CARGO_TARGET_DIR="$root/target/bench-base" \
  cargo bench --quiet --bench solvers -- --save "$work/base.tsv" "$@") || true
if [ ! -f "$work/base.tsv" ]; then
  echo "error: the benchmark did not run on $ref" >&2
  exit 1
fi
echo
echo "== working tree"
cd "$root"
cargo bench --quiet --bench solvers -- --baseline "$work/base.tsv" "$@"
