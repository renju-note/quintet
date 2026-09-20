# How `src/mate/vct/` searches for VCTs

This document explains:

- the VCT (Victory by Continuous Threats) solvers in `src/mate/vct/`;
- the `PotentialField` in `src/analysis/field.rs` that orders their moves.

It assumes [03-solver-overview.en.md](03-solver-overview.en.md). In
particular the following are used without explanation:

- the `solve` entry point, `limit` / `threat_limit`, `Mate` / `End`;
- `Game::check_event` and the `State` trait;
- the VCF solver (`VCFState`, `DFSSolver`, `IDDFSSolver`), which the VCT
  solvers call as a subroutine.

Module map:

| File | Role |
| --- | --- |
| `vct/state.rs` | `VCTState`: `Game` + `limit` + `PotentialField`; derived `VCFState`s for the nested VCF searches; `threat_defences`. |
| `vct/solver.rs` | `VCTSolver<P>`: the one solver struct (tables, nested VCF solvers, caches); `solve` = `search` then `extract`. |
| `vct/threshold.rs` | `ThresholdPolicy` and its three implementations `DFSThreshold`, `PNSThreshold`, `DFPNSThreshold`: the only thing that differs between the solvers. |
| `vct/nested_vcf.rs` | The four nested VCF questions (attacker/defender × VCF/threat). |
| `vct/generator.rs` | Candidate attacks and defences (`Candidates`), with terminal shortcuts. |
| `vct/proof.rs` | `Node` (proof/disproof numbers) and `ProofTable` (transposition table). |
| `vct/searcher.rs` | The AND/OR node functions `search_attacks` / `search_defences` and the expansion loop `expand_attacks` / `expand_defences`. |
| `vct/selector.rs` | `select_attack` / `select_defence`: evaluate a node from its children's table entries and pick the most-proving child. |
| `vct/extractor.rs` | `extract`: walks the tables after a proof to recover the winning line. |
| `analysis/field.rs` | `PotentialField` (§9). |

The search at a glance — `solve` proves the root, then walks the proof again
to read off the path. Each node function generates candidates (asking the
nested VCF solvers a few yes/no questions on the way) and then expands the
most promising child, storing the child's result in a transposition table:

```
VCTSolver::solve
├── search ................................................. §5
│   search_attacks (OR node: attacker to move)
│   ├── generate_attacks ................................... §3
│   │     solve_attacker_vcf     -> Terminal(proven)?
│   │     solve_defender_threat  -> restrict candidates to threat_defences
│   │     candidates ordered by PotentialField ............. §9
│   └── expand_attacks: loop { select_attack; play best; search_defences; store in attacker_table }
│
│   search_defences (AND node: defender to move)
│   ├── generate_defences .................................. §3
│   │     solve_attacker_threat  -> Terminal(disproven)?  (was the attack a threat?)
│   │     solve_defender_vcf     -> Terminal(disproven)?  (does the defender win first?)
│   │     threat_defences ordered by PotentialField
│   └── expand_defences: loop { select_defence; play best; search_attacks; store in defender_table }
│
└── extract ................................................ §6
```

Proof numbers (§4) decide which child `select_*` picks, and the threshold
policy (§5) decides how long `expand_*` stays in that child before returning
to the parent. §7 walks through a complete example.

---

## 1. What counts as a threat

In this solver a VCT is a sequence in which every attacker move is a
*threat*. A threat is a move after which, if the defender were to pass, the
attacker would have a VCF of at most `threat_limit` fours.

- A four is the trivial threat: the defender is `Forced`.
- A three is a threat: after a pass, the straight-four point is a one-move
  VCF.
- With `threat_limit >= 2`, "hidden" threats such as a move that prepares a
  four-three (Japanese *fukumi-te*) qualify as well. `test_vct_fukumi_move`
  is an example and needs `threat_limit = 3`.

The defender may answer a threat with any move that stops the threatened
VCF, including a counter-four or a counter-VCF.

The search is an AND/OR tree.

- Attacker nodes are OR nodes: one good attack suffices.
- Defender nodes are AND nodes: every defence must lose.

The tree is solved with proof numbers; the three `SolveMode`s differ only in
how they traverse it.

## 2. `VCTState`

`VCTState` consists of:

- `Game`;
- `attacker`;
- `limit`;
- a `PotentialField` (§9) for the attacker, initialised with
  `PotentialField::init(attacker, 2, board)` and refreshed along the four
  lines of each move in `after_play` / `after_undo`.

### Nested VCF searches

During the search the VCT solver repeatedly asks "is there a VCF in this
position?". For that, two kinds of `VCFState` are derived from `VCTState`:

| Method | Position | VCF attacker | VCF limit | Used for |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | as is | side to move | `min(limit, max)` | Does the side to move win by fours right now? |
| `threat_state(max)` | after a pass by the side to move | the other side | `min(limit - 1, max)` if the attacker is to move, else `min(limit, max)` | Does the other side have a VCF if I do nothing — i.e. is the last move a threat? |

`nested_vcf.rs` names the four combinations of side to move and question:

| Method | Side to move | Question |
| --- | --- | --- |
| `solve_attacker_vcf` | attacker | Can the attacker win by fours right now? |
| `solve_defender_threat` | attacker | If the attacker passed, would the defender have a VCF? |
| `solve_attacker_threat` | defender | If the defender passed, would the attacker have a VCF? (Was the last attack a threat?) |
| `solve_defender_vcf` | defender | Can the defender win by fours right now? |

There are two VCF depth bounds `max`:

- for the attacker's searches, `attacker_vcf_depth`, which is `threat_limit`;
- for the defender's searches, `defender_vcf_depth`, hard-coded to `2` in
  `solve`.

The solvers behind them are `IDDFSSolver`s with `limits = [1]`, one per
side. Their `deadends` memo persists for the whole VCT search.

### `next_zobrist_hash`

`next_zobrist_hash(m)` computes the key of the child after `m`. It does not
touch the potential field, because the field update is the expensive part.
This is how table lookups for unexpanded children stay cheap.

## 3. Move generation (`generator.rs`)

There are two generators, one for attacks and one for defences. Both return
a `Candidates`:

- `Moves(candidates)`: an inner node, with its list of candidate moves.
- `Terminal(node)`: the position is decided on the spot. `Node::proven` means
  proven for the attacker, `Node::disproven` means disproven.

Results are memoised in an `LruCache` of 1000 entries keyed by
`zobrist_hash()` (one cache for attacks, one for defences).

### `compute_attacks` (attacker to move)

1. Call `solve_attacker_vcf`. If there is a VCF, return `Terminal(proven)`. This
   is not required for correctness (the main search would find the fours
   itself, one `Forced` reply at a time), but it is much faster.
2. Call `solve_defender_threat`. If the defender has a VCF (the defender
   wins if the attacker does nothing), restrict the candidates to
   `threat_defences(threat)` (below): the attack must also parry that
   threat.
3. Build the candidates: the points with potential `>= 3` in the attacker's
   field (`sorted_potentials(3, ..)`), highest first, minus forbidden moves.
   Empty → `Terminal(disproven)`.

The candidates are not filtered for being threats here. That is done one ply
later, at the defender node: a non-threat is refuted by `compute_defences`
step 1.

### `compute_defences` (defender to move)

1. Call `solve_attacker_threat`. If the attacker has no VCF after a defender
   pass, the last attack was not a threat. Return `Terminal(disproven)`.
2. Call `solve_defender_vcf`. If the defender has a VCF of their own (up to
   `defender_vcf_depth` fours), they win first. Return `Terminal(disproven)`.
3. Build the candidates: `threat_defences(threat)` sorted by the attacker's
   potential (`sort_by_potential`), minus forbidden moves. Empty →
   `Terminal(proven)`: the attack cannot be answered.

### `threat_defences`

`threat_defences(threat)` is the set of moves that might stop the threatened
VCF. It is a heuristic and lists the following four kinds, in this order:

- Every point of the threat's path, both the attacker's fours and the
  defender's forced blocks. Occupying any of them breaks the sequence.
- `end_breakers(end)`: points that break the end of the threat.
  - For `Fours(p1, p2)`, the two winning points.
  - For `Forbidden(p)`, the point itself and every empty point within 5
    steps along its four lines (`neighbors(p, 5, true)`), since a stone
    nearby may change whether `p` is forbidden.
- `counter_defences(threat)`: potential counter-fours. Replay the threat
  and, for each of the defender's blocks in it, collect the eyes of the
  defender's `Sword`s through that block. These are points where the
  defender would get a four during the sequence; played now, they may turn
  into a counter-four.
- `four_moves()`: every four-making move (`Sword` eyes) the defender has
  right now, i.e. counter-fours that force the attacker to respond.

The list may contain a point twice. The later `dedup` only removes adjacent
duplicates after sorting, so duplicates are possible but harmless (the same
child is simply looked up twice).

## 4. Proof numbers (`proof.rs`)

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

- `pn` is the proof number: an estimate of how many leaves still have to
  be proven for the attacker to win. `pn == 0` means proven.
- `dn` is the disproof number. `dn == 0` means disproven.

| Constructor | `(pn, dn)` | Meaning |
| --- | --- | --- |
| `Node::unknown()` | `(INF, INF)` | No information (a position not in the tables). |
| `Node::no_threshold()` | `(INF, INF)` | The root threshold "search until decided". Same value as `unknown`, different meaning. |
| `Node::proven(limit)` | `(0, INF)` | Proven (attacker wins). |
| `Node::disproven(limit)` | `(INF, 0)` | Disproven. |
| `Node::unexpanded_defence(n, limit)` | `(n, 1)` | Initial estimate of an unexpanded child of an OR node (defender to move); `n` is the number of siblings. |
| `Node::unexpanded_attack(n, limit)` | `(1, n)` | Initial estimate of an unexpanded child of an AND node (attacker to move); `n` is the number of siblings. |

`Node::is_proven()` is `pn == 0`.

There are two ways to combine children. Sums are saturating.

- `min_pn_sum_dn`: for OR nodes. pn = min, dn = sum.
- `min_dn_sum_pn`: for AND nodes. pn = sum, dn = min.

The intuition: at an OR node the attacker needs only one child proven, so
proving the node costs as much as proving its cheapest child (min), while
disproving it means disproving every child (sum). At an AND node the roles
swap. Walking down from the root, taking the child with the smallest `pn` at
OR nodes and the smallest `dn` at AND nodes, therefore leads to the leaf
whose result would settle the most — the *most-proving node* — and that is
what the selectors do (§5).

`limit` rides along as the minimum over the children. It records how much
budget was left where the subtree was decided; the extractor uses it to pick
the most stubborn defence (§6).

The solver holds two transposition tables (`ProofTable`, a
`HashMap<u64, Node>`):

- `attacker_table`: values of positions reached by an attack (defender to
  move);
- `defender_table`: values of positions reached by a defence (attacker to
  move).

`ProofTable::lookup_next(state, m)` looks up the child after `m` using
`next_zobrist_hash`.

## 5. Search (`searcher.rs`, `selector.rs`, `threshold.rs`)

### Node functions

`VCTSolver::search` returns whether the root is proven. After a `limit == 0`
check, it returns `search_attacks(state, Node::no_threshold()).is_proven()`.

The two mutually recursive node functions are:

```
search_attacks(state, threshold):              # OR node, attacker to move
    Defeated(_)  -> disproven
    Forced(p)    -> expand_attacks(state, [p], threshold)
    otherwise    -> generate_attacks -> Terminal(node) => node
                                     | Moves(attacks) => expand_attacks(...)

search_defences(state, threshold):             # AND node, defender to move
    Defeated(_)  -> proven                      # the attacker has won
    limit <= 1   -> disproven                   # the attacker has no move left after this defence
    Forced(p)    -> expand_defences(state, [p], threshold)
    otherwise    -> generate_defences -> Terminal(node) => node
                                      | Moves(defences) => expand_defences(...)
```

### Selection (`selector.rs`)

`select_attack` expands nothing. It looks at the children's table entries and
evaluates the node, returning a `Selection`:

- `node`: the node's own `(pn, dn)`, computed as `min_pn_sum_dn` over
  the children. A child not yet in the table counts as
  `unexpanded_defence(attacks.len())`.
- `best`: the child with the smallest `pn` (the most-proving child).
- `best_child` / `second_child`: the values of the best and second-best
  child.

If a proven child is found, `node` becomes `(0, INF)` immediately.

`select_defence` is the mirror image. It picks the child with the smallest
`dn`, combines with `min_dn_sum_pn`, and counts a child not in the table as
`unexpanded_attack(defences.len())`. `node.limit` becomes `limit - 1`.

Initialising unexpanded children with the number of siblings is a "trick":
nodes with fewer candidate moves look easier, so the search prefers narrow,
forcing lines.

### Expansion loop

`expand_attacks` is the expansion loop shared by all three solvers:

```
expand_attacks(state, attacks, threshold):
    loop:
        selection = select_attack(state, attacks)
        if selection.node.pn >= threshold.pn or selection.node.dn >= threshold.dn:
            return selection                   # exceeds_threshold
        next = P::next_threshold_attack(selection, threshold)
        play selection.best
            attacker_table.insert(child, search_defences(child, next))
        undo
```

`expand_defences` is the same with the defender table,
`P::next_threshold_defence` and `search_attacks`.

A node is expanded until its numbers cross the threshold handed down by its
parent. The root threshold is `Node::no_threshold()`, so the root loops until
it is decided: `pn == 0` (proven; `dn` is then set to `INF`) or `pn == INF`
(disproven).

### Threshold policies (`threshold.rs`)

The only difference between the solvers is `next_threshold_*`, i.e. how the
threshold for a child is chosen. That choice is the type parameter `P:
ThresholdPolicy` of `VCTSolver<P>`; the three policies are zero-sized types:

| Policy | Child threshold | Behaviour |
| --- | --- | --- |
| `DFSThreshold` | `Node::no_threshold()` | The chosen child is searched to completion before the parent looks at the next one. Ordinary depth-first search; proof numbers are used only for move ordering. |
| `PNSThreshold` | `(best_child.pn + 1, best_child.dn + 1)` | The child returns as soon as its numbers change. Control goes back up and the most-proving child is re-selected at every level. This emulates best-first PNS (re-selecting from the root after each expansion) inside a recursive search. |
| `DFPNSThreshold` | OR node: `pn = min(threshold.pn, second_child.pn + 1)`, `dn = threshold.dn - node.dn + best_child.dn`; AND node mirrored | df-pn thresholds of Nagai & Imai (2002): stay in the best child as long as it remains the best, and never exceed the parent's budget. |

### The solver struct

`VCTSolver<P>` (`solver.rs`) is one struct; `DFSVCTSolver`, `PNSVCTSolver`
and `DFPNSVCTSolver` are type aliases for the three policies. It holds only:

- two `ProofTable`s;
- two `IDDFSSolver`s for VCF and their two depths;
- the two generator caches.

Its methods are split by phase across `searcher.rs`, `selector.rs`,
`generator.rs`, `nested_vcf.rs` and `extractor.rs`. `VCTSolver::solve` is

```rust
if self.search(state) { self.extract(state) } else { None }
```

## 6. Extractor (`extractor.rs`)

The search only proves that a win exists; `extract` walks the tables again
to recover the winning line.

`extract_attacks` (attacker to move):

- If `Forced`, follow it.
- Otherwise scan `state.empties()` and play the first move whose
  `attacker_table` entry is proven.
- If none is proven, return the VCF from `solve_attacker_vcf` as the tail
  of the path. This is a node that was proven by the VCF shortcut in
  `compute_attacks`.

`extract_defences` (defender to move):

- `Defeated(end)` ends the path with that `end`.
- If `Forced`, follow it.
- Otherwise recompute `threat_defences` for the attacker's threat. Among
  the proven children, pick the one with the smallest `Node::limit`. That is
  the defence that made the attacker use the most moves, so the reported
  line is against the most stubborn defence.
- If no candidate is proven, the path ends with `End::Unknown`. This is a
  node that was proven because no legal defence existed.

## 7. Worked example

This section follows the pieces above through one regression test, from the
root position to the reported path. The board is from `test_vct_black`
(Black to move; No. 02 of Hiroshi Okabe's five-move problems):

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . x . . . . . .
 . . . . . . . o . . . . . . .
 . . . . . . . o x o . . . . .
 . . . . . . x o . x . . . . .
 . . . . . . . x o . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`solve(VCTDFS, 4, &board, Black, 1)` (also `VCTPNS`, `VCTDFPNS`) returns
`F10,G9,I10,G10,H11,H12,G12` with end `Fours(F13, K8)`; `limit = 3` fails.

- `F10` makes the three `F10,_,H8,I7` (a threat: after a pass `G9` would be
  a straight four). White's `threat_defences` include `G9`, which it plays.
- `I10` makes the three `F10,_,H10,I10`; White blocks `G10`.
- `H11` makes the four `H8..H11`; `H12` is `Forced`.
- `G12` makes `G12,H11,I10,J9` with `F13` and `K8` open: `Fours`.

At the root the attacker's candidates are the points with potential `>= 3`,
highest first (`I10`, `G10`, `G9`, `F10`, ...; the overlay is in §9). The
depth-first solver therefore tries `I10` before `F10`.

`I10` is refuted at once. It makes no three, so after a White pass Black has
no one-move VCF. `compute_defences` returns `Terminal(disproven)`, and the refutation is
stored in `attacker_table`.

## 8. Lazy VCT (removed)

An experimental "lazy" VCT solver (`vct_lazy/`, `SolveMode::VCTLAZY`) used to
live next to `vct/`. Instead of solving the attacker's threat VCF up front at
every defender node, it interleaved the threat check with the main df-pn
search and recorded the defences found along the way. The idea comes from:

> 長井歩. "難解な必至問題を解くアルゴリズムとその実装." ゲームプログラミング
> ワークショップ 2011 論文集 2011.6 (2011): 1-8.

It never reached the quality of the other solvers and was removed in
[renju-note/quintet#133](https://github.com/renju-note/quintet/pull/133); see that PR for the state it was in and the measurements
that motivated the removal.

## 9. `PotentialField` (`analysis/field.rs`)

The VCT generators need an ordering of empty points by "how useful is a
stone here for the attacker". It has to be cheap and always up to date.
`PotentialField` keeps, for every point, one `u8` per direction
(`Potential { v, h, a, d }`) and reports their sum.

### Per-direction value

The per-direction value is the line potential from
`Board::potentials(player, min, exact)` (see 02, §7), computed as follows:

1. Take the 5-windows containing the point that hold no opponent stone. For
   Black, also require no own stone in the margin (`exact =
   player.is_black()`).
2. For each window, count the own stones it would hold after playing there.
   Keep the count only if it is at least `min`.
3. Report `max × (number of windows reaching that max)`.

With `min = 2` a lone stone four cells away is enough to score.

### Updating and querying

- `init(player, min, board)` fills the whole field.
- `update_along(p, board)` zeroes the four lines through `p` (`reset_along`)
  and recomputes them with `potentials_along`. `VCTState` calls it after
  every play and undo. The cost per move is four line scans rather than a
  full board pass.
- `get(p)` returns the sum over the four directions. `collect(min)` lists
  all points whose sum is at least `min`.
- `VCTState::sorted_potentials(3, ..)` and `sort_by_potential` are thin
  wrappers that sort descending.
- There are two `min`s. The one used at construction (`2`) filters windows;
  the one used at query time (`3`) filters sums.

### Overlay

`overlay(board)` renders the field for debugging. Empty points show their
sum and `.` is zero. For the board in §7 with
`PotentialField::init(Black, 2, ..)`:

```
 . . . . . . . . . . . . . . .
 . . . 2 . . . . . . . . . . .
 . . . 2 2 2 . . . 2 . 2 . 2 .
 . . . . 4 2 4 4 . 2 2 . 4 . .
 . . . . 3 6 210 x 4 . 4 . . .
 . . . 2 41216 o18 8 8 2 . . .
 . . . 2 2 213 o x o 2 2 2 2 .
 . . . . . 2 x o12 x 8 . . . .
 . . . . 2 . 2 x o 8 2 8 2 . .
 . . . 2 . 2 . 2 4 9 4 . 4 . .
 . . . . 2 . 2 . 4 . 6 2 . 2 .
 . . . 2 . 2 . . 4 . . 3 . . .
 . . . . 2 . . . 2 . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`I10` (18) and `G10` (16) sit on two of Black's lines at once. That is why
they head the attacker's candidate list.

The field is the attacker's even when ordering *defences*: a defence that
lands on a high-potential point of the attacker is tried first.

## 10. Cheat sheet

| Question | Where to look |
| --- | --- |
| Why did the solver stop at depth N? | `limit` counts attacker moves; `search_defences` returns `disproven` when `limit <= 1` at a defender node. |
| A threat is not recognised | `compute_defences` step 1 (`solve_attacker_threat`) with `attacker_vcf_depth = threat_limit`; the VCF is limited to `Sword` eyes. |
| A defence is missing | `VCTState::threat_defences` (path, `end_breakers`, `counter_defences`, `four_moves`). |
| A refutation by counter-attack is missing | `solve_defender_vcf` is limited to `defender_vcf_depth = 2`; deeper counter-VCFs are found only if a counter-four appears in `threat_defences`. |
| Move ordering | `PotentialField` (`analysis/field.rs`) with `min = 2`, candidates need a sum `>= 3`. |
| Transposition tables | `VCTSolver::attacker_table` / `defender_table`, `VCTSolver::*_cache`, `DFSSolver::deadends`; all keyed by `zobrist_hash_n(limit)`. |
| Path extraction | `extract` (`extractor.rs`); `End::Unknown` means the tables had no proven child to follow. |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode` (see 03, §4). |
