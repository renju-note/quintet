# How `src/mate/` and `src/analysis/` search for mates

This document explains the mate solvers: how a position is searched for a
VCF (Victory by Continuous Fours) or a VCT (Victory by Continuous Threats),
what `limit` and `threat_limit` mean, and how the proof-number search in
`src/mate/vct/` is put together. It builds on the board vocabulary from
[02-board-implementation.en.md](02-board-implementation.en.md) — in particular
`Four`, `Sword`, `Three`, `eyes()`, forbidden moves and `zobrist_hash_n` — and
is written against the current code; identifiers in backticks can be grepped.

Module map:

| File | Role |
| --- | --- |
| `mate/solve.rs` | Public entry point `solve` and `SolveMode`; input validation; the solver regression tests. |
| `mate/game.rs` | `Game`: board + move history + side to move, with pass support; `check_event` (four detection); `End`. |
| `mate/state.rs` | `State` trait: play/undo that also tracks the remaining `limit`, and the transposition key. |
| `mate/mate.rs` | `Mate`: the result (`End` + move path). |
| `mate/vcf/` | VCF solver: `VCFState` (four-making move pairs), `DFSSolver`, `IDDFSSolver`. |
| `mate/vct/` | VCT solvers built from traits: `Generator`, `Searcher`, `Selector`, `Traverser`, `Resolver`, `ProofTree`, `VCFHelper`; three concrete solvers `DFSVCTSolver`, `PNSVCTSolver`, `DFPNSVCTSolver`. |
| `mate/vct_lazy/` | Experimental "lazy" VCT solver (`LazyVCTSolver`) with the same shape as `vct/`. |
| `analysis/field.rs` | `PotentialField`: incrementally maintained per-point potential used for move ordering in VCT. |

---

## 1. Entry point (`solve.rs`)

```rust
pub fn solve(mode: SolveMode, limit: u8, board: &Board, attacker: Player, threat_limit: u8) -> Option<Mate>
```

`attacker` is the side to move and the side we try to prove a win for. The
other side is called the *defender* throughout the code.

### `SolveMode`

| `SolveMode` | CLI name | Solver | Notes |
| --- | --- | --- | --- |
| `VCFDFS` | `vcf` | `vcf::DFSSolver` | `threat_limit` is ignored. |
| `VCFIDDFS` | `vcf_iddfs` | — | Reserved: `solve` currently returns `None`. |
| `VCTDFS` | `vct` | `DFSVCTSolver` | Depth-first traversal of the same AND/OR tree as the two below. |
| `VCTIDDFS` | `vct_iddfs` | — | Reserved: `solve` currently returns `None`. |
| `VCTPNS` | `vct_pns` | `PNSVCTSolver` | Best-first proof-number search. |
| `VCTDFPNS` | `vct_dfpns` | `DFPNSVCTSolver` | df-pn (depth-first proof-number search). The default in practice. |
| `VCTLAZY` | `vct_lazy` | `LazyVCTSolver` | Experimental (§5). `threat_limit` is ignored. |

### `limit` and `threat_limit`

- `limit` is the maximum number of **attacker moves** in the solution. Every
  attacking move counts, including fours. A solution path of 7 moves (4
  attacks and 3 defences) needs `limit >= 4`; see `test_vct_black`.
- `threat_limit` bounds the depth of the nested VCF searches that the VCT
  solvers run (§4.2). It does not affect `VCFDFS`.

### `validate`

Before searching, `validate` rejects positions the solvers do not handle:

| Position | Result |
| --- | --- |
| A `Five` of either colour is already on the board | `None` |
| Black already has an `Overlined` | `None` |
| The attacker already has a `Four` | `Some(Mate { end: Unknown, path: [] })` — treated as already won, with nothing to prove. |

### `Mate` and `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

`path` alternates attacker and defender moves starting with the attacker.
`end` says why the defender is lost after the last move of the path:

- `Fours(p1, p2)`: the attacker has fours with two distinct winning points
  `p1` and `p2`, so one block is not enough. For White this is a straight
  four or a double-four; for Black a double-four is a forbidden move
  (`is_forbidden_move` is checked before every attack), so in practice it is
  a straight four, which `Square` reports as two adjacent `Four`s with
  different eyes.
- `Forbidden(p)`: the attacker has a single four whose only block `p` is a
  forbidden move for the defender. Only possible when the defender is Black.
- `Unknown`: the search proved the win but the path could not be completed
  (see `validate` above and the `Resolver` in §4.6).

`Mate::n_moves()` is the path length and `n_times()` the number of attacker
moves in it.

## 2. Shared search state (`game.rs`, `state.rs`, `mate.rs`)

### `Game`

`Game` owns a `Board`, the list of moves played during the search
(`Vec<Option<Point>>`, `None` being a pass), the side to move `turn` and a
`passed` flag. `play` / `undo` mutate in place; `into_play(m, f)` plays `m`,
runs `f`, undoes, and returns `f`'s result — this is the pattern every
solver uses to walk the tree without cloning boards.

A pass (`play(None)`) is used to ask "what could the *other* side do if I
did nothing?": it is how threats are detected in VCT (§4.2).

### `check_event`: fours on the board

`Game::check_event()` looks at the fours of the side that just moved (the
opponent of `turn`) and classifies the situation for the side to move:

| Winning points of the opponent's fours | `Event` | Meaning for the side to move |
| --- | --- | --- |
| Two distinct points | `Defeated(Fours(p1, p2))` | Lost: cannot block both. |
| One point `p`, and `p` is forbidden for the side to move | `Defeated(Forbidden(p))` | Lost: the only block is illegal. |
| One point `p` | `Forced(p)` | Must play `p`. |
| None | `None` | Free to choose. |

Only fours through the last move are examined (`structures_on(last_move,
..., Four)`), because earlier fours would already have forced a block. When
the last move was a pass there is no such point, and all fours of the
opponent are scanned instead. The two-point test (`take_distinct_two`)
deliberately treats a straight four — two `Four`s with different eyes — the
same as a double-four.

### `State`: limit bookkeeping and transposition keys

`State` (implemented by `VCFState`, `VCTState` and `LazyVCTState`) wraps a
`Game` with `attacker` and the remaining `limit`:

- `play(m)` plays `m` on the game and, if it is now the attacker's turn
  again (i.e. the defender just moved), decrements `limit`. `undo` reverses
  both. So `limit` is "attacker moves still allowed", and at a defender node
  it still includes the attack that was just played. A hook `after_play` /
  `after_undo` lets `VCTState` refresh its potential field.
- `attacking()` is `turn == attacker`.
- `zobrist_hash()` is `Board::zobrist_hash_n(limit)`: the position combined
  with the remaining depth. Every memo table in the solvers is keyed by this
  value, so the same position reached with a different budget is a
  different entry. A pass does not change the board, so a passed position
  hashes like the position before the pass (with its own `limit`).

## 3. VCF (`vcf/`)

A VCF is a sequence in which every attacker move makes a four (so the
defender's reply is forced) and the last one makes a `Fours` or `Forbidden`
end. The solver is a plain depth-first search with a fail memo.

### Move generation: `Sword` eyes

`VCFState` generates attacks from `Sword` structures of the side to move:
three stones in a 5-window with two empty *eyes*. Playing one eye makes a
`Four` whose remaining winning point is the other eye, so each sword yields
two `(attack, defence)` pairs via `sword_eyes_pairs`:

```
H8 H9 H10 . .   (column H, H7 occupied by White)  -> Sword, eyes H11, H12
pairs: (H11, H12)  play H11: four H8-H11, White must answer H12
       (H12, H11)  play H12: four H8,H9,H10,_,H12, White must answer H11
```

Three generators, in the order the solver tries them:

- `forced_move_pair(p)`: when the attacker is `Forced(p)` (the defender's
  block made a four of its own — a counter-four), the pair whose attack is
  exactly `p`. The block must itself be a four, otherwise the VCF is over.
- `neighbor_move_pairs()`: pairs from swords through `last2_move` (the
  attacker's previous stone). Tried first because continuing on the same
  stones is the most likely way to keep making fours.
- `move_pairs()`: pairs from every sword on the board.

### `DFSSolver`

```
solve(state):
    if state.limit == 0: return None
    if deadends contains state.zobrist_hash(): return None
    result = solve_move_pairs(state)
    if result is None: deadends.insert(hash)
    return result

solve_move_pairs(state):                        # attacker to move
    match state.check_event():
        Defeated(_)  -> None                    # defender has a double four
        Forced(p)    -> forced_move_pair(p) then solve_attack
        None         -> try neighbor pairs, then all other pairs; first Some wins

solve_attack(state, attack, defence):
    if attack is forbidden: return None
    play attack; result = solve_defence(state, defence); undo
    prepend attack to the path

solve_defence(state, defence):                 # defender to move
    if check_event() is Defeated(end): return Mate { end, path: [] }
    play defence; result = solve(state); undo   # limit decrements here
    prepend defence to the path
```

Notes:

- `deadends: HashSet<u64>` remembers positions (with their `limit`) that
  have no VCF, so transpositions and repeated calls from VCT do not redo the
  work. Successes are not memoised; they are returned immediately.
- The defence point is not re-derived from the board after the attack: if
  the attack happens to make two fours, `check_event` on the defender's side
  reports `Defeated(Fours(..))` before `defence` is ever played, so the
  stored partner eye only matters for a single four.
- Counter-fours are handled by `Forced` on the attacker's side: after the
  defender's block the attacker may have to block in turn, and that block
  must be a four (`forced_move_pair`), otherwise the sequence fails.
  `test_vcf_counter` and `test_vcf_not_opponent_double_four` cover this.

### `IDDFSSolver`

`IDDFSSolver::init(limits)` runs the same `DFSSolver` with each `limit` in
`limits` that is smaller than the state's own limit, then with the full
limit, returning the first solution. Because the memo is keyed by `(board,
limit)`, the shallow passes never poison the deeper ones. The VCT solvers use
`IDDFSSolver::init(vec![1])` — "check for a one-move win first" — and the
lazy solver uses `(1..u8::MAX)`, i.e. full iterative deepening.

### Example

The board from `test_vcf_counter`, Black to move:

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . x . . . . . . .
 . . . . . . . o . o o x . . .
 . . . . . . . . o x . o . . .
 . . . . . . . . o . x . . . .
 . . . . . . . . x . . x . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`solve(VCFDFS, 3, &board, Black, 0)` returns the path `I8,G8,I10,I9,J9` with
end `Fours(H11, M6)`:

1. `I8` makes the four `H8,I8,J8,K8` (sword `H8,J8,K8` with eyes `G8`, `I8`);
   White must block `G8`.
2. `I10` makes the four `I6,I7,I8,_,I10`; White must block `I9`.
3. `J9` makes `I10,J9,K8,L7` with both `H11` and `M6` open: a straight four,
   reported as two `Four`s, so `check_event` for White is
   `Defeated(Fours(H11, M6))`.

## 4. VCT (`vct/`)

### 4.1 What counts as a threat

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

### 4.2 `VCTState`

`VCTState` = `Game` + `attacker` + `limit` + a `PotentialField` (§6) for the
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

### 4.3 Move generation (`generator.rs`)

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

### 4.4 Proof numbers (`proof.rs`)

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
the most stubborn defence (§4.6).

`ProofTree` gives access to two transposition tables (`Table`, a
`HashMap<u64, Node>`): `attacker_table` stores the values of positions
reached by an attack (defender to move) and `defender_table` those reached
by a defence (attacker to move). `Table::lookup_next(state, m)` looks up the
child after `m` using `next_zobrist_hash`.

### 4.5 Search (`searcher.rs`, `selector.rs`, `traverser.rs`)

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

### 4.6 Resolver (`resolver.rs`)

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
highest first (`I10`, `G10`, `G9`, `F10`, ...; the overlay is in §6). The
depth-first solver therefore tries `I10` before `F10`. `I10` is refuted at
once: it makes no three, so after a White pass Black has no one-move VCF,
`compute_defences` returns `zero_dn`, and the refutation is stored in
`attacker_table`.

## 5. Lazy VCT (`vct_lazy/`)

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
(`F10,G9,I10` for the board in §4, versus the full line from the other
solvers). The `VCTLAZY` expectations in `solve.rs` document this behaviour
rather than a target.

## 6. `PotentialField` (`analysis/field.rs`)

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
sum, `.` is zero). For the board in §4's example with `PotentialField::init(Black, 2, ..)`:

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

## 7. Cheat sheet

| Question | Where to look |
| --- | --- |
| Add a new solve mode | `SolveMode` + `FromStr` + the `match` in `solve` (`solve.rs`). |
| Why did the solver stop at depth N? | `limit` counts attacker moves; `search_defences` returns `zero_dn` when `limit <= 1` at a defender node. |
| A threat is not recognised | `compute_defences` step 1 (`solve_attacker_threat`) with `attacker_vcf_depth = threat_limit`; the VCF is limited to `Sword` eyes. |
| A defence is missing | `VCTState::threat_defences` (path, `end_breakers`, `counter_defences`, `four_moves`). |
| A refutation by counter-attack is missing | `solve_defender_vcf` is limited to `defender_vcf_depth = 2`; deeper counter-VCFs are found only if a counter-four appears in `threat_defences`. |
| Move ordering | `PotentialField` (`analysis/field.rs`) with `min = 2`, candidates need a sum `>= 3`. |
| Transposition tables | `ProofTree::attacker_table` / `defender_table`, `Generator::*_cache`, `DFSSolver::deadends`; all keyed by `zobrist_hash_n(limit)`. |
| Path extraction | `Resolver`; `End::Unknown` means the tables had no proven child to follow. |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode`. |
