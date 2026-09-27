# VCT: the threat search (`src/mate/vct/`)

A **VCT** (victory by continuous threats) is a line in which every attacking
move is a *threat*: a four, a three, or any move after which the attacker
would have a VCF if the defender did nothing. Unlike VCF the defender has a
choice of replies, so the tree is a real AND/OR tree and the solver searches
it with **proof numbers**. This document explains that search and the
`ShapeMap` that orders its moves.

Assumes [04](04-solver-framework.en.md) and [05](05-solver-vcf.en.md): in
particular `State`, `Key`, `Memo`, the `Solver` trait and `DFSSolver`.

```
src/mate/vct.rs         module doc: the algorithm in one page, re-exports, the three aliases
src/mate/vct/
├── state.rs            VCTState: Game + attacker + limit + ShapeMap + SwordMap;             (§2, §3, §8)
│                       threat_defences, sorted_attacks / sorted_defences, priority
├── nested_vcf.rs       NestedVCF: one side's VCF sub-search                                  (§2)
├── generator.rs        generate_attacks / generate_defences → Candidates                     (§3)
├── proof.rs            Node (proof numbers), ProofTable (transposition table)                (§4)
├── searcher.rs         search_attacks / search_defences, expand_attacks / expand_defences,   (§5)
│                       select_attack / select_defence → Selection
├── threshold.rs        ThresholdPolicy: DFSThreshold, PNSThreshold, DFPNSThreshold           (§5)
├── solver.rs           VCTSolver<P>: the struct, Solver impl                                 (§5)
└── extractor.rs        extract: the winning line from the tables                             (§6)
src/feature/shape.rs       ShapeMap: what a stone would make, per point, player and direction    (§8)
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
│   │     attacker_vcf.vcf      has a VCF?          → Terminal(proven), else its zone
│   │     defender_vcf.threat   must parry a threat? → only threat_defences
│   │     points making at least a Two, by priority                    §8
│   │     minus those the zone rules out as threats
│   └── expand_attacks: loop { select_attack; play best; search_defences; store in attacker_table }
│
│   search_defences (AND node, defender to move)
│   ├── check_event: Defeated → proven; limit ≤ 1 → disproven; Forced(p) → defences = [p]
│   ├── generate_defences                                              §3
│   │     attacker_vcf.threat   was that a threat?  → else Terminal(disproven)
│   │     defender_vcf.vcf      defender wins first? → Terminal(disproven)
│   │     threat_defences, by priority                                 §8
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
| 0 | fours only — the nested VCF has no depth, so the threat check always fails and only defender nodes that are `Forced` (must block a four) go on |
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
pub struct VCTState {
    game: Game, pub attacker: Player, pub limit: u8,
    shapes: ShapeMap, swords: SwordMap,
}
```

`shapes` is the `ShapeMap` (§8) of both players. `after_play` /
`after_undo` only mark the move's lines stale; they are recomputed when
next read (`sorted_attacks` / `sorted_defences`), which happens only on a
candidate-cache miss. `next_key(m)`
is `key()` of the child after `m`, computed from the board's Zobrist hash by
XOR without playing `m`, which is what keeps table lookups for unexpanded
children cheap. `VCTState` also keeps a `SwordMap` (02 §7), marked the same way;
`vcf_state` / `threat_state` sync it and hand a clone to the nested
`VCFState`, so that each nested VCF starts in sync and the next one reuses
what this one computed.

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
    pub fn vcf_or_zone(&mut self, state: &mut VCTState, budget) -> Result<Mate, Area>; // the same, or where a stone could give it one
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
    Moves(Vec<Candidate>),  // an inner node: the moves to try, best first
    Terminal(Node),         // decided here without expanding: Node::proven or Node::disproven
}
pub struct Candidate { pub point: Point, pub estimate: u32 }
```

A `Candidate`'s `estimate` is the number its child starts from while the
tables know nothing about it: the proof number of an attack, the disproof
number of a defence (§5, *Selection*).

Results are cached in an `LruCache` of 1000 entries per generator, keyed by
`zobrist_hash()` (position *and* limit, because the nested VCF depth is
`min(limit, depth)`), and never cached when the budget ran out.

**`compute_attacks`** (attacker to move):

1. `attacker_vcf.vcf_or_zone` — a VCF now? → `Terminal(proven)`. Not
   needed for correctness (the main search would find the fours one
   `Forced` reply at a time) but much faster. If not, it hands back the
   search's zone (05, §2): the points where one more attacker stone could
   give the attacker a VCF.
2. `defender_vcf.threat` — would the defender have a VCF if the attacker
   passed? If so the attack must also parry it: restrict candidates to
   `threat_defences(threat)`.
3. Candidates are the points where the attacker makes at least a `Two`
   along some line in the `ShapeMap` (`sorted_attacks(only)`), in order of
   `priority` (§8). Their number is the `width`.
4. Of those, keep the ones that may be threats (`may_threaten`): in the
   zone, or in a segment already holding two attacker stones (at least a
   `Sword` along some line in the `ShapeMap`, §8), which the zone leaves
   to the caller. Any other move leaves the attacker without a VCF after
   a pass, so it is no threat. Skipped when the budget ran out, as the
   zone is then incomplete.
   None left → `Terminal(disproven)`.
5. Each gets its estimate from the biggest shape it makes (`best_shape`,
   from the `ShapeMap`, §8): `width / 4` for a four, `width / 2` for a
   three, `width` for any other (§5, *Selection*).

Candidates are otherwise *not* checked for being threats here. That
happens one ply down, at the defender node, where a non-threat is refuted
at once — for a nested VCF search and a node each, which step 4 saves for
about half the non-threats on the benchmark (where no threat ever fell
outside the zone).

The `width` is taken before step 4 because it seeds the proof numbers of
the unexpanded children (§5): a move that is no threat would only be
disproven, never make its siblings easier to prove, and seeding with the
count after pruning tips the search towards other lines than it would
otherwise take.

**`compute_defences`** (defender to move):

1. `attacker_vcf.threat` — would the attacker have a VCF if the defender
   passed? If not, the last attack was no threat → `Terminal(disproven)`.
2. `defender_vcf.vcf` — does the defender have a VCF of their own (within
   `defender_vcf_depth`)? Then the defender wins first → `Terminal(disproven)`.
3. Candidates are `threat_defences(threat)` in order of `priority` (§8)
   (`sorted_defences`), minus forbidden moves. None left → `Terminal(proven)`: the threat cannot be
   answered.
4. Each gets the estimate `n`, the number of candidates.

### `threat_defences`

`VCTState::threat_defences(&threat)` is the heuristic list of moves that
might stop a threatened VCF, in this order:

| Part | Points |
| --- | --- |
| the path | every stone of the threat — the attacker's fours and the defender's blocks; occupying any of them breaks the sequence |
| `end_breakers(end)` | for `Fours(p1, p2)` the two winning points; for `Forbidden(p)` the point itself plus every empty point within 5 along its four lines (`neighbors(p, 5, true)`), since a stone nearby can change whether `p` is forbidden |
| `counter_defences(threat)` | replay the threat after a pass and, at each of the defender's blocks in it, collect the eyes of the defender's `Sword`s through that block — points where the defender would get a four during the sequence, which played now may become a counter-four |
| `four_moves()` | every four-making move (`Sword` eye) the defender has right now — counter-fours that force the attacker to answer |

The list may repeat a point; `sorted_defences` keeps only the first of
each.

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
| `Node::unexpanded_defence(n, limit)` | `(n, 1)` | first guess for an unexpanded child of an OR node (defender to move); `n` = the attack's estimate (§3) |
| `Node::unexpanded_attack(n, limit)` | `(1, n)` | first guess for an unexpanded child of an AND node (attacker to move); `n` = the defence's estimate |

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
`lookup_children(state, candidates)` is `lookup_next` for each candidate,
what an expansion starts from (§5, *Expansion*).

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

`select_attack(limit, attacks, children)` expands nothing; it reads the
children's numbers as the attacker table has them (`children`, kept by the
expansion below) and returns a `Selection`:

| Field | |
| --- | --- |
| `node` | the node's own numbers: `min_pn_sum_dn` over the children, a child not yet in the table counting as `unexpanded_defence(estimate)` (§3; `1` for a forced move) |
| `best` | the child with the smallest `pn` — the most-proving child — as an index into the candidates |
| `fresh` | whether `best` is not in the table yet: it has never been searched |
| `best_child`, `second_child` | the numbers of the best and second-best child |

If a proven child turns up, `node` becomes `(0, INF)` on the spot.
`select_defence` mirrors it: smallest `dn`, `min_dn_sum_pn`, unexpanded
children count as `unexpanded_attack(estimate)`, and `node.limit` is
`limit − 1`.

The estimates (§3) are a deliberate trick. They start from the number of
siblings, so that narrow nodes look easier and the search prefers forcing
lines. An attack's is then lowered by how far the move narrows the
defender's replies: a four leaves one and starts at a quarter of the
sibling count, a three leaves a few and starts at half of it. So the
search follows the fours and threes further before it turns to the
quieter attacks, wherever they stand in the candidate order.

What was tried on the benchmark (07), against the sibling count alone:

- Favouring fours is most of the gain, and also its one cost. Without it
  (threes only at half) the `heavy` cases got 17% dearer. With it,
  `vct_small_but_long` gets about three times dearer, whatever the other
  weights: at `limit` 255 nothing stops the search from following fours
  ever deeper.
- Raising the other attacks' estimates above the sibling count (to one and
  a half times it) paid before the attacks that cannot be threats were
  ruled out, and only costs since.
- Proof numbers are compared across nodes, so the scale matters as well as
  the ratio: halving every estimate alike makes the search dearer.
- Weighting defences by the fours and threes they make did not pay.

With these weights the whole set, `heavy` included, takes 38% fewer
nodes, and the `heavy` cases 47% fewer (`vct_unstable` 109.5M → 41.6M).
The default set takes 6% more in total because of `vct_small_but_long`
(5.8M → 16.7M); without it, 26% fewer.

### Expansion

`expand_attacks` is one loop, shared by all three modes:

```
expand_attacks(state, attacks, threshold):
    children = attacker_table.lookup_children(state, attacks)
    loop:
        s = select_attack(limit, attacks, children)
        if s.node.pn ≥ threshold.pn or s.node.dn ≥ threshold.dn: return s   # exceeds_threshold
        if budget exhausted:                                     return s
        if s.fresh and s.best is forbidden:
            into_play(s.best): attacker_table.insert(child, disproven)
            children[s.best] = disproven
            continue
        next = P::next_threshold_attack(s, threshold)
        into_play(s.best):
            result = search_defences(child, next)
            if budget not exhausted: attacker_table.insert(child, result)
        children[s.best] = result
```

The children's numbers are read from the table once, when the expansion
starts, and after that the loop writes back only the child it has just
searched. That is exactly what reading the table again would give:
searching one child cannot change a sibling's entry, as every position
under the child holds the child's stone and no sibling does (the VCT
search never passes). When the loop read the table for every child at
every step, the lookups were a tenth of the time in a profile of
`vct_small_but_long`; reading them once takes 4% off the whole benchmark
(5% off the default set), with the same nodes.

`compute_attacks` leaves forbidden moves among the candidates: few are
forbidden (about one in three hundred on the benchmark), and most
candidates are never searched. An attack is only asked about when it is
first about to be searched, and a forbidden one is disproven in the table
without a node. Until then it counts in its parent's numbers like any
other unexpanded child, so the search takes other lines than when the
forbidden moves were left out beforehand.

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
| `DFPNSThreshold` | OR node: `pn = min(threshold.pn, second_child.pn + 1 + second_child.pn / 8)`, `dn = threshold.dn − node.dn + best_child.dn`; AND node mirrored | df-pn (Nagai & Imai, 2002): stay in the best child as long as it stays best, give or take a margin, never beyond the parent's own threshold |

The margin of `second_child.pn / 8` is the "1 + ε trick" (Pawlewicz & Lew,
2006) with ε = 1/8. Plain df-pn lets the best child grow only one past the
second-best, so when the two are close the search keeps switching between
them, going back up to the parent and down again each time. With the margin
it stays a while longer in the one it is in. On the benchmark (07) that took
the whole set, `heavy` included, from 137.3M nodes to 99.4M (−28%): −19% on
the default set, −27% on the other `heavy` cases, and `vct_unstable` 40.2M →
25.4M. Three cases got 6 to 14% dearer, none of them over 170k nodes.

The other ε tried, as the change in nodes over the whole set:

| ε | whole set | default set | `vct_small_but_long` | `vct_unstable` |
| --- | --- | --- | --- | --- |
| 1/16 | −14% | −8% | +5% | −7% |
| 1/8 | −28% | −19% | −14% | −37% |
| 1/6 | −20% | −18% | −9% | −7% |
| 1/4 | −8% | −25% | −25% | +47% |
| 1/3 | −25% | −11% | +11% | −23% |
| 1/2 | −10% | −25% | −22% | +41% |

From 1/4 up the cases other than those two gain a little more (about −32%
against −26% at 1/8), but the two swing widely: they are the ones whose
search follows a long line of fours, where any change to the order of
expansion sends it down another. 1/8 is the one that does well on both.

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

At the root, `generate_attacks` orders the candidates by `priority` (§8):
`H11` (10: a four and a two, 5, plus 5 for the four), `H12` (9), `F10` (6:
a three and a two, 4, plus 2 for the three), `G9`, `J6`, `K5` (5 each), …
The depth-first mode therefore tries `H11` first. White's `H12` is forced,
and with that four spent Black has no VCT in the three attacks left, so it
is disproven. The refutation goes into `attacker_table`, `select_attack`
moves on, `H12` fares no better, and `F10` is proven.

## 8. Move ordering (`src/feature/`, `VCTState::priority`)

Among candidates with the same estimate (§3), the order decides which
child the search expands first. A cache of `src/feature/` describes the
points, and `VCTState::priority` weighs them.

### `ShapeMap` (`src/feature/shape.rs`)

The `ShapeMap` says, per player and per empty point, what a stone there
would make on each of the four lines through it, a `Shape`:

| `Shape` | The point is | From (02 §3.1) |
| --- | --- | --- |
| `Five` | the eye of a `Four` | `row_eyes(r, Four)` |
| `Four` | an eye of a `Sword` | `row_eyes(r, Sword)` |
| `Three` | an eye of a `Two` | `row_eyes(r, Two)` |
| `Sword` | in a segment holding two | `eyes_of(scoring(r, 2), ..)` |
| `Two` | in the shared cells of an open pair of segments holding one | `eyes_of(open_starts(r, 1), ..)` |
| `Nothing` | none of the above, or occupied | |

A cell takes the biggest that applies. It is kept like the `SwordMap`
(02 §7): per line, marked stale on a move and recomputed on `sync`, a few
bit operations per line and player.

`get(p, r)` returns the four as `Shapes`, which counts them
(`count`, `count_from`), sums them by rank (`total`: `Two` 1 up to
`Five` 5), drops one direction (`except`) and guesses whether
Black may not play there (`looks_forbidden`: two fours or two threes and no
five; it does not see a double-four on one line, an overline, or that a
three is fake). `forbidden_eyes(board, p, r)` counts the fours and threes
`r` makes at `p` that leave an eye where Black's stone looks forbidden: for
White, a four Black cannot block or a three Black has fewer ways to stop;
for Black, a three that cannot become a straight four there. The stone at
`p` is on no other line through the eye than the row's, so Black's shapes
along the others are read as they are.

### `priority`

`sorted_attacks` / `sorted_defences` sort by `VCTState::priority(p)`,
highest first; equal ones keep their order. It is what the move makes for
the side to move, after the way a player sizes up a move:

| Term | Value | Why |
| --- | --- | --- |
| attack only: every direction, by its shape | `Shapes::total` | the next threats are made of twos and swords too |
| each `Four` | +5 | narrows the opponent's replies to one |
| each `Three` | +2 | narrows them to a few |
| each direction at `Sword` or more, beyond the first | +5 | threats along several lines at once (four-three, double threats) |
| White: each four / three with an eye Black looks forbidden at | +20 / +10 | Black cannot answer there |
| Black: each three whose straight-four point looks forbidden | −2 | the three may be fake |
| Black: the point itself `looks_forbidden` | −20 | Black cannot play it if it really is forbidden, which is only found when it is first searched (§5); if it is not, the threes are fewer than they look |

For a defence the first term is left out. Every defence here stops the
threat one way or another (`threat_defences`), and ordering them by how
much the attacker wants the point, by its stones or its shapes there, made
the search larger on the benchmark (07) than these terms alone. The
weights were tuned on the benchmark too.

The first term replaced a `PotentialField`, per point the attacker's
stones in the live segments through it, which the defences were ordered by
too. With the fours and threes above, it barely changed the attacks'
order, the defences did better without it, and keeping it in step with the
board took about a fifth of the search's time.

## 9. Cheat sheet

| Question | Where to look |
| --- | --- |
| Why did the search stop at depth N? | `limit` counts attacker moves; `search_defences` returns `disproven` at `limit ≤ 1` |
| A threat is not recognised | `compute_defences` step 1, `attacker_vcf.threat` with depth `threat_limit`; the nested VCF sees only `Sword` eyes |
| A defence is missing | `VCTState::threat_defences`: path, `end_breakers`, `counter_defences`, `four_moves` |
| An attack is missing | a `Two` or more along some line (`sorted_attacks`), then `VCTState::may_threaten` over the zone of `DFSSolver::search_zone` (05, §2) |
| A counter-attack refutation is missing | `defender_vcf.vcf` is bounded by `defender_vcf_depth` (2); deeper counter-VCFs are found only if a counter-four is in `threat_defences`. Raise it with `SolveLimits::with_defender_vcf_depth` |
| Move ordering | `VCTState::priority` over the `ShapeMap` (§8); attack candidates need a `Two` or more along some line |
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
