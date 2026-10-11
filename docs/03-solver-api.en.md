# Using the mate solvers

For code that *calls* the solvers: `wasm.rs`, the CLI in
`examples/solve.rs`, or Rust that keeps a solver across many questions.
How the search works is in [04](04-solver-framework.en.md),
[05](05-solver-vcf.en.md) and [06](06-solver-vct.en.md); none of it is
needed here.

- Terms from [01](01-renju-rules.en.md) are assumed: four, straight four,
  three, forbidden move.
- The Rust solver API is in `src/mate/`, re-exported from `quintet::mate`.
  The JS entry points are in `src/wasm.rs`.

<a id="quick-start"></a>

## Solve one position first

The Rust, CLI and JavaScript examples solve the same position for Black.
They allow five attacking moves and three moves in the nested VCF threat
check. The result is `I8,G8,K7,H10,J6,E10,I7,K5,I6`.

### Rust

```rust
use quintet::board::{Board, Player, Points};
use quintet::mate::{solve, SolveLimits, SolveMode, SolveResult};

fn main() -> Result<(), &'static str> {
    let board: Board = "H8,H7,I9,G7,J8,J9,G9,F10".parse()?;
    let limits = SolveLimits::new(5)
        .with_threat_limit(3)
        .with_max_nodes(100_000);
    match solve(SolveMode::VCTDFPNS, &board, Player::Black, limits) {
        SolveResult::Proven(mate) => println!("{}", Points(mate.path)),
        SolveResult::Disproven => println!("No mate within these limits"),
        SolveResult::Aborted => println!("Unknown: budget exhausted"),
    }
    Ok(())
}
```

### CLI

Run from the repository root. `o` means Black and `x` means White. The
position string lists alternating moves, starting with Black.

```sh
cargo run --release --example solve -- vct_dfpns 5 3 o H8,H7,I9,G7,J8,J9,G9,F10
```

### JavaScript

Install the published package with `npm install @renju-note/quintet` and
run in an environment that can load WebAssembly. Coordinates start at zero:
`H8` is `(7, 7)`. `16` selects df-pn; `true` selects Black as the attacker.

```js
import init, { solve, encode_xy, decode_x, decode_y } from "@renju-note/quintet";

await init();
const blacks = [[7, 7], [8, 8], [9, 7], [6, 8]].map(([x, y]) => encode_xy(x, y));
const whites = [[7, 6], [6, 6], [9, 8], [5, 9]].map(([x, y]) => encode_xy(x, y));
const path = solve(16, 5, blacks, whites, true, 3);
if (path !== undefined) {
  console.log(Array.from(path, p => [decode_x(p), decode_y(p)]));
}
```

In Rust, `Proven` means a mate exists, `Disproven` means none within the
specified bounds, and `Aborted` means unknown. JS returns a winning line or
`undefined`; it does not distinguish no mate, budget exhaustion, invalid
point codes, and similar failures. The JS example has no node budget; use
`solve_vct_auto` to specify one.

Read [§5](#section-5) to interpret the line and [§6](#section-6) for special
input positions. For ordinary use, continue with [§1](#section-1) and
[§2](#section-2). Read [§4](#section-4) only when keeping a solver for
repeated queries.

<a id="section-1"></a>

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

<a id="section-2"></a>

## 2. One-shot: `solve`

```rust
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

- Builds a solver, runs it once, drops it.
- `limits` carries the depths and an optional node budget.
- The answer has three values. `wasm.rs` calls it too, and reduces the
  answer to the winning line or nothing.

See the [Rust quick start](#quick-start) for a complete calling example.

`solve_with_stats` is the same call, plus what the search cost:

```rust
let (result, stats) = solve_with_stats(SolveMode::VCTDFPNS, &board, Player::Black, limits);
stats.nodes      // nodes visited, as NodeBudget counts them (§3)
stats.memo_len   // entries left in the solver's memos (Solver::memo_len, §4)
```

Both numbers are deterministic: the same arguments give the same numbers on
every run and machine. So they are what solver changes are compared by, as
the benchmark does (07).

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

### `SolveMode`

| `SolveMode` | Code | CLI name | What it searches |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | VCF, depth-first. `threat_limit` is ignored. |
| `VCFIDDFS` | 1 | `vcf_iddfs` | Reserved: `solve` searches nothing and returns `Aborted`. |
| `VCTDFS` | 10 | `vct` | VCT, depth-first over the proof-number tree. |
| `VCTIDDFS` | 11 | `vct_iddfs` | Reserved: `solve` searches nothing and returns `Aborted`. |
| `VCTPNS` | 15 | `vct_pns` | VCT, proof-number search. |
| `VCTDFPNS` | 16 | `vct_dfpns` | VCT, depth-first proof-number search (df-pn). **The one to use.** |
| `VCTAUTO` | 17 | `vct_auto` | VCT by df-pn with only the node budget: `limit` and `threat_limit` are ignored (below). |

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
| `threat_limit` | Attacker moves in the nested VCF that decides whether a move is a *threat* ([06 §2](06-solver-vct.en.md#section-2)). `0`: fours only. `1`: fours and threes. `2`+: also moves that set up a four-three, and deeper preparation (`test_vct_fukumi_move` needs `3`). | VCT |
| `defender_vcf_depth` | Attacker moves in the nested VCF that looks for the *defender's* counter-VCF. | VCT |
| `max_nodes` | Total work, in nodes ([§3](#section-3)). | every mode |

The `with_*` methods return a new value, so fields added later do not break
callers that build limits this way.

### Only a budget: `VCTAUTO`

`VCTAUTO` asks a VCT with no `limit` or `threat_limit` to choose: give it
`with_max_nodes` and nothing else matters (`defender_vcf_depth` still
applies). It is `DFPNSVCTSolver::solve_auto` ([06 §5](06-solver-vct.en.md#section-5)), and from JS
`solve_vct_auto(blacks, whites, black, max_nodes)`.

```rust
let limits = SolveLimits::new(0).with_max_nodes(1_000_000);   // the 0 is ignored
let result = solve(SolveMode::VCTAUTO, &board, Player::Black, limits);
```

- It searches with no limit on the length, under the `threat_limit`s 2, 3
  and 1 (`AUTO_THREAT_LIMITS`) in turn, a slice of the budget each, the
  slice doubling every round.
- `Proven`: a VCT, often longer than the shortest. A longer one is often
  cheaper to prove.
- `Disproven`: no VCT of any length under any of those `threat_limit`s.
  It never says "none within N moves".
- `Aborted`: unknown, as usual.

<a id="section-3"></a>

## 3. Budgets: `NodeBudget`

There is no clock. Everything under `src/` compiles for
`wasm32-unknown-unknown`, which has no `std::time`. A caller with a time
control converts it into a number of nodes.

```rust
let mut budget = NodeBudget::new(100_000);   // or NodeBudget::unlimited()
budget.is_exhausted();                       // did a search run out?
budget.nodes();                              // nodes counted so far
budget.reset();                              // back to zero, same limit
let part = budget.slice(1_000);              // a budget for part of the work, at most what is left
budget.absorb(&part);                        // count what that part spent
```

- One node is one visit of a search function, nested VCF searches
  included: `DFSSolver::search` in VCF, `search_attacks` /
  `search_defences` in VCT.
- A budget stays exhausted until `reset()`. Calls sharing one budget stop
  as a whole, rather than each doing a little more.

Two guarantees make a budget safe:

- **A proof is never spurious.** A mate found just before the budget ran
  out is real, and is returned.
- **Incomplete results are not reused.** Results left incomplete by budget
  exhaustion are not written to the memos. Completed subtrees remain
  ([04 §5](04-solver-framework.en.md#section-5)), so the solver can be reused at once.
  `test_abort_leaves_no_wrong_memo` aborts one solver at a range of small
  budgets and checks it still answers right afterwards.

<a id="section-4"></a>

## 4. Keeping a solver: the `Solver` trait

`solve` throws its solver away, with the tables the search filled. A caller
asking many related questions (a game engine, an analysis view walking a
variation) should keep one solver. All the pieces are public:

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // bounds the whole loop
for board in candidates {
    let state = &mut VCTState::from_board(&board, attacker, limit, threat_limit);
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
| `VCTSolver<P>` — aliased `DFSVCTSolver`, `PNSVCTSolver`, `DFPNSVCTSolver` | `VCTState` | `new(defender_vcf_depth)` | VCT |

Build a fresh state per question with `VCFState::from_board(&board,
attacker, limit)` or `VCTState::from_board(&board, attacker, limit,
threat_limit)`. The solver is what carries over.

<details>
<summary>Reuse guarantees and memo management (implementation details)</summary>

What reuse gives:

- **Any question, in any order.** Every memo is keyed by the position, the
  turn, the remaining limit *and* the attacker. One solver can answer about
  Black's and White's mate on the same board without mixing them up
  (`test_reused_solver_both_attackers`).
  - A VCT solver's memos are also keyed by the state's `threat_limit`, so
    one solver answers under several
    (`test_reused_solver_matches_fresh_across_threat_limits`).
  - Only the nested VCF searches share across `threat_limit`s; the proof
    tables do not ([06 §2](06-solver-vct.en.md#section-2)).
- **Bounded memory.** Each `solve` opens a new *generation*. Once a memo
  has grown past its carry capacity, entries older than the previous search
  are dropped, so the memos settle at about two searches' worth
  (`test_reused_solver_memo_stays_bounded`).
  - `with_carry_capacity(..)` on each solver sets the threshold (default
    `1 << 16` entries per memo).
  - `memo_len()` reports what is held.
- **The same question again is nearly free**: a few nodes instead of the
  whole search (`test_reused_solver`).
- **Decisions can carry to another limit.** This holds at every limit for
  VCF, and from `transfer_from` up for VCT ([06 §4](06-solver-vct.en.md#section-4)). Within that range, a
  decision is a bound:
  a mate within 4 is a mate within 5 and 6; no mate within 3 is no mate
  within 2 and 1. Deepening one position over limits 1..6 costs about a
  fifth of six fresh searches (`test_decisions_carry_between_limits`).
- **Different positions share little**, since the remaining limit is in the
  key. The same board reached from two roots is two entries, unless the
  roots are the same distance from it.
- `clear()` forgets everything. Never required: use it to hand a solver on
  with a clean slate, or to give memory back.
- `advance_generation()` is what `solve` calls first. Only a caller driving
  a solver's lower-level `search` / `extract` by hand needs it ([04 §4](04-solver-framework.en.md#section-4)).

</details>

### Other questions a caller can ask

- **Which moves defend against this mate?**
  `VCTState::threat_defences(&mate)` lists the moves worth trying against a
  found mate: its own path, the points that break its end, counter-fours,
  and the defender's own four-making moves. The VCT search generates
  defences from the same list ([06 §3](06-solver-vct.en.md#section-3)).
- **Which moves stop this VCT?** Right after a `VCTSolver` proved a
  position, `proof_tree(&mut state, &mut budget)` reads the whole proof off
  its tables as a `ProofTree`: one attack where the attacker moves, every
  defence worth trying where the defender does. On the position with one
  more stone of the defender's, `proves(&mut state, &tree, &mut budget)`
  checks that same proof node by node, for about 1/100 of a new search. A
  stone outside `tree.zone()` needs not even that: the proof stands. So a
  move outside the zone, or one `proves` accepts, does not stop the VCT;
  the others are the moves to search again ([06 §6](06-solver-vct.en.md#section-6)).
- **Is this move a threat?** Let the side to move pass
  (`Game::play(None)`) and ask for the opponent's VCF: `VCFState::new(game,
  limit)` on the passed game, then `DFSSolver::new().solve(..)`.
  `test_threat_after_pass` does it move by move.

<a id="section-5"></a>

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
| `Unknown` | The win is proven but the line could not be completed: the attacker already had a four before the search ([§6](#section-6)), or the extractor found no proven child to follow ([06 §6](06-solver-vct.en.md#section-6)). | — |

<a id="section-6"></a>

## 6. What is checked before searching: `trivial_result`

`solve` answers some positions without searching, since the solvers do not
handle them. (A reserved mode is checked first: it is `Aborted` whatever
the board.)

| Position | Result |
| --- | --- |
| Either colour already has a `Five` | `Disproven` |
| Black already has an `Overlined` | `Disproven` |
| The attacker already has a `Four` | `Proven(Mate { end: Unknown, path: [] })`: already won, nothing to prove |

A caller using a solver directly ([§4](#section-4)) gets none of this, and should check
what it needs itself.

<a id="section-7"></a>

## 7. Cheat sheet

| I want to… | Use |
| --- | --- |
| Get a yes/no/line once | `solve` |
| Know what a search cost | `solve_with_stats`; `SolveStats::nodes` and `memo_len` |
| Find only VCFs | `SolveMode::VCFDFS` |
| Find VCTs | `SolveMode::VCTDFPNS`; `threat_limit` decides what counts as a threat |
| Find VCTs with only a node budget | `SolveMode::VCTAUTO`, `VCTSolver::solve_auto`, `wasm::solve_vct_auto` |
| Stop a search that takes too long | `SolveLimits::with_max_nodes`; treat `Aborted` as unknown, not safe |
| Ask many questions cheaply | Keep a `DFPNSVCTSolver`, give it a fresh `VCTState` per question, share one `NodeBudget` |
| Ask about both colours | The same solver; the attacker is part of every key |
| Know which defences to try | `VCTState::threat_defences(&mate)` |
| Know which moves stop a VCT, move by move | `VCTSolver::proof_tree` after the solve, then `ProofTree::zone` and `VCTSolver::proves` per move |
| Know whether the last move was a threat | Pass with `Game::play(None)`, then ask for the opponent's VCF |
| Read the winning line from JS | `wasm::solve` returns the path as `u8` point codes ([02 §1](02-board-implementation.en.md#section-1)); `decode_x` / `decode_y` |
| Add a solve mode | `SolveMode`, its `TryFrom<u8>` and `FromStr`, and the `match` in `solve` (`solve.rs`) |
