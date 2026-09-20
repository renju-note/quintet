# How `src/mate/vct/` searches for VCTs

This document explains:

- the VCT (Victory by Continuous Threats) solvers in `src/mate/vct/`;
- the experimental lazy variant in `src/mate/vct_lazy/`;
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
| `vct/helper.rs` | `VCFHelper`: the four nested VCF questions (attacker/defender × VCF/threat). |
| `vct/generator.rs` | `Generator`: candidate attacks and defences, with terminal shortcuts. |
| `vct/proof.rs` | `Node` (proof/disproof numbers) and `Table` (transposition tables), `ProofTree`. |
| `vct/searcher.rs` | `Searcher`: the AND/OR node functions `search_attacks` / `search_defences`. |
| `vct/selector.rs` | `Selector`: evaluates a node from its children's table entries and picks the most-proving child. |
| `vct/traverser.rs` + `traverser/*.rs` | `Traverser`: the expansion loop; `DFSTraverser`, `PNSTraverser`, `DFPNSTraverser` differ only in child thresholds. |
| `vct/resolver.rs` | `Resolver`: walks the tables after a proof to extract the path. |
| `vct/solver.rs` + `solver/*.rs` | `VCTSolver` = `search` then `resolve`; the three concrete structs. |
| `vct_lazy/` | `LazyVCTSolver`, the same shape with lazy threat detection (§7). |
| `analysis/field.rs` | `PotentialField` (§8). |

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
- a `PotentialField` (§8) for the attacker, initialised with
  `PotentialField::init(attacker, 2, board)` and refreshed along the four
  lines of each move in `after_play` / `after_undo`.

### Nested VCF searches

During the search the VCT solver repeatedly asks "is there a VCF in this
position?". For that, two kinds of `VCFState` are derived from `VCTState`:

| Method | Position | VCF attacker | VCF limit | Used for |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | as is | side to move | `min(limit, max)` | Does the side to move win by fours right now? |
| `threat_state(max)` | after a pass by the side to move | the other side | `min(limit - 1, max)` if the attacker is to move, else `min(limit, max)` | Does the other side have a VCF if I do nothing — i.e. is the last move a threat? |

`VCFHelper` names the four combinations of side to move and question:

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
`Result<Vec<Point>, Node>`:

- `Ok(candidates)`: an inner node, with its list of candidate moves.
- `Err(node)`: the position is decided on the spot. `Node::zero_pn` means
  proven for the attacker, `Node::zero_dn` means disproven.

Results are memoised in an `LruCache` of 1000 entries keyed by
`zobrist_hash()` (one cache for attacks, one for defences).

### `compute_attacks` (attacker to move)

1. Call `solve_attacker_vcf`. If there is a VCF, return `Err(zero_pn)`. This
   is not required for correctness (the main search would find the fours
   itself, one `Forced` reply at a time), but it is much faster.
2. Call `solve_defender_threat`. If the defender has a VCF (the defender
   wins if the attacker does nothing), restrict the candidates to
   `threat_defences(threat)` (below): the attack must also parry that
   threat.
3. Build the candidates: the points with potential `>= 3` in the attacker's
   field (`sorted_potentials(3, ..)`), highest first, minus forbidden moves.
   Empty → `Err(zero_dn)`.

The candidates are not filtered for being threats here. That is done one ply
later, at the defender node: a non-threat is refuted by `compute_defences`
step 1.

### `compute_defences` (defender to move)

1. Call `solve_attacker_threat`. If the attacker has no VCF after a defender
   pass, the last attack was not a threat. Return `Err(zero_dn)`.
2. Call `solve_defender_vcf`. If the defender has a VCF of their own (up to
   `defender_vcf_depth` fours), they win first. Return `Err(zero_dn)`.
3. Build the candidates: `threat_defences(threat)` sorted by the attacker's
   potential (`sort_by_potential`), minus forbidden moves. Empty →
   `Err(zero_pn)`: the attack cannot be answered.

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
| `Node::inf()` | `(INF, INF)` | No information; also the root threshold "search until decided". |
| `Node::zero_pn(limit)` | `(0, INF)` | Proven (attacker wins). |
| `Node::zero_dn(limit)` | `(INF, 0)` | Disproven. |
| `Node::unit_dn(n, limit)` | `(n, 1)` | Initial estimate of an unexpanded attacker child; `n` is the number of siblings. |
| `Node::unit_pn(n, limit)` | `(1, n)` | Initial estimate of an unexpanded defender child; `n` is the number of siblings. |

There are two ways to combine children. Sums are saturating.

- `min_pn_sum_dn`: for OR nodes. pn = min, dn = sum.
- `min_dn_sum_pn`: for AND nodes. pn = sum, dn = min.

`limit` rides along as the minimum over the children. It records how much
budget was left where the subtree was decided; the resolver uses it to pick
the most stubborn defence (§6).

`ProofTree` gives access to two transposition tables (`Table`, a
`HashMap<u64, Node>`):

- `attacker_table`: values of positions reached by an attack (defender to
  move);
- `defender_table`: values of positions reached by a defence (attacker to
  move).

`Table::lookup_next(state, m)` looks up the child after `m` using
`next_zobrist_hash`.

## 5. Search (`searcher.rs`, `selector.rs`, `traverser.rs`)

### `Searcher`

`Searcher::search` returns whether the root is proven. After a `limit == 0`
check, it returns `search_attacks(state, Node::inf()).proven()`.

The two mutually recursive node functions are:

```
search_attacks(state, threshold):              # OR node, attacker to move
    Defeated(_)  -> zero_dn
    Forced(p)    -> traverse_attacks(state, [p], threshold, search_defences)
    otherwise    -> generate_attacks -> Err(node) => node
                                     | Ok(attacks) => traverse_attacks(...)

search_defences(state, threshold):             # AND node, defender to move
    Defeated(_)  -> zero_pn                     # the attacker has won
    limit <= 1   -> zero_dn                     # the attacker has no move left after this defence
    Forced(p)    -> traverse_defences(state, [p], threshold, search_attacks)
    otherwise    -> generate_defences -> Err(node) => node
                                      | Ok(defences) => traverse_defences(...)
```

### `Selector`

`Selector` expands nothing. It looks at the children's table entries and
evaluates the node. `select_attack` returns a `Selection`:

- `current`: the node's own `(pn, dn)`, computed as `min_pn_sum_dn` over
  the children. A child not yet in the table counts as
  `unit_dn(attacks.len())`.
- `best`: the child with the smallest `pn` (the most-proving child).
- `next1` / `next2`: the values of the best and second-best child.

If a proven child is found, `current` becomes `(0, INF)` immediately.

`select_defence` is the mirror image. It picks the child with the smallest
`dn`, combines with `min_dn_sum_pn`, and counts a child not in the table as
`unit_pn(defences.len())`. `current.limit` becomes `limit - 1`.

Initialising unexpanded children with the number of siblings is a "trick":
nodes with fewer candidate moves look easier, so the search prefers narrow,
forcing lines.

### `Traverser`

`Traverser` is the expansion loop shared by all three solvers:

```
traverse_attacks(state, attacks, threshold, search_defences):
    loop:
        selection = select_attack(state, attacks)
        if selection.current.pn >= threshold.pn or selection.current.dn >= threshold.dn:
            return selection                   # backoff
        next = next_threshold_attack(selection, threshold)
        play selection.best
            attacker_table.insert(child, search_defences(child, next))
        undo
```

`traverse_defences` is the same with the defender table and
`next_threshold_defence`.

A node is expanded until its numbers cross the threshold handed down by its
parent. The root threshold is `Node::inf()`, so the root loops until it is
decided: `pn == 0` (proven; `dn` is then set to `INF`) or `pn == INF`
(disproven).

The only difference between the solvers is `next_threshold_*`, i.e. how the
threshold for a child is chosen:

| Trait | Child threshold | Behaviour |
| --- | --- | --- |
| `DFSTraverser` | `Node::inf()` | The chosen child is searched to completion before the parent looks at the next one. Ordinary depth-first search; proof numbers are used only for move ordering. |
| `PNSTraverser` | `(next1.pn + 1, next1.dn + 1)` | The child returns as soon as its numbers change. Control goes back up and the most-proving child is re-selected at every level. This emulates best-first PNS (re-selecting from the root after each expansion) inside a recursive search. |
| `DFPNSTraverser` | OR node: `pn = min(threshold.pn, next2.pn + 1)`, `dn = threshold.dn - current.dn + next1.dn`; AND node mirrored | df-pn thresholds of Nagai & Imai (2002): stay in the best child as long as it remains the best, and never exceed the parent's budget. |

### The solver structs

`DFSVCTSolver`, `PNSVCTSolver` and `DFPNSVCTSolver` (`solver/*.rs`) are
identical structs apart from the thresholds. They hold only the following,
and all behaviour comes from the trait default methods:

- two `Table`s;
- two `IDDFSSolver`s for VCF and their two depths;
- the two generator caches.

`VCTSolver::solve` is

```rust
if self.search(state) { self.resolve(state) } else { None }
```

## 6. Resolver (`resolver.rs`)

The search only proves that a win exists; `Resolver` walks the tables again
to produce the path.

`resolve_attacks` (attacker to move):

- If `Forced`, follow it.
- Otherwise scan `state.empties()` and play the first move whose
  `attacker_table` entry is proven.
- If none is proven, return the VCF from `solve_attacker_vcf` as the tail
  of the path. This is a node that was proven by the VCF shortcut in
  `compute_attacks`.

`resolve_defences` (defender to move):

- `Defeated(end)` ends the path with that `end`.
- If `Forced`, follow it.
- Otherwise recompute `threat_defences` for the attacker's threat. Among
  the proven children, pick the one with the smallest `Node::limit`. That is
  the defence that made the attacker use the most moves, so the reported
  line is against the most stubborn defence.
- If no candidate is proven, the path ends with `End::Unknown`. This is a
  node that was proven because no legal defence existed.

### Example

The board from `test_vct_black` (Black to move; No. 02 of Hiroshi Okabe's
five-move problems):

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
highest first (`I10`, `G10`, `G9`, `F10`, ...; the overlay is in §8). The
depth-first solver therefore tries `I10` before `F10`.

`I10` is refuted at once. It makes no three, so after a White pass Black has
no one-move VCF. `compute_defences` returns `zero_dn`, and the refutation is
stored in `attacker_table`.

## 7. Lazy VCT (`vct_lazy/`)

`LazyVCTSolver` is an earlier, experimental variant that is kept for
comparison. It is not maintained to the same standard (see the comment at
the top of `vct_lazy.rs`). The idea comes from Nagai's 2011 GPW paper on
solving hisshi problems.

It has the same files as `vct/` and the same `Searcher` / `Traverser` /
`Resolver` structure. The differences are the following.

### Thresholds and candidates

- Only df-pn thresholds (`Traverser::next_threshold_*` are the df-pn
  formulas).
- Candidates carry their own initial `Node` (`&[(Point, Node)]`).

### Attack generation

`generate_attacks` is just the potential filter (`>= 3`, not forbidden).
There is no VCF shortcut and no narrowing by the defender's threat.

### Defence generation

`generate_defences` does not call a separate VCF solver to check the threat.
Instead:

- It treats "the defender passes" as a pseudo-child (move `None`) of the
  defender node.
- It searches that child with the same df-pn machinery, restricted to
  four-making moves (`loop_defence_pass` → `search_limit_passed` →
  `search_attacks_passed`). `search_attacks_passed` generates `four_moves()`
  only and accepts a `Forced` reply only if it is a four.
- The pass node is stored in `defender_table` like any other child, and its
  search is bounded by the parent's threshold.
- If the pass node is not proven, its `Node` is returned as the value of the
  defender node.

So the threat check is interleaved with the main search instead of being run
to completion up front — hence "lazy".

### Recording defences

While the pass subtree is being proven, the points that would break it are
recorded in `defences_memory: HashMap<u64, Vec<Point>>`, keyed by position.
The recorded points are:

- `end_breakers` at the terminal;
- the winning attack and the forced block at each level
  (`traverse_attacks_passed`, `traverse_defences_passed`);
- the eyes of the defender's swords through each block (`next_sword_eyes`,
  the analogue of `counter_defences`).

Once the pass node is proven, the defender's candidates are the recorded set
plus `four_moves()`, sorted by potential.

### Resolver

`Resolver` needs `solve_attacker_vcf` / `solve_attacker_threat`.
`LazyVCTSolver` provides them with a single `IDDFSSolver` over `1..u8::MAX`
bounded by the state's `limit`; `threat_limit` is not used.

The resolver was copied from `vct/` and rebuilds the defender's candidates
with `threat_defences`. That set does not always coincide with the lazily
recorded one. So the extracted path can stop early with `End::Unknown`: for
the board in §6 it stops at `F10,G9,I10`, while the other solvers return
the full line. The `VCTLAZY` expectations in `solve.rs` document this
behaviour rather than a target.

## 8. `PotentialField` (`analysis/field.rs`)

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
sum and `.` is zero. For the board in §6's example with
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

## 9. Cheat sheet

| Question | Where to look |
| --- | --- |
| Why did the solver stop at depth N? | `limit` counts attacker moves; `search_defences` returns `zero_dn` when `limit <= 1` at a defender node. |
| A threat is not recognised | `compute_defences` step 1 (`solve_attacker_threat`) with `attacker_vcf_depth = threat_limit`; the VCF is limited to `Sword` eyes. |
| A defence is missing | `VCTState::threat_defences` (path, `end_breakers`, `counter_defences`, `four_moves`). |
| A refutation by counter-attack is missing | `solve_defender_vcf` is limited to `defender_vcf_depth = 2`; deeper counter-VCFs are found only if a counter-four appears in `threat_defences`. |
| Move ordering | `PotentialField` (`analysis/field.rs`) with `min = 2`, candidates need a sum `>= 3`. |
| Transposition tables | `ProofTree::attacker_table` / `defender_table`, `Generator::*_cache`, `DFSSolver::deadends`; all keyed by `zobrist_hash_n(limit)`. |
| Path extraction | `Resolver`; `End::Unknown` means the tables had no proven child to follow. |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode` (see 03, §4). |
