# Inside `src/mate/`: the search framework

Read this before changing anything under `src/mate/`. It explains the
pieces every solver is built from — the game, the state, the memo, the
`Solver` trait — and the rules they all follow. The two searches themselves
are [05](05-solver-vcf.en.md) (VCF) and [06](06-solver-vct.en.md) (VCT).

It assumes [03](03-solver-api.en.md) (what a solver is asked, `limit`,
`NodeBudget`) and the board vocabulary of [02](02-board-implementation.en.md):
`Four`, `Sword`, `eyes()`, forbidden moves, `zobrist_hash`.

## 1. Layout

```
src/mate.rs          module root: re-exports, the overview doc comment
src/mate/
├── solve.rs         solve, SolveMode, SolveLimits, SolveResult, validate
│                    + the solver regression tests
├── solver.rs        trait Solver                      (§4)
├── game.rs          Game, Event, End                  (§2)
├── state.rs         trait State, Key                  (§3)
├── memo.rs          Memo<V>: generational memo        (§5)
├── budget.rs        NodeBudget                        (§6)
├── mate.rs          Mate: the result
├── vcf.rs, vcf/     the VCF solver                    (05)
└── vct.rs, vct/     the VCT solver                    (06)
src/feature/potential.rs   PotentialField: move ordering for VCT (06, §8)
src/feature/shape.rs       ShapeMap: what a move would make, for VCT move ordering (06, §8)
src/feature/sword.rs       SwordMap: cached swords for VCF (02 §8, 05)
```

How they fit together, from the bottom up:

```
Board  ──►  Game  ──►  State (VCFState | VCTState)  ──►  Solver::solve(state, budget) ──► Option<Mate>
            stones,    + attacker, remaining limit,       reads and writes Memo<V>s
            moves,     + Key for the memos                 keyed by State::key()
            turn
```

- A **`Game`** is a board plus the moves played on it during the search.
- A **`State`** is a game plus what the search needs on top: whose side the
  search is for, how many attacker moves are left, and — for VCT — a
  potential field and a shape map for move ordering.
- A **`Solver`** searches a state. It owns **`Memo`**s that outlive one
  question, all keyed by the state's **`Key`**.
- A **`NodeBudget`** bounds how much work one question, or a series of them,
  may do.

## 2. `Game`: the board during a search

```rust
pub struct Game { board: Board, moves: Vec<Option<Point>>, pub turn: Player }
```

`Game::init(&board, turn)` clones the board once. From then on the search
never clones again: `play(m)` puts a stone and flips `turn`, `undo()` takes
it back, and `into_play(m, f)` does play → `f(self)` → undo, returning
`f`'s result. Every solver walks its tree with `into_play`.

A move is an `Option<Point>`. `play(None)` is a **pass**: the turn flips and
the board stays. A pass is how a threat is defined — "what could the other
side do if I did nothing?" — so it is a first-class move here even though
Renju has no passes.

### `check_event`: fours already on the board

Before generating anything, every node asks `Game::check_event()`: what do
the fours of the side that *just moved* mean for the side *to move*?

| The opponent's fours have… | `Event` | For the side to move |
| --- | --- | --- |
| two different winning points `p1`, `p2` | `Defeated(Fours(p1, p2))` | lost — one block cannot cover both |
| one winning point `p`, forbidden for the side to move | `Defeated(Forbidden(p))` | lost — the only block is illegal |
| one winning point `p` | `Forced(p)` | must play `p` |
| none | `None` | free to choose |

Only fours through the last move are examined (`rows_on(last_move,
opponent, Four)`): an older four would already have forced a reply. After a
pass there is no last move, so every four of the opponent is scanned
instead. A straight four shows up as two `Four`s with different eyes and is
handled by the same two-point test (`take_distinct_two`) as a double-four.

`End` is the `Defeated` payload and is what a `Mate` ends with (03, §5).

## 3. `State`: the limit and the key

```rust
pub trait State {
    fn game(&self) -> &Game;        fn game_mut(&mut self) -> &mut Game;
    fn attacker(&self) -> Player;
    fn limit(&self) -> u8;          fn set_limit(&mut self, limit: u8);
    // provided:
    fn play(&mut self, m: Option<Point>);   fn undo(&mut self);   fn into_play(..);
    fn attacking(&self) -> bool;    // turn == attacker
    fn key(&self) -> Key;           fn zobrist_hash(&self) -> u64;
    fn after_play(..) / after_undo(..)      // hooks, used by VCTState for its field
}
```

`VCFState` and `VCTState` implement it. `State::play` is `Game::play` plus
the limit bookkeeping, which is the one rule about `limit` to remember:

> `limit` is the number of attacker moves still allowed. It is decremented
> when the turn *returns* to the attacker — i.e. after a defender move — so
> at a defender node it still counts the attack that was just played.

Following it down a line: root (attacker to move) `limit = 3` → after the
first attack, still `3` → after the defence, `2` → after the second attack,
`2` → … → `0` means the attacker may not move again. `test_vcf_counter`'s
walk-through in 05, §4 shows this on a board.

### `Key`: what every memo is keyed by

```rust
pub struct Key { pub position: u64, pub limit: u8 }
```

`State::key()` is what all memos in all solvers use. `position` is three
things folded into one Zobrist hash: the stones (`Board::zobrist_hash`),
whose turn it is (`apply_turn`; a pass changes the turn without touching
the stones, so it must be in) and **the attacker** (`apply_attacker`).
`limit` is the remaining limit.

The attacker is in the key so that one solver can be asked about both sides:
the same stones with the same side to move can be "Black mates" for one
question and "White has no mate" for another, and those must not share an
entry (`test_zobrist_hash_separates_the_attacker`).

The two parts are kept apart because they are remembered differently:

- A **decision** — proven or disproven — is a *bound*, not a fact about one
  limit. A mate within `limit` is a mate within any larger limit; no mate
  within `limit` is no mate within any smaller one. So decisions are stored
  under `position` alone, with the limit as data: `DFSSolver::deadends`
  keeps the largest limit with no VCF, `ProofTable::decided` keeps the
  smallest proven and largest disproven limit.
- **Anything short of a decision** — a proof number, a candidate list — is
  about the tree at one limit, so it is stored under `Key::hash()`, the two
  combined (`ProofTable::estimates`, the VCT candidate caches).

For the VCF solver the bound is exact, because nothing it generates reads
`limit`. For VCT it holds from `transfer_from` upwards (06, §4).
`test_verdict_is_monotone_in_limit` (`--ignored`) checks that the verdict
never flips back from "mate" to "no mate" as the limit grows.

## 4. `Solver`: one question, one generation

```rust
pub trait Solver {
    type State: State;
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;
    fn clear(&mut self);
    fn advance_generation(&mut self);
    fn memo_len(&self) -> usize;
}
```

Every solver has two entry points, and the split is the same in all three:

| | `solve` | `search` |
| --- | --- | --- |
| Is | one whole question | the search itself |
| Does | `advance_generation()`, then `search` (VCT: then `extract`) | the recursion; no memo housekeeping |
| Called by | the outside world: `mate::solve`, an engine | other solvers: `IDDFSSolver` over `DFSSolver::search`, the VCT solver over its nested VCF solvers' `search` |

The point of the split: a solver can sit inside another one. The VCT solver
asks its nested VCF solvers hundreds of times per search; if each of those
calls opened a generation, the nested memo would be churned constantly.
Only the outermost question decides where one generation ends and the next
begins.

`clear()` forgets everything; `memo_len()` counts what is held. Both exist
for callers, nothing inside the solvers needs them.

## 5. `Memo`: remembering across searches

```rust
pub struct Memo<V> { entries: HashMap<u64, Entry<V>>, generation: u32, carry_capacity: usize }
```

Every table a solver keeps is a `Memo`, or a pair of them (`ProofTable`).
Three rules govern what goes in and when it leaves.

**What is stored stays true.** Every entry is a fact about its key — a
deadend, a proof, a disproof, a proof-number estimate — and the key
contains everything the fact depends on (§3). So a memo never needs to be
invalidated, only bounded.

**Generations bound it.** `advance_generation()`, called once per `solve`,
opens a new generation; when the memo has grown past `carry_capacity`, it
first drops every entry older than the *previous* generation. The search
that just finished is always kept — asking the same question again is what
reuse really buys — so the memo settles at about two searches' worth
however many questions are asked. `DEFAULT_CARRY_CAPACITY` is `1 << 16`
entries per memo.

**Nothing is dropped during a search.** `carry_capacity` is not a hard cap.
A df-pn node only makes progress once its child's numbers are in the table;
a memo that evicted mid-search could send the expansion loop round again on
the same child forever. Within one search, the node budget is the bound.

And one rule about what is *not* stored, from `NodeBudget`:

**An aborted search writes nothing.** When the budget runs out, `None` /
"unknown" propagates up, and every insert on the way is skipped: no deadend
in `DFSSolver`, no node in a `ProofTable`, no list in a candidate cache.
A memo entry made from a search that gave up would be a guess dressed as a
fact. The checks are the `!budget.is_exhausted()` guards next to every
insert.

## 6. `NodeBudget`: counting work

`NodeBudget::consume()` counts one node and returns `false` once the limit
is passed; exhaustion is sticky until `restart()`. It is called at the top
of each search function — `DFSSolver::search`, `search_attacks`,
`search_defences` — and nowhere else, so a node is one visit of one of
those, nested VCF searches included. There is no clock anywhere under
`src/`: the crate must compile for `wasm32-unknown-unknown`.

`VCTSolver::solve` runs `extract` with an unlimited budget: once the root is
proven, reading the line back is bounded by the line's length, and giving up
on a proof for want of budget would be pointless.

## 7. Cheat sheet

| Question | Where to look |
| --- | --- |
| What is a solver, minimally? | `Solver` in `solver.rs`: `solve` = `advance_generation` + the solver's own `search` |
| Why did the search stop at depth N? | `limit` counts attacker moves and is decremented in `State::play` after each defender move |
| Why is the attacker in the hash? | `State::key` — one solver answers about both sides |
| Why are decisions keyed without the limit? | A decision is a bound over limits (§3); `DFSSolver::deadends`, `ProofTable::decided` |
| Why did the memo not grow? | `Memo::advance_generation` drops all but the previous generation once over `carry_capacity` |
| Why is something not memoized? | Aborted searches write nothing — the `!budget.is_exhausted()` guards |
| What is one node? | One call of `DFSSolver::search` / `search_attacks` / `search_defences` |
| How are fours on the board detected? | `Game::check_event` → `Defeated` / `Forced` |
| How is a pass represented? | `Game::play(None)` — the turn flips, the board does not |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode` |
