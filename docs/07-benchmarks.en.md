# Benchmarking the solvers

How to tell whether a change to a solver made it faster. The benchmark is a
runner, `benches/solvers.rs`, over a set of positions, `benches/cases/*.txt`.
It solves each position, checks the answer, and reports what the search
cost.

Prerequisites: [03](03-solver-api.en.md) (`solve_with_stats`, `SolveLimits`,
`SolveResult`).

```
benches/
├── solvers.rs              the runner (`harness = false`)                (§2)
└── cases/*.txt             one position per file                         (§4)
scripts/bench-compare.sh    the runner on a git ref and on the working tree (§3)
```

## 1. What is measured

| Column | What it is | Deterministic |
| --- | --- | --- |
| `nodes` | Nodes visited: `SolveStats::nodes`, counted as `NodeBudget` counts them (03 §3), nested VCF searches included. | yes |
| `memo` | Entries left in the solver's memos: `SolveStats::memo_len`. A measure of the memory the search needed. | yes |
| `time` | Wall-clock time of `solve_with_stats`, the median of `--runs` runs. | no |

Nodes are the number to compare changes by. They do not depend on the
machine or on its load, so any difference in them is caused by the change in
code, and they can be compared across machines and in CI. They measure the
search, not the cost of one node: a change that makes each node cheaper
(bit tricks, fewer allocations, a cheaper move generator) leaves them as they
are and shows only in time. Time is comparable only between runs on the same
machine, one after the other.

The runner checks that every run of a case gives the same verdict and the
same numbers, and reports the case as failed if not.

## 2. Running it

```sh
cargo bench --bench solvers -- [OPTIONS] [NAME_FILTER...]
```

`cargo bench` builds with the release profile. The arguments after `--` go to
the runner:

| Option | Effect |
| --- | --- |
| `NAME_FILTER...` | Only the cases whose name contains one of these. |
| `--tag TAG` | Only the cases tagged `TAG`. Repeatable; a case must have all of them. |
| `--skip-tag TAG` | Leave out the cases tagged `TAG`. Repeatable. |
| `--all` | Also run the cases tagged `heavy`, which are left out unless this or `--tag heavy` is given. |
| `--mode MODE` | Run every case in `MODE` (`vcf`, `vct`, `vct_pns`, `vct_dfpns`) instead of its own. The verdict is still checked; the exact line is not, since the VCT modes may report different winning lines (03 §2). |
| `--runs N` | Time each case `N` times and report the median. Default 3. |
| `--save PATH` | Write the results as TSV. |
| `--baseline PATH` | Compare with a file written by `--save`: each row gets its change in nodes, memo entries and time, and a total at the end. |
| `--cases DIR` | Read the cases from `DIR` instead of `benches/cases`. |

A case whose answer is wrong is printed as `FAILED` with what was expected
and what was found, and the runner exits with a non-zero status. So the
benchmark is also a test, of positions too slow for `cargo test`.

Typical uses:

```sh
cargo bench --bench solvers -- --tag quick          # a few seconds
cargo bench --bench solvers                         # everything but `heavy`: a few minutes
cargo bench --bench solvers -- --tag vct --tag disproven
cargo bench --bench solvers -- --mode vct_pns --tag vct   # the same positions with PNS
```

## 3. Comparing two versions

```sh
scripts/bench-compare.sh main                   # working tree vs. main
scripts/bench-compare.sh HEAD~1 --tag vct       # runner options are passed through
```

The script checks the ref out into a temporary `git worktree`, replaces its
`benches/` with the working tree's, runs the benchmark there with `--save`,
and then runs it in the working tree with `--baseline`. Both sides run the
same runner and the same cases, so the only difference is the solver. The
ref is built in `target/bench-base`, which is kept so that the next
comparison does not rebuild it from scratch. The ref must have
`solve_with_stats`.

By hand, the same thing is:

```sh
git stash
cargo bench --bench solvers -- --save base.tsv
git stash pop
cargo bench --bench solvers -- --baseline base.tsv
```

On GitHub, the **Bench** workflow (`.github/workflows/bench.yml`) does this
for every pull request: it runs `scripts/bench-compare.sh` against the PR's
base with `--runs 1` over the default set, and puts the table in the job
summary. On a push to `main`, or when the base predates the benchmark, it
runs the benchmark without comparing. It is separate from the CI workflow
and is not a required check, so it never blocks a merge, whether it is still
running or has failed. It does fail, and show red, when a case gets a wrong
answer. Its times come from a shared runner and are rough; its node counts
are exact.

When a PR changes a solver, paste the comparison into its description. For
an algorithmic change the node column is what reviewers look at. For a
constant-factor change it is the time column: nodes should stay at `=` and
time should go down. If time changes noticeably with nodes at `=` in a PR
that did not aim for it, run the comparison again before believing it.

## 4. Cases

One file per position; the file name without `.txt` is the case name.

```
# Anything after `#` on a line of its own is a comment.
mode: vct_dfpns
attacker: o
limit: 4
threat_limit: 1
tags: vct proven quick
expect: F10,G9,I10,G10,H11,H12,G12
board:
. . . . . . . . . . . . . . .
. . . . . . . . x . . . . . .
...
```

| Key | Required | Meaning |
| --- | --- | --- |
| `mode` | yes | A `SolveMode` CLI name (03 §2). |
| `attacker` | yes | `o` (Black) or `x` (White). |
| `limit` | yes | `SolveLimits::limit`. |
| `threat_limit` | no | `SolveLimits::with_threat_limit`; default 0. |
| `defender_vcf_depth` | no | `SolveLimits::with_defender_vcf_depth`; default `DEFAULT_DEFENDER_VCF_DEPTH`. |
| `max_nodes` | no | `SolveLimits::with_max_nodes`; default unlimited. |
| `tags` | no | Space-separated tags (below). |
| `expect` | yes | The winning line, comma-separated; or `proven` (any line), `disproven` or `aborted`. |
| `board` | yes | The last key. The board follows on the next lines in the test format (`o` = Black, `x` = White, horizontal line 15 at the top), or on the same line as moves (`H8,I9,...`). |

Tags in use:

| Tag | Meaning |
| --- | --- |
| `vcf` / `vct` | Which search the case exercises. |
| `proven` / `disproven` | The expected verdict. Disproofs matter as much as proofs: most of what an engine asks has no mate, and a disproof has to search the whole tree. |
| `quick` | Well under a second. The set to run on every change. |
| `slow` | Seconds to tens of seconds. |
| `heavy` | Too slow for the default set: a minute or more, or 3 s or more among the `game` cases. Left out unless asked for. |
| `game` | Taken from a played game (§4.1). |

Adding a case:

- A position that a change is about, a position a user found slow, and a
  regression all belong here. Tag it so that it lands in the right set.
- An `expect` line pins down which winning line the solver finds. That is a
  useful check, but a change to move ordering can legitimately find another
  one. When that happens, check that the new line is a mate and update the
  file, saying so in the PR.
- Take positions whose origin is known, and write the origin in a comment:
  a puzzle's author and number, or a link.
- Keep `quick` quick, and the default set (everything but `heavy`) within a
  few minutes: the Bench workflow runs it on every pull request.

### 4.1 Positions from games (`vct_game_*`)

The puzzles above are made to have one hard mate. The `game` cases are
positions met in play instead: 100 VCT positions from about 2,400 games
between Rapfi (classical) and quintet-ai, an engine in development. A game
that ends in a five ends in a mate, so each game was walked back from its
end, two plies at a time, over the positions with the winner to move; a
position with a VCF was passed over, and for the others the shortest VCT
limit `L` was found by asking one reused df-pn solver limit 1, 2, 3, ...
(`threat_limit` 2). The walk stopped at the first position with no VCT
within limit 20.

- **Names.** `vct_game_{black|white}_lLL_N` is proven at its shortest limit
  `LL`; `..._short` is the same position at `LL - 1`, disproven. Together
  they pin the length of the mate from both sides.
- **Spread.** 50 cases per colour: 30 proven, over the limit bands 3–4, 5–6,
  7–8, 9–10, 11–12 and 13 or more (5, 6, 7, 6, 3, 3 cases), each band spread
  from its easiest to its hardest position by df-pn nodes; 20 of them also
  have their `_short` twin. The proven cases of one band come from different
  games (four games give a case to two bands each). The longest are limit 17
  (Black) and 18 (White).
- **Checked.** Every verdict was reached afresh by `vct_dfpns` and `vct_pns`
  and never contradicted by `vct` (which ran out of a 20M-node budget on 29
  of them). Proven at `LL` and disproven at `LL - 1` means `LL` is the
  shortest limit.
- **Cost.** All 100 take about 140 s in one run on an Apple M1; the 14 of
  3 s or more are tagged `heavy`, which leaves about 40 s in the default set.
  `--tag game --all` runs them all.
- Each file's comment gives the game's players and the moves up to the
  position.

## 5. Cheat sheet

| I want to… | Use |
| --- | --- |
| Know the nodes one solve took, in code | `solve_with_stats` (03 §2) |
| Run the everyday set | `cargo bench --bench solvers` |
| Check my change quickly | `cargo bench --bench solvers -- --tag quick` |
| Compare with `main` | `scripts/bench-compare.sh main` |
| Compare two VCT modes on the same positions | `--mode vct_pns --tag vct`, then `--mode vct_dfpns --tag vct` |
| Keep a result to compare with later | `--save PATH`, later `--baseline PATH` |
| Run the positions from games | `cargo bench --bench solvers -- --tag game --all` (§4.1) |
| Add a position | A new `benches/cases/NAME.txt` (§4) |
