# Performance history

How fast the solvers got, what made them so, and what is left.

- A record of the work from #149 (2026-09-23) to #170 (2026-09-28), measured
  against the 2024 version.
- For the next person tuning the solvers: what has been tried, and where the
  time goes now.

Prerequisites: [07](07-benchmarks.en.md) (the benchmark, nodes against
time), [06](06-solver-vct.en.md) (the VCT search).

## 1. Summary

All 121 cases of `benches/cases/` (`--all`), at four points:

| Point | Commit | Time, all | VCT default (99) | VCT `heavy` (16) | Nodes, all |
| --- | --- | --- | --- | --- | --- |
| 2024-03, #120 | `d8bdf3c` | 498.3 s | 106.4 s | 391.9 s | — |
| 2026-09-23, #149 | `bef47e5` | 433.5 s | 90.4 s | 343.1 s | 309.4M |
| 2026-09-27, #165 | `a34e0c8` | 82.6 s | 24.1 s | 58.4 s | 137.3M |
| 2026-09-28, #169 | `f73f4c5` | 49.6 s | 16.4 s | 33.2 s | 92.3M |

- Time: 10.0 times faster than 2024 in total; 6.5 times on the default set,
  11.8 times on `heavy`.
- Per case: 6.5 times faster as a geometric mean, 8.1 times as a median
  (over the 95 cases that took 1 ms or more in 2024).
- The 6 VCF cases: 1.25 ms → 0.36 ms in total.
- Every case gives the same verdict at all four points, and none timed out.
- #120 has no `solve_with_stats`, so its nodes cannot be counted.

How it was measured:

- On one Apple Silicon Mac, one case per process, one point after another.
  The load average was about 2.
- The default set is the median of 3 runs; `heavy` is 1 run.
- The times are only comparable with each other. The node counts hold on
  any machine.

Cases that changed the most:

| Case | 2024 | Now | Faster |
| --- | --- | --- | --- |
| `vct_unstable` | 164.3 s | 11.0 s | 15× |
| `vct_black_long_short` | 31.8 s | 2.8 s | 11× |
| `vct_game_black_l09_3` | 16.0 s | 0.8 s | 20× |
| `vct_small_but_long` | 23.4 s | 6.4 s | 3.6× (the least) |

## 2. What made the difference

Each merged commit from #149 to #170, run with the current `benches/`
(§7). #159 only adds cases.

Changes to the search (nodes, all 121 cases):

| PR | What | Nodes, all | Default set | `heavy` | Details |
| --- | --- | --- | --- | --- | --- |
| #160 | Order VCT candidates by the shapes a move makes | 309.4M → 285.6M (−8%) | −16% | −6% | 06 §8 |
| #161 | Rule out attacks that cannot be threats (the VCF zone) | → 221.2M (−23%) | −32% | −20% | 06 §3 |
| #162 | Lower the proof-number estimates of fours and threes | → 137.4M (−38%) | +6% | −47% | 06 §5 |
| #163 | `ShapeMap` replaces `PotentialField` | → 137.0M (−0.3%) | −1% | 0% | 06 §8 |
| #165 | Fewer forbidden move checks | → 137.3M (+0.2%) | 0% | +0.3% | 06 §5 |
| #167 | 1 + ε in the df-pn threshold | → 99.4M (−28%) | −19% | −31% | 06 §5 |
| #168 | Candidates caches to 65536 entries | → 92.3M (−7%) | −2% | −9% | 06 §3 |

- The other PRs leave every case's nodes as they were.
- #162's +6% on the default set is `vct_small_but_long` (5.8M → 16.7M);
  the rest of the set went down 26%. `vct_unstable` went 109.5M → 41.6M.
- #165 changes nodes a little: a forbidden attack is now found only when it
  is first searched, so it counts in its parent's numbers until then.

Time, every PR (the default set, 105 cases, one run each, one commit after
another on one cloud machine):

| PR | What | Time | Change |
| --- | --- | --- | --- |
| #149 | (start) | 188.7 s | |
| #151 | Docs and case files | 184.4 s | −2% |
| #152 | Child keys by XOR, identity hashing, lazy potentials | 124.7 s | −32% |
| #153 | Cache swords for VCF move generation | 118.3 s | −5% |
| #154 | Rename | 115.0 s | −3% |
| #155 | Move `SwordMap` into the solver states | 112.2 s | −2% |
| #156 | `Segment` and `Line` built on it | 107.6 s | −4% |
| #157 | Rename | 109.1 s | +1% |
| #158 | `Bits` iterator, bit operations for forbidden moves | 100.3 s | −8% |
| #160 | Order by shapes | 94.0 s | −6% |
| #161 | Rule out non-threats | 68.8 s | −27% |
| #162 | Proof-number estimates | 69.5 s | +1% |
| #163 | `ShapeMap` replaces `PotentialField` | 54.5 s | −22% |
| #164 | Refactor | 53.5 s | −2% |
| #165 | Fewer forbidden move checks | 45.8 s | −14% |
| #166 | Read children's numbers once | 43.7 s | −5% |
| #167 | 1 + ε | 36.2 s | −17% |
| #168 | Larger candidates caches | 35.8 s | −1% |
| #169 | Four constant factors | 33.4 s | −7% |
| #170 | Rename | 33.4 s | 0% |

- The renames (#154, #157, #170) show the noise: about ±3%.
- #163 cut time far more than nodes: keeping `PotentialField` in step with
  the board took about a fifth of the search's time (06 §8).
- The default set is mostly small cases. Changes aimed at the large ones
  show little here: #168 is −1% (−2% nodes) against −9% nodes on `heavy`.
- On this machine the 2024 version takes 190.4 s on the same set: 5.7
  times the current one (6.5 times on the Mac of §1).

In short:

- Search (nodes): ruling out attacks that cannot be threats (#161), move
  ordering and estimates by shapes (#160, #162), the 1 + ε threshold (#167)
  and larger candidates caches (#168).
- Cost per node (time at the same nodes): child keys by XOR (#152), bit
  operations (#156, #158), dropping `PotentialField` (#163), fewer
  forbidden checks (#165), and the smaller ones in #153, #166 and #169.

## 3. Cases that got slower

8 cases are slower than in 2024. All are small, under 70 ms (the default
set on the cloud machine of §2, median of 3 runs):

| Case | 2024 | Now | Nodes at #149 → now |
| --- | --- | --- | --- |
| `vct_game_black_l04_1` | 0.7 ms | 1.3 ms | 189 → 1,056 |
| `vct_game_black_l05_2` | 1.5 ms | 7.7 ms | 633 → 6,932 |
| `vct_game_black_l10_1` | 23 ms | 63 ms | 8,711 → 54,123 |
| `vct_game_white_l04_2` | 0.9 ms | 1.1 ms | 230 → 851 |
| `vct_game_white_l05_1` | 1.5 ms | 1.9 ms | 266 → 1,703 |
| `vct_game_white_l06_1` | 1.3 ms | 2.1 ms | 548 → 1,990 |
| `vct_game_white_l06_2` | 14 ms | 28 ms | 6,105 → 25,089 |
| `vct_game_white_l10_1` | 7.4 ms | 18 ms | 4,012 → 19,643 |

- The cost per node fell; the nodes grew 4 to 11 times.
- All eight grew at #162, by 1.7 to 9 times. Its estimates send the search
  after fours and threes first. On these positions that is the longer way.
- The other PRs moved them less, but for #160 on `vct_game_white_l10_1`
  (2.3 times).
- Together they cost under 0.1 s more than in 2024, against over 150 s saved
  on the same set.
- On the Mac of §1 there were 9. Cases of a millisecond or two cross over
  with the noise.

## 4. Where the time goes now

A profile of the whole benchmark on `main` just before #169:

| Part | Share |
| --- | --- |
| Nested VCF searches (`DFSSolver`, via `NestedVCF`) | about 55% |
| `check_event` | about 16% |
| `SwordMap::sync` | about 11% |
| `ShapeMap::sync` | about 10% |
| Allocation and freeing | about 5–10% |

(The parts overlap: the nested VCF searches call `check_event` and
`SwordMap::sync` too.)

- #169 then took about 5% of the whole off `check_event`, about 4% off
  `ShapeMap::sync`, and 5 to 6% off allocation (each expansion's `Vec` of
  children, and the move history cloned for every nested search).
- The nested VCF searches are still over half. Most of them answer "is this
  a threat?" (`attacker_vcf.threat`) or "does the attacker have a VCF?".

## 5. Tried and left out

| Idea | Result | Why it stays out |
| --- | --- | --- |
| ε = 1/4 to 1/2 in the df-pn threshold | `vct_unstable` +47% (1/4), +41% (1/2) nodes | Better on the default set, but unstable on the long cases. 1/8 does well on both (06 §5). |
| Sync the `SwordMap` only for the attacker | −0.6% time | Within the noise. |
| Always expand the best attack once (#108) | `vct_unstable` 345× cheaper, default set 39% dearer | Depends too much on the position (06 §5, issue #148). |
| A candidates cache over 65536 entries | 262144: −8% nodes, 100 MB | Little more for 4 times the memory (06 §3). |

## 6. What is left to try

- Fewer or cheaper nested VCF searches. They are over half the time, so
  anything that avoids asking them (as #161 did for non-threats) pays
  most.
- `check_event`, still about a tenth of the time after #169.
- The small cases in §3: estimates that keep #162's gains on the large
  cases without its losses on those.
- `vct_small_but_long` gained the least (3.6×). Its search is a long line of
  fours, where the order of expansion matters most.

## 7. Reproducing it

Nodes of a commit (any machine):

1. Check the commit out into a `git worktree`, and replace its `benches/`
   with the current one (as `scripts/bench-compare.sh` does, 07 §3).
2. In the copied cases, change every `expect:` line that is a move list to
   `expect: proven`. An older solver may find another winning line; only the
   verdict is checked.
3. Run `cargo bench --bench solvers -- --all --runs 1 --save <commit>.tsv`.

Time: run the commits one after another on the same machine, with nothing
else running. The table in §2 is `--runs 1` over the default set.

The 2024 version (#120) needs more:

- Its API is `solve(mode, limit, board, attacker, threat_limit) ->
  Option<Mate>`, with no `solve_with_stats` and no node counts.
- Time it with a small temporary example that reads a case file and calls
  that function, one case per process.
