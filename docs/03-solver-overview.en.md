# Solver overview: `src/mate/` and the VCF search

This document is the entry point to the mate solvers. It covers the public
`solve` function and its parameters, the search state shared by every
solver, and the VCF (Victory by Continuous Fours) solver. The VCT (Victory
by Continuous Threats) solvers, which build on all of this, are described
in [04-solver-algorithm-vct.en.md](04-solver-algorithm-vct.en.md).

It builds on the board vocabulary from
[02-board-implementation.en.md](02-board-implementation.en.md) — in particular
`Four`, `Sword`, `Three`, `eyes()`, forbidden moves and `zobrist_hash_n` — and
is written against the current code; identifiers in backticks can be grepped.

Module map:

| File | Role |
| --- | --- |
| `mate/solve.rs` | Public entry points `solve` / `solve_limited`, `SolveMode`, `SolveLimits`, `SolveResult`; input validation; the solver regression tests. |
| `mate/budget.rs` | `NodeBudget`: how much work a search may do, counted in nodes. |
| `mate/game.rs` | `Game`: board + move history + side to move, with pass support; `check_event` (four detection); `End`. |
| `mate/state.rs` | `State` trait: play/undo that also tracks the remaining `limit`, and the transposition key. |
| `mate/mate.rs` | `Mate`: the result (`End` + move path). |
| `mate/memo.rs` | `Memo`: what the solvers remember between searches, and the generations that bound it. |
| `mate/vcf/` | VCF solver: `VCFState` (four-making move pairs), `DFSSolver`, `IDDFSSolver`. |
| `mate/vct/` | VCT solvers (`DFSVCTSolver`, `PNSVCTSolver`, `DFPNSVCTSolver`); see 04. |
| `analysis/field.rs` | `PotentialField`, move ordering for VCT; see 04, §9. |

---

## 1. Entry points (`solve.rs`)

```rust
pub fn solve(mode: SolveMode, limit: u8, board: &Board, attacker: Player, threat_limit: u8) -> Option<Mate>
pub fn solve_limited(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

`attacker` is the side to move and the side we try to prove a win for. The
other side is called the *defender* throughout the code.

`solve` is the short form and is what `wasm.rs` calls; `solve_limited` is the
same search with a node budget and a three-valued answer. `solve` is written
in terms of it.

### `SolveMode`

| `SolveMode` | CLI name | Solver | Notes |
| --- | --- | --- | --- |
| `VCFDFS` | `vcf` | `vcf::DFSSolver` | `threat_limit` is ignored. |
| `VCFIDDFS` | `vcf_iddfs` | — | Reserved: `solve` currently returns `None`. |
| `VCTDFS` | `vct` | `DFSVCTSolver` | Depth-first traversal of the same AND/OR tree as the two below. |
| `VCTIDDFS` | `vct_iddfs` | — | Reserved: `solve` currently returns `None`. |
| `VCTPNS` | `vct_pns` | `PNSVCTSolver` | Best-first proof-number search. |
| `VCTDFPNS` | `vct_dfpns` | `DFPNSVCTSolver` | df-pn (depth-first proof-number search). The default in practice. |

### `SolveLimits`: how far a search may go

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)
    .with_defender_vcf_depth(depth)   // default 2
    .with_max_nodes(nodes)            // default: no budget
```

- `limit` is the maximum number of **attacker moves** in the solution. Every
  attacking move counts, including fours. A solution path of 7 moves (4
  attacks and 3 defences) needs `limit >= 4`; see `test_vct_black`. The
  example in §3 shows how the value is consumed move by move.
- `threat_limit` bounds the depth of the nested VCF searches that decide
  whether a move is a threat (04, §2). It does not affect `VCFDFS`.
- `defender_vcf_depth` bounds the nested VCF that looks for the defender's
  counter-VCF (04, §2). It used to be fixed at 2, which is still the
  default (`DEFAULT_DEFENDER_VCF_DEPTH`).
- `max_nodes` is the budget; see §1.1.

The `with_*` methods return a new value, so adding a field later does not
break callers that build their limits this way.

### `SolveResult` and `NodeBudget`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` cannot say why there is no mate. `SolveResult` can:
`Disproven` means there is none within the limits, `Aborted` means the
search ran out of budget and the position is still open. A caller that
searches with a budget must treat the two differently — taking `Aborted`
for "safe" is how an engine walks into a mate.

A `NodeBudget` counts nodes, not time: everything under `src/` compiles for
`wasm32-unknown-unknown`, where there is no clock, so a caller with a time
control converts it into a number of nodes itself. One node is one call of
`DFSSolver::solve` (VCF) or of `search_attacks` / `search_defences` (VCT),
nested VCF searches included.

The solvers take `&mut NodeBudget`, so one budget can bound a whole group of
related searches:

```rust
let budget = &mut NodeBudget::new(100_000);
let mut solver = DFPNSVCTSolver::init(threat_limit, DEFAULT_DEFENDER_VCF_DEPTH);
for board in candidates {
    let state = &mut VCTState::init(&board, attacker, limit);
    match solver.solve(state, budget) {
        Some(mate) => /* proven */,
        None if budget.is_exhausted() => break,   // out of budget, nothing proved
        None => /* no mate within the limits */,
    }
}
```

An exhausted budget stays exhausted until `restart()`, so every later call
gives up at once instead of doing a little more work each time.

A search that gave up proves nothing, so nothing it computed is written to
the memo tables: not `DFSSolver::deadends`, not the VCT proof tables, not
the candidate-move caches. A solver whose search was aborted can therefore
be reused as is, and `test_abort_leaves_no_wrong_memo` checks exactly that.
Proofs are never spurious, though: a mate found just before the budget ran
out is a real mate and is returned.

### Using the solvers directly

`solve`/`solve_limited` build a solver, use it once and drop it. A caller
that asks many related questions — a game engine, an analysis view — should
keep one instead, because the tables it fills are most of its value. The
solvers, the states and `Game` are all public for that:

- Just keep asking. Every memo is keyed by the attacker as well as the
  position, so one solver can answer about both sides without its answers
  being confused, and `solve` opens a new generation each time so the memos
  settle at about two searches' worth however many searches are asked
  (`Memo` in `mate/memo.rs`). `VCTSolver::with_carry_capacity` sets how much
  is carried; `memo_len()` reports what is held.
- Repeated questions are what reuse makes cheap: asking the same thing again
  costs a few nodes rather than the whole search. Two *different* positions
  share much less than one might hope, because the remaining `limit` is part
  of the key, so the same board reached from two roots is two entries unless
  the roots are the same depth apart from it.
- `VCTSolver::clear()` throws away both proof tables, both candidate caches
  and the nested VCF deadends. Nothing requires it — use it to hand a solver
  on with a clean slate, or to give the memory back.
- `VCTState::threat_defences(&threat)` lists the moves worth trying against
  a mate that was found — the threat's own path, the points that break its
  end, counter-fours and the defender's own four-making moves. It is what
  the VCT search itself generates defences from (04, §3), and what a
  defender should try.
- `Game::play(None)` is a pass, which is how the "is this a threat?"
  question is asked: let the side to move pass and see whether the opponent
  then has a VCF. `test_threat_after_pass` does it move by move.

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
  (see `validate` above and the extractor in 04, §6).

`Mate::n_moves()` is the path length and `n_times()` the number of attacker
moves in it.

## 2. Shared search state (`game.rs`, `state.rs`, `mate.rs`)

### `Game`

`Game` owns a `Board`, the list of moves played during the search
(`Vec<Option<Point>>`, `None` being a pass) and the side to move `turn`.
`play` / `undo` mutate in place; `into_play(m, f)` plays `m`,
runs `f`, undoes, and returns `f`'s result — this is the pattern every
solver uses to walk the tree without cloning boards.

A pass (`play(None)`) is used to ask "what could the *other* side do if I
did nothing?": it is how threats are detected in VCT (04, §2).

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

`State` (implemented by `VCFState` and `VCTState`) wraps a
`Game` with `attacker` and the remaining `limit`:

- `play(m)` plays `m` on the game and, if it is now the attacker's turn
  again (i.e. the defender just moved), decrements `limit`. `undo` reverses
  both. So `limit` is "attacker moves still allowed", and at a defender node
  it still includes the attack that was just played. A hook `after_play` /
  `after_undo` lets `VCTState` refresh its potential field.
- `attacking()` is `turn == attacker`.
- `zobrist_hash()` is the key every memo table in the solvers uses, and it
  carries four things: the stones, whose turn it is, the remaining `limit`
  and the `attacker`. `Game::zobrist_hash(n)` supplies the first three
  (`Board::zobrist_hash_n(n)` plus the turn) and `State::zobrist_hash` adds
  the attacker. So the same position reached with a different budget is a
  different entry, a pass is a different entry from the position before it,
  and a question about Black's mate is a different entry from the same
  question about White's.

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
`IDDFSSolver::init(vec![1])` — "check for a one-move win first".

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

Following `limit` along this path shows how the budget is spent. It starts
at 3 and drops by one each time White answers, i.e. each time the turn
returns to Black:

| Position | `limit` | Note |
| --- | --- | --- |
| root, Black to move | 3 | |
| after `I8`, White to move | 3 | the attack just played is still counted |
| after `G8`, Black to move | 2 | |
| after `I9`, Black to move | 1 | |
| after `J9`, White to move | 1 | `check_event` reports `Defeated` |

With `limit = 2` the same search fails: after `I9` the limit is 0, and
`solve` returns `None` before `J9` is ever tried. A VCF of three fours
needs `limit >= 3`, whether from the CLI or in a test.

## 4. Cheat sheet

| Question | Where to look |
| --- | --- |
| Add a new solve mode | `SolveMode` + `FromStr` + the `match` in `solve` (`solve.rs`). |
| Why did the solver stop at depth N? | `limit` counts attacker moves and is decremented in `State::play` after each defender move. |
| A four-making move is not generated | `VCFState::move_pairs` only looks at `Sword` eyes; for Black, `exact` margins exclude overline-making fours. |
| A counter-four is mishandled | `Game::check_event` on the attacker's side and `VCFState::forced_move_pair`. |
| Transposition memo | `DFSSolver::deadends`, keyed by `zobrist_hash_n(limit)`; only failures are stored, and never after the budget ran out. |
| Stop a search that is taking too long | `SolveLimits::with_max_nodes`, or pass a `NodeBudget` to the solver directly; the answer is then `SolveResult::Aborted`. |
| Reuse a solver across positions | Keep the `VCTSolver` and pass it a fresh `VCTState` each time, for either attacker. `clear()` only to forget the tables. |
| Which moves defend against a mate | `VCTState::threat_defences`. |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode`. |
| VCT-specific questions | See the cheat sheet in [04-solver-algorithm-vct.en.md](04-solver-algorithm-vct.en.md). |
