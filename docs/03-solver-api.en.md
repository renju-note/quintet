# Using the mate solvers

Read this if you *call* the solvers: from `wasm.rs`, from the CLI in
`examples/solve.rs`, or from Rust code that keeps a solver alive across
many questions. Nothing here requires knowing how the search works; that is
[04](04-solver-framework.en.md), [05](05-solver-vcf.en.md) and
[06](06-solver-vct.en.md).

Vocabulary from [01](01-renju-rules.en.md) is assumed: four, straight four,
three, forbidden move. Everything below lives in `src/mate/` and is
re-exported from `quintet::mate`.

## 1. The question a solver answers

Every call asks the same thing:

> On this `board`, with `attacker` to move, does the attacker have a
> **mate** — a forced win — within `limit` attacking moves?

- The *attacker* is the side to move and the side we try to prove a win
  for. The other side is the *defender* throughout the code. Either colour
  can be the attacker.
- A mate is a **VCF** (victory by continuous fours: every attacking move is
  a four) or a **VCT** (victory by continuous threats: every attacking move
  is a threat, a four or a move that would otherwise become a VCF). Which
  one is asked for is the `SolveMode`.
- `limit` counts the attacker's moves only. A winning line of 7 stones
  (4 attacks, 3 defences) needs `limit >= 4`.

A positive answer is a [`Mate`](#5-mate-and-end): the winning line and why
the defender is lost at the end of it.

## 2. One-shot: `solve`

```rust
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

It builds a solver, runs it once and drops it. `limits` carries the depths
and an optional node budget, and the answer is three-valued. `wasm.rs` calls
it too, reducing the answer to the winning line or nothing.

```rust
use quintet::board::{Board, Player};
use quintet::mate::{SolveLimits, SolveMode, SolveResult, solve};

let board: Board = "H8,I9,J9,H7".parse().unwrap();
let limits = SolveLimits::new(5).with_threat_limit(2).with_max_nodes(100_000);
match solve(SolveMode::VCTDFPNS, &board, Player::Black, limits) {
    SolveResult::Proven(mate) => { /* mate.path is the winning line */ }
    SolveResult::Disproven => { /* no mate within the limits */ }
    SolveResult::Aborted => { /* out of budget: still unknown */ }
}
```

`solve_with_stats` is the same call that also reports what the search cost:

```rust
let (result, stats) = solve_with_stats(SolveMode::VCTDFPNS, &board, Player::Black, limits);
stats.nodes      // nodes visited, as NodeBudget counts them (§3)
stats.memo_len   // entries left in the solver's memos (Solver::memo_len, §4)
```

Both numbers are deterministic: the same arguments give the same numbers on
every run and every machine. That makes them the measure to compare solver
changes by, which is what the benchmark does (07).

### `SolveMode`

| `SolveMode` | Code | CLI name | What it searches |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | VCF, depth-first. `threat_limit` is ignored. |
| `VCFIDDFS` | 1 | `vcf_iddfs` | Reserved; `solve` returns `Disproven`. |
| `VCTDFS` | 10 | `vct` | VCT, depth-first traversal of the proof-number tree. |
| `VCTIDDFS` | 11 | `vct_iddfs` | Reserved; `solve` returns `Disproven`. |
| `VCTPNS` | 15 | `vct_pns` | VCT, proof-number search. |
| `VCTDFPNS` | 16 | `vct_dfpns` | VCT, depth-first proof-number search (df-pn). **The one to use.** |

The codes are the wasm/JS representation (`SolveMode::try_from(u8)`) and are
public API: never renumber them. The CLI names are `SolveMode::from_str`.
The three VCT modes explore the same tree in different orders and reach the
same verdict; they differ in speed, and in which of several winning lines
they report first.

### `SolveLimits`: how far a search may go

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)     // default 0
    .with_defender_vcf_depth(depth)      // default DEFAULT_DEFENDER_VCF_DEPTH = 2
    .with_max_nodes(nodes)               // default: unlimited
```

| Field | Bounds | Applies to |
| --- | --- | --- |
| `limit` | attacker moves in the main line | every mode |
| `threat_limit` | attacker moves in the nested VCF that decides whether a move is a *threat* (06, §2). `0` recognises only fours, `1` fours and threes, `2`+ also moves that set up a four-three and deeper preparation (`test_vct_fukumi_move` needs `3`). | VCT |
| `defender_vcf_depth` | attacker moves in the nested VCF that looks for the *defender's* counter-VCF. | VCT |
| `max_nodes` | total work, in nodes (§3) | every mode |

The `with_*` methods return a new value, so fields added later do not break
callers that build their limits this way.

### `SolveResult`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` cannot say *why* there is no mate. `SolveResult` can, and the
difference matters: `Disproven` means there is none within the limits,
`Aborted` means the budget ran out and the position is still open. An engine
that takes `Aborted` for "safe" walks into mates. `is_proven()`,
`is_disproven()`, `is_aborted()`, `mate()` and `into_mate()` are the
accessors.

## 3. Budgets: `NodeBudget`

There is no clock. Everything under `src/` compiles for
`wasm32-unknown-unknown`, which has no `std::time`, so a caller with a time
control converts it into a number of nodes itself.

```rust
let mut budget = NodeBudget::new(100_000);   // or NodeBudget::unlimited()
budget.is_exhausted();                       // did a search run out?
budget.nodes();                              // nodes counted so far
budget.restart();                            // back to zero, same limit
```

One node is one visit of a search function: `DFSSolver::search` in VCF,
`search_attacks` / `search_defences` in VCT, the nested VCF searches
included. A budget stays exhausted until `restart()`, so a series of calls
sharing one budget stops as a whole rather than each doing a little more.

Two guarantees make a budget safe to use:

- **A proof is never spurious.** A mate found just before the budget ran
  out is a real mate and is returned.
- **An aborted search leaves nothing false behind.** Whatever it computed is
  not written to any memo (04, §5), so the same solver can be reused right
  away. `test_abort_leaves_no_wrong_memo` aborts one solver at a range of
  small budgets and checks that it still gives the right answer afterwards.

## 4. Keeping a solver: the `Solver` trait

`solve` throws its solver away, and with it the tables the search filled.
A caller that asks many related questions — a game engine, an analysis view
walking a variation — should keep one solver instead. All the pieces are
public for that:

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::init(threat_limit, DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // bounds the whole loop
for board in candidates {
    let state = &mut VCTState::init(&board, attacker, limit);
    match solver.solve(state, budget) {
        Some(mate) => { /* proven */ }
        None if budget.is_exhausted() => break,
        None => { /* no mate within limit */ }
    }
}
```

Every solver implements `Solver` (`mate/solver.rs`):

```rust
pub trait Solver {
    type State: State;
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;
    fn clear(&mut self);
    fn advance_generation(&mut self);
    fn memo_len(&self) -> usize;
}
```

| Solver | `State` | Constructor | Searches for |
| --- | --- | --- | --- |
| `DFSSolver` | `VCFState` | `init()` | VCF |
| `IDDFSSolver` | `VCFState` | `init(limits)` | VCF, at each of `limits` in turn |
| `VCTSolver<P>` — aliased `DFSVCTSolver`, `PNSVCTSolver`, `DFPNSVCTSolver` | `VCTState` | `init(threat_limit, defender_vcf_depth)` | VCT |

A state is built with `VCFState::init(&board, attacker, limit)` or
`VCTState::init(&board, attacker, limit)`; make a fresh one per question,
the solver is what carries over.

What to expect from reuse:

- **Ask about anything, in any order.** Every memo is keyed by the position,
  whose turn it is, the remaining limit *and* the attacker, so one solver
  can answer about Black's mate and White's mate on the same board without
  confusing the two (`test_reused_solver_both_attackers`).
- **Memory stays bounded.** Each `solve` opens a new *generation*; once a
  memo has grown past its carry capacity, entries older than the previous
  search are dropped. However many questions are asked, the memos settle
  at about two searches' worth (`test_reused_solver_memo_stays_bounded`).
  `with_carry_capacity(..)` on each solver sets the threshold
  (default `1 << 16` entries per memo); `memo_len()` reports what is held.
- **The same question again is nearly free** — a few nodes instead of the
  whole search (`test_reused_solver`).
- **The same position at another limit is cheap.** A decision is a bound:
  a mate within 4 is a mate within 5 and 6, no mate within 3 is no mate
  within 2 and 1. Deepening one position over limits 1..6 costs about a
  fifth of six fresh searches (`test_decisions_carry_between_limits`).
- **Different positions share little**, because the remaining limit is part
  of the key: the same board reached from two roots is two entries unless
  the roots are the same distance from it.
- `clear()` forgets everything. Nothing requires it — use it to hand a
  solver on with a clean slate or to give the memory back.
  `advance_generation()` is what `solve` calls first; only a caller driving
  the lower-level `search` / `extract` of a solver by hand needs it (04, §4).

### Two more questions a caller can ask

- **Which moves defend against this mate?** `VCTState::threat_defences(&mate)`
  lists the moves worth trying against a mate that was found: the mate's own
  path, the points that break its end, counter-fours and the defender's own
  four-making moves. It is the same list the VCT search itself generates
  defences from (06, §3).
- **Is this move a threat?** Let the side to move pass — `Game::play(None)` —
  and ask for the opponent's VCF: `VCFState::new(game, limit)` on the passed
  game, then `DFSSolver::init().solve(..)`. `test_threat_after_pass` does it
  move by move.

## 5. `Mate` and `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

`path` alternates attacker and defender moves, attacker first.
`Mate::n_moves()` is its length, `n_times()` the number of attacker moves in
it. `end` says why the defender is lost after the last move:

| `End` | The defender faces | Who can suffer it |
| --- | --- | --- |
| `Fours(p1, p2)` | fours with two different winning points `p1`, `p2` — one block is not enough. A straight four (which `Square` reports as two `Four`s with different eyes) or a double-four. | Either. As an attacker, Black gets it only from a straight four, because a double-four is forbidden for Black and never played. |
| `Forbidden(p)` | a single four whose only block `p` is a forbidden move. | Black only. |
| `Unknown` | The win was proven but the line could not be completed: the attacker already had a four before the search (§6), or the extractor found no proven child to follow (06, §6). | — |

## 6. What is checked before searching: `validate`

`solve` first rejects positions the solvers do not handle:

| Position | Result |
| --- | --- |
| Either colour already has a `Five` | `Disproven` |
| Black already has an `Overlined` | `Disproven` |
| The attacker already has a `Four` | `Proven(Mate { end: Unknown, path: [] })` — already won, nothing to prove |

A caller using a solver directly (§4) gets none of this and should check
what it needs itself.

## 7. Cheat sheet

| I want to… | Use |
| --- | --- |
| Get a yes/no/line once | `solve` |
| Know what a search cost | `solve_with_stats`; `SolveStats::nodes` and `memo_len` |
| Find only VCFs | `SolveMode::VCFDFS` |
| Find VCTs | `SolveMode::VCTDFPNS`; `threat_limit` decides what counts as a threat |
| Stop a search that takes too long | `SolveLimits::with_max_nodes`; treat `Aborted` as unknown, not as safe |
| Ask many questions cheaply | Keep a `DFPNSVCTSolver`, pass it a fresh `VCTState` per question, share one `NodeBudget` |
| Ask about both colours | The same solver; the attacker is part of every key |
| Know which defences to try | `VCTState::threat_defences(&mate)` |
| Know whether the last move was a threat | Pass with `Game::play(None)`, then ask for the opponent's VCF |
| Read the winning line from JS | `wasm::solve` returns the path as `u8` point codes (02, §1); `decode_x` / `decode_y` |
| Add a solve mode | `SolveMode`, its `TryFrom<u8>` and `FromStr`, and the `match` in `solve` (`solve.rs`) |
