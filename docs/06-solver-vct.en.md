# VCT: the threat search (`src/mate/vct/`)

A **VCT** (victory by continuous threats) is a line in which every attacking
move is a *threat*: a four, a three, or any move after which the attacker
would have a VCF if the defender did nothing. Unlike VCF the defender has a
choice of replies, so the tree is a real AND/OR tree and the solver searches
it with **proof numbers**. This document explains that search and the
`PotentialField` that orders its moves.

Assumes [04](04-solver-framework.en.md) and [05](05-solver-vcf.en.md): in
particular `State`, `Key`, `Memo`, the `Solver` trait and `DFSSolver`.

```
src/mate/vct.rs         module doc: the algorithm in one page, re-exports, the three aliases
src/mate/vct/
├── state.rs            VCTState: Game + attacker + limit + PotentialField + SwordFields;    (§2, §3)
│                       threat_defences
├── nested_vcf.rs       NestedVCF: one side's VCF sub-search                                  (§2)
├── generator.rs        generate_attacks / generate_defences → Candidates                     (§3)
├── proof.rs            Node (proof numbers), ProofTable (transposition table)                (§4)
├── searcher.rs         search_attacks / search_defences, expand_attacks / expand_defences,   (§5)
│                       select_attack / select_defence → Selection
├── threshold.rs        ThresholdPolicy: DFSThreshold, PNSThreshold, DFPNSThreshold           (§5)
├── solver.rs           VCTSolver<P>: the struct, Solver impl                                 (§5)
└── extractor.rs        extract: the winning line from the tables                             (§6)
src/analysis/potential.rs  PotentialField                                                     (§8)
src/analysis/sword.rs      SwordField (05, §1)                                                (§2)
```

The whole search on one screen — `solve` proves the root, then walks the
proof again to read off the line:

```
VCTSolver::solve = advance_generation; search; extract
│
├── search ──► search_attacks(root, no_threshold).is_proven()
│
│   search_attacks (OR node, attacker to move)                         §5
│   ├── check_event: Defeated → disproven; Forced(p) → attacks = [p]
│   ├── generate_attacks                                               §3
│   │     attacker_vcf.vcf      has a VCF?          → Terminal(proven)
│   │     defender_vcf.threat   must parry a threat? → only threat_defences
│   │     points with potential ≥ 3, best first                        §8
│   └── expand_attacks: loop { select_attack; play best; search_defences; store in attacker_table }
│
│   search_defences (AND node, defender to move)
│   ├── check_event: Defeated → proven; limit ≤ 1 → disproven; Forced(p) → defences = [p]
│   ├── generate_defences                                              §3
│   │     attacker_vcf.threat   was that a threat?  → else Terminal(disproven)
│   │     defender_vcf.vcf      defender wins first? → Terminal(disproven)
│   │     threat_defences, best first
│   └── expand_defences: loop { select_defence; play best; search_attacks; store in defender_table }
│
└── extract ──► follow proven children through the tables               §6
```

Proof numbers (§4) decide which child `select_*` picks; the threshold
policy (§5) decides how long `expand_*` stays in it before returning to the
parent. That policy is the only difference between the DFS, PNS and df-pn
modes.

## 1. Threats and the AND/OR tree

A move is a **threat** if, were the defender to pass, the attacker would
have a VCF of at most `threat_limit` fours.

| `threat_limit` | Recognised as threats |
| --- | --- |
| 0 | fours only — the nested VCF has no depth, so only `Forced` replies get through |
| 1 | also threes: after a pass, the straight-four point is a one-move VCF |
| 2 | also moves that prepare a two-four VCF, such as one setting up a four-three |
| 3+ | deeper preparation — `test_vct_fukumi_move` needs 3 |

The defender may answer with any move that stops the threatened VCF,
including a counter-four or a counter-VCF of their own.

The tree alternates two kinds of node:

- **OR node**, attacker to move: *one* attack that wins suffices.
- **AND node**, defender to move: *every* defence must lose.

## 2. `VCTState` and the nested VCF searches

```rust
pub struct VCTState { game: Game, pub attacker: Player, pub limit: u8, field: PotentialField, swords: [SwordField; 2] }
```

The field is the attacker's `PotentialField` (§8), built with
`PotentialField::init(attacker, 2, board)`. `after_play` / `after_undo` only
mark the move's point stale; the field is refreshed along the four lines of
each stale point when it is next read (`sorted_potentials` /
`sort_by_potential`), which happens only on a candidate-cache miss.

`swords` are Black's and White's `SwordField`s (05, §1), updated the same
lazy way. `vcf_state` / `threat_state` sync the one of the side the nested
VCF is for and hand it a copy (`VCFState::with_swords`), which the nested
search then keeps up to date as it plays. `next_key(m)`
is `key()` of the child after `m`, computed from the board's Zobrist hash by
XOR without playing `m`, which is what keeps table lookups for unexpanded
children cheap.

### Two derived `VCFState`s

The VCT search constantly asks "is there a VCF here?". `VCTState` derives
the VCF state for two questions:

| Method | Position | VCF attacker | VCF limit | Question |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | as is | side to move | `min(limit, max)` | does the side to move win by fours right now? |
| `threat_state(max)` | after a pass by the side to move | the other side | `min(limit − 1, max)` if the attacker is to move, else `min(limit, max)` | if I did nothing, would the other side have a VCF? |

### `NestedVCF`: one per side

`nested_vcf.rs` packages a VCF solver with its depth bound and the side it
belongs to:

```rust
pub struct NestedVCF { solver: IDDFSSolver, depth: u8, for_attacker: bool }
impl NestedVCF {
    pub fn vcf(&mut self, state: &mut VCTState, budget) -> Option<Mate>;     // this side, to move, has a VCF?
    pub fn threat(&mut self, state: &mut VCTState, budget) -> Option<Mate>;  // this side would have one if the other passed?
}
```

`VCTSolver` holds `attacker_vcf` (depth `threat_limit`) and `defender_vcf`
(depth `defender_vcf_depth`, default 2). Two objects × two questions gives
the four calls the generators make:

| Call | Side to move | Asks |
| --- | --- | --- |
| `attacker_vcf.vcf` | attacker | can the attacker win by fours right now? |
| `defender_vcf.threat` | attacker | if the attacker passed, would the defender have a VCF — what must this attack also parry? |
| `attacker_vcf.threat` | defender | if the defender passed, would the attacker have a VCF — was the last attack a threat? |
| `defender_vcf.vcf` | defender | can the defender win by fours right now — does the defender win first? |

`vcf` asserts that its own side is to move and `threat` that the other side
is; a call the other way round is a bug in the search, not a question. The
solver inside is an `IDDFSSolver` with `limits = [1]`, called through
`search` so that its deadend memo (04, §5) lives for the whole VCT search
and across searches, and is charged to the same `NodeBudget`.

## 3. Move generation (`generator.rs`)

`generate_attacks` and `generate_defences` return a `Candidates`:

```rust
pub enum Candidates {
    Moves(Vec<Point>),   // an inner node: the moves to try, best first
    Terminal(Node),      // decided here without expanding: Node::proven or Node::disproven
}
```

Results are cached in an `LruCache` of 1000 entries per generator, keyed by
`zobrist_hash()` (position *and* limit, because the nested VCF depth is
`min(limit, depth)`), and never cached when the budget ran out.

**`compute_attacks`** (attacker to move):

1. `attacker_vcf.vcf` — a VCF now? → `Terminal(proven)`. Not needed for
   correctness (the main search would find the fours one `Forced` reply at a
   time) but much faster.
2. `defender_vcf.threat` — would the defender have a VCF if the attacker
   passed? If so the attack must also parry it: restrict candidates to
   `threat_defences(threat)`.
3. Candidates are the points with potential `≥ 3` in the attacker's field
   (`sorted_potentials(3, only)`), highest first, minus forbidden moves.
   None left → `Terminal(disproven)`.

Candidates are *not* checked for being threats here. That happens one ply
down, at the defender node, where a non-threat is refuted at once.

**`compute_defences`** (defender to move):

1. `attacker_vcf.threat` — would the attacker have a VCF if the defender
   passed? If not, the last attack was no threat → `Terminal(disproven)`.
2. `defender_vcf.vcf` — does the defender have a VCF of their own (within
   `defender_vcf_depth`)? Then the defender wins first → `Terminal(disproven)`.
3. Candidates are `threat_defences(threat)` sorted by the *attacker's*
   potential (`sort_by_potential`), minus forbidden moves. None left →
   `Terminal(proven)`: the threat cannot be answered.

### `threat_defences`

`VCTState::threat_defences(&threat)` is the heuristic list of moves that
might stop a threatened VCF, in this order:

| Part | Points |
| --- | --- |
| the path | every stone of the threat — the attacker's fours and the defender's blocks; occupying any of them breaks the sequence |
| `end_breakers(end)` | for `Fours(p1, p2)` the two winning points; for `Forbidden(p)` the point itself plus every empty point within 5 along its four lines (`neighbors(p, 5, true)`), since a stone nearby can change whether `p` is forbidden |
| `counter_defences(threat)` | replay the threat after a pass and, at each of the defender's blocks in it, collect the eyes of the defender's `Sword`s through that block — points where the defender would get a four during the sequence, which played now may become a counter-four |
| `four_moves()` | every four-making move (`Sword` eye) the defender has right now — counter-fours that force the attacker to answer |

The list may repeat a point; `dedup` after sorting only removes adjacent
duplicates, so a duplicate can survive, harmlessly (the same child is looked
up twice).

## 4. Proof numbers (`proof.rs`)

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

`pn` is the **proof number**: how many leaves, at least, still have to be
won for the attacker to prove this node. `dn` is the **disproof number**,
the same for refuting it. `pn == 0` is proven, `dn == 0` disproven.

| Constructor | `(pn, dn)` | Meaning |
| --- | --- | --- |
| `Node::unknown()` | `(INF, INF)` | a position the tables know nothing about |
| `Node::no_threshold()` | `(INF, INF)` | a threshold never exceeded: "search until decided" (same value as `unknown`, different role) |
| `Node::proven(limit)` | `(0, INF)` | the attacker wins |
| `Node::disproven(limit)` | `(INF, 0)` | the attacker cannot win within `limit` |
| `Node::unexpanded_defence(n, limit)` | `(n, 1)` | first guess for an unexpanded child of an OR node (defender to move); `n` = number of siblings |
| `Node::unexpanded_attack(n, limit)` | `(1, n)` | first guess for an unexpanded child of an AND node (attacker to move) |

Children combine in two ways, with saturating sums:

- `min_pn_sum_dn` at an **OR** node: proving it costs as much as its
  cheapest child (min pn); disproving it means disproving every child
  (sum dn).
- `min_dn_sum_pn` at an **AND** node: the mirror image.

Walking from the root, taking the smallest `pn` at OR nodes and the smallest
`dn` at AND nodes, leads to the leaf whose result would change the root the
most — the *most-proving node*. That is what the selectors follow (§5).
`limit` rides along as the minimum over the children: the least budget that
was left where the subtree was decided. The extractor uses it (§6).

### `ProofTable`

The solver keeps two tables:

- `attacker_table`: nodes reached by an attack (defender to move);
- `defender_table`: nodes reached by a defence (attacker to move).

Each is a `ProofTable`, two `Memo`s under one roof:

| Half | Keyed by | Holds |
| --- | --- | --- |
| `estimates: Memo<Node>` | `Key::hash()` — position and limit | every node inserted, as is: a proof number short of a decision belongs to the limit it was computed at |
| `decided: Memo<Decided>` | `Key::position` alone | the smallest proven limit and the largest disproven limit seen for the position |

`insert(state, node)` writes `estimates` always and `decided` when the node
is proven or disproven. `lookup_next(state, m)` reads the child after `m` by
`next_key`: first `estimates`, then `decided`, where a proof at a smaller
limit or a disproof at a larger one still answers (04, §3). Through
`decided`, a search at limit 5 reuses what a search at limit 4 settled, and
a proof found from one root answers under another root at any deeper limit.

`transfer_from` is where `decided` starts to apply. The generators ask the
nested VCF solvers with `min(limit, depth)`, so below that depth the
candidate lists still change with the limit and two limits describe
different trees. `VCTSolver::with_carry_capacity` sets it to
`max(attacker_vcf_depth, defender_vcf_depth + 1, 2)`; nothing is recorded
in or read from `decided` below it.

## 5. Search (`searcher.rs`, `threshold.rs`, `solver.rs`)

### Node functions

`VCTSolver::search(state, budget)` is `search_attacks(state,
Node::no_threshold(), budget).is_proven()` after a `limit == 0` check. The
two node functions call each other:

```
search_attacks(state, threshold):              # OR node, attacker to move
    budget.consume() or → unknown
    Defeated(_)  → disproven(limit)
    Forced(p)    → expand_attacks([p])
    otherwise    → generate_attacks: Terminal(n) → n | Moves(a) → expand_attacks(a)

search_defences(state, threshold):             # AND node, defender to move
    budget.consume() or → unknown
    Defeated(_)  → proven(limit)                # the attacker has won
    limit ≤ 1    → disproven(limit)             # no attacking move left after this defence
    Forced(p)    → expand_defences([p])
    otherwise    → generate_defences: Terminal(n) → n | Moves(d) → expand_defences(d)
```

### Selection

`select_attack(state, attacks)` expands nothing; it reads the children's
table entries and returns a `Selection`:

| Field | |
| --- | --- |
| `node` | the node's own numbers: `min_pn_sum_dn` over the children, a child not yet in the table counting as `unexpanded_defence(attacks.len())` |
| `best` | the child with the smallest `pn` — the most-proving child |
| `best_child`, `second_child` | the numbers of the best and second-best child |

If a proven child turns up, `node` becomes `(0, INF)` on the spot.
`select_defence` mirrors it: smallest `dn`, `min_dn_sum_pn`, unexpanded
children count as `unexpanded_attack(defences.len())`, and `node.limit` is
`limit − 1`.

Seeding unexpanded children with the sibling count is a deliberate trick:
narrow nodes look easier, so the search prefers forcing lines.

### Expansion

`expand_attacks` is one loop, shared by all three modes:

```
expand_attacks(state, attacks, threshold):
    loop:
        s = select_attack(state, attacks)
        if s.node.pn ≥ threshold.pn or s.node.dn ≥ threshold.dn: return s   # exceeds_threshold
        if budget exhausted:                                     return s
        next = P::next_threshold_attack(s, threshold)
        into_play(s.best):
            result = search_defences(child, next)
            if budget not exhausted: attacker_table.insert(child, result)
```

`expand_defences` is the same with `select_defence`, the defender table,
`P::next_threshold_defence` and `search_attacks`. A node keeps expanding
its most-proving child until its own numbers cross the threshold its parent
handed down. The root's threshold is `no_threshold`, so the root loops until
decided.

The threshold is checked before the first expansion too. A node whose seeded
numbers already cross it returns at once, having only generated its moves,
and its parent learns its numbers that way. From #108 to #110 (2022)
`expand_attacks` skipped that first check and always expanded its best
attack once. Whether that pays depends a lot on the position. On the
benchmark (07) it makes `vct_unstable` 345 times cheaper (119.5M nodes →
346k), but the default set 39% dearer (24.4M → 34.0M) and
`vct_black_long_short` 53% dearer (17.4M → 26.7M). So it stays out (issue
#148).

### Threshold policies

`P: ThresholdPolicy` is the type parameter of `VCTSolver<P>`; the three
policies are zero-sized types, and `DFSVCTSolver`, `PNSVCTSolver`,
`DFPNSVCTSolver` are the aliases.

| Policy | Child threshold | Effect |
| --- | --- | --- |
| `DFSThreshold` | `no_threshold` | the chosen child is searched to a decision before the parent looks at another: plain depth-first search, proof numbers used only for move ordering |
| `PNSThreshold` | `(best_child.pn + 1, best_child.dn + 1)` | the child returns as soon as its numbers change, so the most-proving child is re-selected at every level after every expansion: best-first proof-number search inside a recursion |
| `DFPNSThreshold` | OR node: `pn = min(threshold.pn, second_child.pn + 1)`, `dn = threshold.dn − node.dn + best_child.dn`; AND node mirrored | df-pn (Nagai & Imai, 2002): stay in the best child as long as it stays best, never beyond the parent's own threshold |

### The solver struct

```rust
pub struct VCTSolver<P: ThresholdPolicy> {
    attacker_table: ProofTable,  defender_table: ProofTable,
    attacker_vcf: NestedVCF,     defender_vcf: NestedVCF,
    attacks_cache: LruCache<u64, Candidates>,  defences_cache: LruCache<u64, Candidates>,
    policy: PhantomData<P>,
}
```

That is all of its state; the methods above are `impl` blocks on it, one
file per phase. It implements `Solver` (04, §4): `solve` is
`advance_generation` on both tables and both nested solvers, `search`, and
on success `extract` with an unlimited budget. `clear` empties all six
fields; `memo_len` sums the four memos (the caches are bounded already).

## 6. Extraction (`extractor.rs`)

`search` only proves that a win exists. `extract` walks the tables again to
recover the line:

```
extract_attacks(state):                        # attacker to move
    Forced(p)  → play p, extract_defences
    else       → the first empty point whose attacker_table entry is proven: play it, extract_defences
    none       → attacker_vcf.vcf(state)      # proven by the VCF shortcut in compute_attacks: its path is the tail

extract_defences(state):                       # defender to move
    Defeated(end) → Mate { end, path: [] }
    Forced(p)     → play p, extract_attacks
    else          → among threat_defences(attacker_vcf.threat(state)), the proven child with the smallest Node::limit
    none proven   → Mate { end: Unknown, path: [] }   # proven because no legal defence existed
```

Picking the proven defence with the smallest `limit` picks the one that made
the attacker use the most moves, so the reported line is against the most
stubborn defence.

## 7. Walk-through

`test_vct_black` (No. 02 of Hiroshi Okabe's five-move problems), Black to
move:

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

`solve(VCTDFS, &board, Black, SolveLimits::new(4).with_threat_limit(1))` —
and likewise `VCTPNS`, `VCTDFPNS` — proves it with the line
`F10,G9,I10,G10,H11,H12,G12` and end `Fours(F13, K8)`; `limit = 3` is
`Disproven`.

| Move | Why it is a threat / forced | `limit` after |
| --- | --- | --- |
| `F10` (Black) | three `F10,_,H8,I7`: after a pass `G9` is a straight four, a one-move VCF | 4 |
| `G9` (White) | in `threat_defences`: the path of the threatened VCF | 3 |
| `I10` (Black) | three `F10,_,H10,I10` | 3 |
| `G10` (White) | blocks it | 2 |
| `H11` (Black) | four `H8..H11` | 2 |
| `H12` (White) | `Forced` | 1 |
| `G12` (Black) | `G12,H11,I10,J9` with `F13` and `K8` open: `Fours` | 1 |

At the root, `generate_attacks` orders the empty points by potential (§8):
`I10` (18), `G10` (16), `G9` (13), `F10` (12), `I8` (12), `H11` (10), …
The depth-first mode therefore tries `I10` first. It is refuted in one ply:
`I10` makes no three, so at the defender node `attacker_vcf.threat` finds
no one-move VCF and `compute_defences` returns `Terminal(disproven)`. The
refutation goes into `attacker_table`, `select_attack` moves on, and `F10`
is eventually proven.

## 8. `PotentialField` (`src/analysis/potential.rs`)

The generators need "how useful is a stone here for the attacker?" for
every empty point, cheaply and always current. `PotentialField` keeps one
`u8` per direction per point (`Potential { v, h, a, d }`) and reports the
sum.

**Per-direction value.** From `Board::potentials(player, min, exact)`
(02, §7): for each five-window through the point with no opponent stone
(and, for Black, `exact` — no own stone in the margin, which would make an
overline), count the own stones it would hold after playing there; keep the
count if it is at least `min`; report `max × (number of windows reaching
that max)`. With `min = 2`, a single own stone within four cells already
scores.

**Keeping it current.** `init(player, min, board)` fills the field;
`update_along(p, board)` zeroes the four lines through `p` (`reset_along`)
and recomputes them (`potentials_along`). For lazy use, `mark_stale(p)`
only notes the point and `sync(board)` runs `update_along` on every point
noted since. `VCTState` marks each point played or taken back and syncs
before reading the field — four line scans
per such point instead of a board pass, and none for the many nodes whose
candidates come from the cache.

**Querying.** `get(p)` is the sum over the four directions; `collect(min)`
lists every point whose sum is at least `min`. `VCTState::sorted_potentials`
and `sort_by_potential` sort descending. Note the two different `min`s: `2`
at construction filters windows, `3` at query time filters sums.

**Overlay.** `overlay(board)` prints the field for debugging (`.` = 0). For
the board of §7 with `PotentialField::init(Black, 2, ..)`:

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

`I10` (18) and `G10` (16) lie on two of Black's lines at once, which is why
they head the candidate list. The field is the *attacker's* even when
ordering defences: a defence landing on the attacker's best point is tried
first.

## 9. Cheat sheet

| Question | Where to look |
| --- | --- |
| Why did the search stop at depth N? | `limit` counts attacker moves; `search_defences` returns `disproven` at `limit ≤ 1` |
| A threat is not recognised | `compute_defences` step 1, `attacker_vcf.threat` with depth `threat_limit`; the nested VCF sees only `Sword` eyes |
| A defence is missing | `VCTState::threat_defences`: path, `end_breakers`, `counter_defences`, `four_moves` |
| A counter-attack refutation is missing | `defender_vcf.vcf` is bounded by `defender_vcf_depth` (2); deeper counter-VCFs are found only if a counter-four is in `threat_defences`. Raise it with `SolveLimits::with_defender_vcf_depth` |
| Move ordering | `PotentialField` with `min = 2`; attack candidates need a sum `≥ 3` |
| Which mode does what | `ThresholdPolicy` in `threshold.rs`; everything else is shared |
| Transposition tables | `attacker_table` / `defender_table` (`ProofTable`), the two `LruCache`s, the nested solvers' `deadends`; all keyed by `State::key()`, decisions by position alone |
| Why do decisions carry between limits only from some depth? | `transfer_from` in `ProofTable` (§4) |
| Path extraction | `extract`; `End::Unknown` means no proven child to follow |
| Adding a regression case | ASCII board + expected path in `solve.rs`, one assertion per relevant `SolveMode` |

*History.* An experimental "lazy" VCT solver (`vct_lazy/`, after 長井歩,
GPW 2011) that interleaved the threat check with the main search used to
live beside `vct/`. It was removed in
[renju-note/quintet#133](https://github.com/renju-note/quintet/pull/133),
which records its state and the measurements behind the removal.
