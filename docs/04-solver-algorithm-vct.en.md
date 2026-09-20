# How `src/mate/vct/` searches for VCTs

This document explains the VCT (Victory by Continuous Threats) solvers in
`src/mate/vct/`, the experimental lazy variant in `src/mate/vct_lazy/`, and
the `PotentialField` in `src/analysis/field.rs` that orders their moves. It
assumes [03-solver-overview.en.md](03-solver-overview.en.md): the `solve`
entry point, `limit` / `threat_limit`, `Mate` / `End`, `Game::check_event`,
the `State` trait, and the VCF solver (`VCFState`, `DFSSolver`,
`IDDFSSolver`), which the VCT solvers call as a subroutine.

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
*threat*: if the defender were to pass, the attacker would have a VCF of at
most `threat_limit` fours. A four is the trivial case (the defender is
`Forced`), a three is a threat with a one-move VCF (the straight-four
point), and with `threat_limit >= 2` "hidden" threats such as a move that
prepares a four-three (Japanese *fukumi-te*; see `test_vct_fukumi_move`,
which needs `threat_limit = 3`) qualify as well. The defender may answer any
threat with any move that stops the threatened VCF, including a counter-four
or a counter-VCF.

The search is an AND/OR tree: attacker nodes (OR: one good attack suffices)
and defender nodes (AND: every defence must lose). It is solved with proof
numbers, and the three `SolveMode`s differ only in how they traverse the
tree.

## 2. `VCTState`

`VCTState` = `Game` + `attacker` + `limit` + a `PotentialField` (§8) for the
attacker, initialised with `PotentialField::init(attacker, 2, board)` and
refreshed along the four lines of each move in `after_play` / `after_undo`.

Two derived `VCFState`s drive the nested VCF searches (`VCFHelper`):

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

The `max` argument is `attacker_vcf_depth` (= `threat_limit`) for the
attacker's searches and `defender_vcf_depth` (hard-coded to `2` in `solve`)
for the defender's. The solvers behind them are `IDDFSSolver`s with
`limits = [1]`, one per side, whose `deadends` memo persists for the whole
VCT search.

`next_zobrist_hash(m)` computes the key of the child after `m` without
touching the potential field (the field update is the expensive part), which
is how table lookups for unexpanded children stay cheap.

## 3. Move generation (`generator.rs`)

Both generators return `Result<Vec<Point>, Node>`: `Ok(candidates)` for an
inner node, or `Err(node)` when the position can be decided on the spot
(`Node::zero_pn` = proven for the attacker, `Node::zero_dn` = disproven).
Results are memoised in two `LruCache`s of 1000 entries keyed by
`zobrist_hash()`.

`compute_attacks` (attacker to move):

1. If `solve_attacker_vcf` finds a VCF, return `Err(zero_pn)`. Not required
   for correctness (the main search would find the fours itself, one
   `Forced` reply at a time), but much faster.
2. If `solve_defender_threat` finds a VCF for the defender (the defender
   threatens to win if the attacker does nothing), restrict the candidates
   to `threat_defences(threat)` (below): the attack must also parry that
   threat.
3. Candidates are the points with potential `>= 3` in the attacker's field
   (`sorted_potentials(3, ..)`), highest first, minus forbidden moves.
   Empty → `Err(zero_dn)`.

Note that the candidates are not filtered for being threats here; that is
done one ply later, at the defender node, where a non-threat is refuted by
`compute_defences` step 1.

`compute_defences` (defender to move):

1. `solve_attacker_threat`: if the attacker has no VCF after a defender
   pass, the last attack was not a threat → `Err(zero_dn)`.
2. `solve_defender_vcf`: if the defender has a VCF of their own (up to
   `defender_vcf_depth` fours), they win first → `Err(zero_dn)`.
3. Candidates are `threat_defences(threat)` sorted by the attacker's
   potential (`sort_by_potential`), minus forbidden moves. Empty →
   `Err(zero_pn)`: the attack cannot be answered.

`threat_defences(threat)` is the heuristic set of moves that might stop the
threatened VCF, in this order:

- every point of the threat's path — both the attacker's fours and the
  defender's forced blocks (occupying any of them breaks the sequence);
- `end_breakers(end)`: for `Fours(p1, p2)` the two winning points; for
  `Forbidden(p)` the point itself and every empty point within 5 steps along
  its four lines (`neighbors(p, 5, true)`), since a stone nearby may change
  whether `p` is forbidden;
- `counter_defences(threat)`: replay the threat and, for each forced block
  the defender makes in it, the eyes of the defender's `Sword`s through that
  block — points where the defender would get a four during the sequence,
  which played now may turn into a counter-four;
- `four_moves()`: every four-making move (`Sword` eyes) the defender has
  right now, i.e. counter-fours that force the attacker to respond.

The list may contain a point twice; the later `dedup` only removes adjacent
duplicates after sorting by potential, so duplicates are possible but
harmless (the same child is simply looked up twice).

## 4. Proof numbers (`proof.rs`)

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

`pn` is the proof number (an estimate of how many leaves still have to be
proven for the attacker to win) and `dn` the disproof number. `pn == 0`
means proven, `dn == 0` disproven:

| Constructor | `(pn, dn)` | Meaning |
| --- | --- | --- |
| `Node::inf()` | `(INF, INF)` | No information; also the root threshold "search until decided". |
| `Node::zero_pn(limit)` | `(0, INF)` | Proven (attacker wins). |
| `Node::zero_dn(limit)` | `(INF, 0)` | Disproven. |
| `Node::unit_dn(n, limit)` | `(n, 1)` | Initial estimate of an unexpanded attacker child among `n` siblings. |
| `Node::unit_pn(n, limit)` | `(1, n)` | Initial estimate of an unexpanded defender child among `n` siblings. |

Combining children: `min_pn_sum_dn` (OR node: pn = min, dn = sum) and
`min_dn_sum_pn` (AND node: pn = sum, dn = min), both with saturating sums.
`limit` rides along as the minimum over the children and records how much
budget was left where the subtree was decided; the resolver uses it to pick
the most stubborn defence (§6).

`ProofTree` gives access to two transposition tables (`Table`, a
`HashMap<u64, Node>`): `attacker_table` stores the values of positions
reached by an attack (defender to move) and `defender_table` those reached
by a defence (attacker to move). `Table::lookup_next(state, m)` looks up the
child after `m` using `next_zobrist_hash`.

## 5. Search (`searcher.rs`, `selector.rs`, `traverser.rs`)

`Searcher::search` returns whether the root is proven:
`search_attacks(state, Node::inf()).proven()` (after a `limit == 0` check).
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

`Selector` evaluates a node from its children's table entries without
expanding anything. `select_attack` returns a `Selection`:

- `current`: the node's own `(pn, dn)` = `min_pn_sum_dn` over the children,
  where a child not yet in the table counts as `unit_dn(attacks.len())`;
- `best`: the child with the smallest `pn` (the most-proving child);
- `next1` / `next2`: the values of the best and second-best child.

If a proven child is found, `current` becomes `(0, INF)` immediately.
`select_defence` is the mirror image (smallest `dn`, `min_dn_sum_pn`,
`unit_pn(defences.len())`, `current.limit = limit - 1`). The "trick" of
initialising unexpanded children with the number of siblings makes nodes
with fewer candidate moves look easier, so the search prefers narrow,
forcing lines.

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

`traverse_defences` is identical with the defender table and
`next_threshold_defence`. A node is expanded until its numbers cross the
threshold handed down by its parent; with the root threshold `Node::inf()`
the root loops until `pn == 0` (proven: `dn` is set to `INF`) or `pn ==
INF` (disproven). The only difference between the solvers is
`next_threshold_*`:

| Trait | Child threshold | Behaviour |
| --- | --- | --- |
| `DFSTraverser` | `Node::inf()` | The chosen child is searched to completion before the parent looks at the next one: ordinary depth-first search with proof numbers used only for move ordering. |
| `PNSTraverser` | `(next1.pn + 1, next1.dn + 1)` | The child returns as soon as its numbers change, so control goes back up and the most-proving child is re-selected at every level: this emulates best-first PNS (re-selecting from the root after each expansion) inside a recursive search. |
| `DFPNSTraverser` | OR node: `pn = min(threshold.pn, next2.pn + 1)`, `dn = threshold.dn - current.dn + next1.dn`; AND node mirrored | df-pn thresholds of Nagai & Imai (2002): stay in the best child as long as it remains the best, and never exceed the parent's budget. |

`DFSVCTSolver`, `PNSVCTSolver` and `DFPNSVCTSolver` (`solver/*.rs`) are
otherwise identical structs: two `Table`s, two `IDDFSSolver`s for VCF, the
two VCF depths and the two generator caches, with all behaviour coming from
the trait default methods. `VCTSolver::solve` is

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
- If none is proven (the node was proven by the VCF shortcut in
  `compute_attacks`), return the VCF from `solve_attacker_vcf` as the tail
  of the path.

`resolve_defences` (defender to move):

- `Defeated(end)` ends the path with that `end`.
- If `Forced`, follow it.
- Otherwise recompute `threat_defences` for the attacker's threat and, among
  the proven children, pick the one with the smallest `Node::limit`. That is
  the defence that made the attacker use the most moves, so the reported
  line is against the most stubborn defence.
- If no candidate is proven (the node was proven because no legal defence
  existed), the path ends with `End::Unknown`.

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
depth-first solver therefore tries `I10` before `F10`. `I10` is refuted at
once: it makes no three, so after a White pass Black has no one-move VCF,
`compute_defences` returns `zero_dn`, and the refutation is stored in
`attacker_table`.

## 7. Lazy VCT (`vct_lazy/`)

`LazyVCTSolver` is an earlier, experimental variant that is kept for
comparison and is not maintained to the same standard (see the comment at
the top of `vct_lazy.rs`; the idea comes from Nagai's 2011 GPW paper on
solving hisshi problems). It has the same files as `vct/` and the same
`Searcher` / `Traverser` / `Resolver` structure, with these differences:

- Only df-pn thresholds (`Traverser::next_threshold_*` are the df-pn
  formulas) and candidates carry their own initial `Node`
  (`&[(Point, Node)]`).
- `generate_attacks` is just the potential filter (`>= 3`, not forbidden);
  there is no VCF shortcut and no narrowing by the defender's threat.
- `generate_defences` does not call a separate VCF solver to check the
  threat. Instead it treats "the defender passes" as a pseudo-child (move
  `None`) of the defender node and searches it with the same df-pn machinery
  restricted to four-making moves (`loop_defence_pass` →
  `search_limit_passed` → `search_attacks_passed`, which generates
  `four_moves()` only and accepts a `Forced` reply only if it is a four).
  The pass node is stored in `defender_table` like any other child, and its
  search is bounded by the parent's threshold, so the threat check is
  interleaved with the main search instead of being run to completion up
  front — hence "lazy". If the pass node is not proven, its `Node` is
  returned as the value of the defender node.
- While the pass subtree is being proven, the points that would break it
  are recorded in `defences_memory: HashMap<u64, Vec<Point>>`, keyed by
  position: `end_breakers` at the terminal, the winning attack and the
  forced block at each level (`traverse_attacks_passed`,
  `traverse_defences_passed`), and the eyes of the defender's swords through
  each block (`next_sword_eyes`, the analogue of `counter_defences`). Once
  the pass node is proven, the defender's candidates are the recorded set
  plus `four_moves()`, sorted by potential.
- `Resolver` needs `solve_attacker_vcf` / `solve_attacker_threat`, which
  `LazyVCTSolver` provides with a single `IDDFSSolver` over `1..u8::MAX`
  bounded by the state's `limit`; `threat_limit` is not used.

Because the resolver was copied from `vct/` and rebuilds the defender's
candidates with `threat_defences`, which does not always coincide with the
lazily recorded set, the extracted path can stop early with `End::Unknown`
(`F10,G9,I10` for the board in §6, versus the full line from the other
solvers). The `VCTLAZY` expectations in `solve.rs` document this behaviour
rather than a target.

## 8. `PotentialField` (`analysis/field.rs`)

The VCT generators need a cheap, always up-to-date ordering of empty points
by "how useful is a stone here for the attacker". `PotentialField` keeps,
for every point, one `u8` per direction (`Potential { v, h, a, d }`) and
reports their sum.

The per-direction value is the line potential from
`Board::potentials(player, min, exact)` (see 02, §7), computed as follows:

1. Take the 5-windows containing the point that hold no opponent stone. For
   Black, also require no own stone in the margin (`exact =
   player.is_black()`).
2. For each window, count the own stones it would hold after playing there,
   and keep the count only if it is at least `min`.
3. Report `max × (number of windows reaching that max)`.

With `min = 2` a lone stone four cells away is enough to score.

Updating and querying:

- `init(player, min, board)` fills the whole field.
- `update_along(p, board)` zeroes the four lines through `p` (`reset_along`)
  and recomputes them with `potentials_along`. `VCTState` calls it after
  every play and undo, so the cost per move is four line scans rather than a
  full board pass.
- `get(p)` is the sum over the four directions, `collect(min)` lists all
  points whose sum is at least `min`. `VCTState::sorted_potentials(3, ..)`
  and `sort_by_potential` are thin wrappers that sort descending.
- The `min` used at construction (`2`) filters windows; the `min` used at
  query time (`3`) filters sums.

`overlay(board)` renders the field for debugging (empty points show their
sum, `.` is zero). For the board in §6's example with `PotentialField::init(Black, 2, ..)`:

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

`I10` (18) and `G10` (16) sit on two of Black's lines at once, which is why
they head the attacker's candidate list. The field is the attacker's even
when ordering *defences*: a defence that lands on a high-potential point of
the attacker is tried first.

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
