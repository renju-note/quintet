# Using the mate solvers

For code that *calls* the solvers: `wasm.rs`, the CLI in
`examples/solve.rs`, or Rust that keeps a solver across many questions.
How the search works is in [04](04-solver-framework.en.md),
[05](05-solver-vcf.en.md) and [06](06-solver-vct.en.md); none of it is
needed here.

- Terms from [01](01-renju-rules.en.md) are assumed: four, straight four,
  three, forbidden move.
- Everything below is in `src/mate/`, re-exported from `quintet::mate`.

## 1. The question a solver answers

Every call asks:

> On this `board`, with `attacker` to move, does the attacker have a
> **mate** — a forced win — within `limit` attacking moves?

- The *attacker* is the side to move, the side we try to prove a win for.
  The other side is the *defender*. Either colour can attack.
- A mate is one of two kinds, chosen by `SolveMode`:
  - **VCF** (victory by continuous fours): every attacking move is a four.
  - **VCT** (victory by continuous threats): every attacking move is a
    threat — a four, or a move that would otherwise become a VCF.
- `limit` counts attacker moves only. A 7-stone winning line (4 attacks,
  3 defences) needs `limit >= 4`.

A positive answer is a [`Mate`](#5-mate-and-end): the winning line, and why
the defender is lost at its end.

## 2. One-shot: `solve`

```rust
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

- Builds a solver, runs it once, drops it.
- `limits` carries the depths and an optional node budget.
- The answer has three values. `wasm.rs` calls it too, and reduces the
  answer to the winning line or nothing.

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

`solve_with_stats` is the same call, plus what the search cost:

```rust
let (result, stats) = solve_with_stats(SolveMode::VCTDFPNS, &board, Player::Black, limits);
stats.nodes      // nodes visited, as NodeBudget counts them (§3)
stats.memo_len   // entries left in the solver's memos (Solver::memo_len, §4)
```

Both numbers are deterministic: the same arguments give the same numbers on
every run and machine. So they are what solver changes are compared by, as
the benchmark does (07).

### `SolveMode`

| `SolveMode` | Code | CLI name | What it searches |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | VCF, depth-first. `threat_limit` is ignored. |
| `VCFIDDFS` | 1 | `vcf_iddfs` | Reserved: `solve` searches nothing and returns `Aborted`. |
| `VCTDFS` | 10 | `vct` | VCT, depth-first over the proof-number tree. |
| `VCTIDDFS` | 11 | `vct_iddfs` | Reserved: `solve` searches nothing and returns `Aborted`. |
| `VCTPNS` | 15 | `vct_pns` | VCT, proof-number search. |
| `VCTDFPNS` | 16 | `vct_dfpns` | VCT, depth-first proof-number search (df-pn). **The one to use.** |

- The codes are the wasm/JS form (`SolveMode::try_from(u8)`) and public
  API: never renumber them.
- The CLI names are `SolveMode::from_str`.
- The three VCT modes explore the same tree in different orders and reach
  the same verdict. They differ in speed, and in which of several winning
  lines they report.

### `SolveLimits`: how far a search may go

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)     // default 0
    .with_defender_vcf_depth(depth)      // default DEFAULT_DEFENDER_VCF_DEPTH = 2
    .with_max_nodes(nodes)               // default: unlimited
```

| Field | Bounds | Applies to |
| --- | --- | --- |
| `limit` | Attacker moves in the main line. | every mode |
| `threat_limit` | Attacker moves in the nested VCF that decides whether a move is a *threat* (06, §2). `0`: fours only. `1`: fours and threes. `2`+: also moves that set up a four-three, and deeper preparation (`test_vct_fukumi_move` needs `3`). | VCT |
| `defender_vcf_depth` | Attacker moves in the nested VCF that looks for the *defender's* counter-VCF. | VCT |
| `max_nodes` | Total work, in nodes (§3). | every mode |

The `with_*` methods return a new value, so fields added later do not break
callers that build limits this way.

### `SolveResult`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` cannot say *why* there is no mate; `SolveResult` can.

- `Disproven`: there is no mate within the limits.
- `Aborted`: the budget ran out (or a reserved mode searched nothing). The
  position is still open. An engine that takes `Aborted` as "safe" walks
  into mates.
- Accessors: `is_proven()`, `is_disproven()`, `is_aborted()`, `mate()`,
  `into_mate()`.

## 3. Budgets: `NodeBudget`

There is no clock. Everything under `src/` compiles for
`wasm32-unknown-unknown`, which has no `std::time`. A caller with a time
control converts it into a number of nodes.

```rust
let mut budget = NodeBudget::new(100_000);   // or NodeBudget::unlimited()
budget.is_exhausted();                       // did a search run out?
budget.nodes();                              // nodes counted so far
budget.reset();                              // back to zero, same limit
```

- One node is one visit of a search function, nested VCF searches
  included: `DFSSolver::search` in VCF, `search_attacks` /
  `search_defences` in VCT.
- A budget stays exhausted until `reset()`. Calls sharing one budget stop
  as a whole, rather than each doing a little more.

Two guarantees make a budget safe:

- **A proof is never spurious.** A mate found just before the budget ran
  out is real, and is returned.
- **An aborted search leaves nothing false behind.** What it computed is
  not written to any memo (04, §5), so the solver can be reused at once.
  `test_abort_leaves_no_wrong_memo` aborts one solver at a range of small
  budgets and checks it still answers right afterwards.

## 4. Keeping a solver: the `Solver` trait

`solve` throws its solver away, with the tables the search filled. A caller
asking many related questions (a game engine, an analysis view walking a
variation) should keep one solver. All the pieces are public:

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::new(threat_limit, DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // bounds the whole loop
for board in candidates {
    let state = &mut VCTState::from_board(&board, attacker, limit);
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
| `DFSSolver` | `VCFState` | `new()` | VCF |
| `IDDFSSolver` | `VCFState` | `new(limits)` | VCF, at each of `limits` in turn |
| `VCTSolver<P>` — aliased `DFSVCTSolver`, `PNSVCTSolver`, `DFPNSVCTSolver` | `VCTState` | `new(threat_limit, defender_vcf_depth)` | VCT |

Build a fresh state per question with `VCFState::from_board(&board,
attacker, limit)` or `VCTState::from_board(..)`. The solver is what carries
over.

What reuse gives:

- **Any question, in any order.** Every memo is keyed by the position, the
  turn, the remaining limit *and* the attacker. One solver can answer about
  Black's and White's mate on the same board without mixing them up
  (`test_reused_solver_both_attackers`).
- **Bounded memory.** Each `solve` opens a new *generation*. Once a memo
  has grown past its carry capacity, entries older than the previous search
  are dropped, so the memos settle at about two searches' worth
  (`test_reused_solver_memo_stays_bounded`).
  - `with_carry_capacity(..)` on each solver sets the threshold (default
    `1 << 16` entries per memo).
  - `memo_len()` reports what is held.
- **The same question again is nearly free**: a few nodes instead of the
  whole search (`test_reused_solver`).
- **The same position at another limit is cheap.** A decision is a bound:
  a mate within 4 is a mate within 5 and 6; no mate within 3 is no mate
  within 2 and 1. Deepening one position over limits 1..6 costs about a
  fifth of six fresh searches (`test_decisions_carry_between_limits`).
- **Different positions share little**, since the remaining limit is in the
  key. The same board reached from two roots is two entries, unless the
  roots are the same distance from it.
- `clear()` forgets everything. Never required: use it to hand a solver on
  with a clean slate, or to give memory back.
- `advance_generation()` is what `solve` calls first. Only a caller driving
  a solver's lower-level `search` / `extract` by hand needs it (04, §4).

### Two more questions a caller can ask

- **Which moves defend against this mate?**
  `VCTState::threat_defences(&mate)` lists the moves worth trying against a
  found mate: its own path, the points that break its end, counter-fours,
  and the defender's own four-making moves. The VCT search generates
  defences from the same list (06, §3).
- **Is this move a threat?** Let the side to move pass
  (`Game::play(None)`) and ask for the opponent's VCF: `VCFState::new(game,
  limit)` on the passed game, then `DFSSolver::new().solve(..)`.
  `test_threat_after_pass` does it move by move.

## 5. `Mate` and `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

- `path` alternates attacker and defender moves, attacker first.
- `Mate::n_moves()` is its length; `n_attacks()` the attacker moves in it.
- `end` says why the defender is lost after the last move:

| `End` | The defender faces | Who can suffer it |
| --- | --- | --- |
| `Fours(p1, p2)` | Fours with two different winning points `p1`, `p2`: one block is not enough. A straight four (`Grid` reports two `Four`s with different eyes) or a double-four. | Either. An attacking Black gets it only from a straight four: a double-four is forbidden for Black and never played. |
| `Forbidden(p)` | A single four whose only block `p` is a forbidden move. | Black only. |
| `Unknown` | The win is proven but the line could not be completed: the attacker already had a four before the search (§6), or the extractor found no proven child to follow (06, §6). | — |

## 6. What is checked before searching: `trivial_result`

`solve` answers some positions without searching, since the solvers do not
handle them. (A reserved mode is checked first: it is `Aborted` whatever
the board.)

| Position | Result |
| --- | --- |
| Either colour already has a `Five` | `Disproven` |
| Black already has an `Overlined` | `Disproven` |
| The attacker already has a `Four` | `Proven(Mate { end: Unknown, path: [] })`: already won, nothing to prove |

A caller using a solver directly (§4) gets none of this, and should check
what it needs itself.

## 7. Cheat sheet

| I want to… | Use |
| --- | --- |
| Get a yes/no/line once | `solve` |
| Know what a search cost | `solve_with_stats`; `SolveStats::nodes` and `memo_len` |
| Find only VCFs | `SolveMode::VCFDFS` |
| Find VCTs | `SolveMode::VCTDFPNS`; `threat_limit` decides what counts as a threat |
| Stop a search that takes too long | `SolveLimits::with_max_nodes`; treat `Aborted` as unknown, not safe |
| Ask many questions cheaply | Keep a `DFPNSVCTSolver`, give it a fresh `VCTState` per question, share one `NodeBudget` |
| Ask about both colours | The same solver; the attacker is part of every key |
| Know which defences to try | `VCTState::threat_defences(&mate)` |
| Know whether the last move was a threat | Pass with `Game::play(None)`, then ask for the opponent's VCF |
| Read the winning line from JS | `wasm::solve` returns the path as `u8` point codes (02, §1); `decode_x` / `decode_y` |
| Add a solve mode | `SolveMode`, its `TryFrom<u8>` and `FromStr`, and the `match` in `solve` (`solve.rs`) |
