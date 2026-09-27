# Inside `src/mate/`: the search framework

Read this before changing anything under `src/mate/`. It covers the pieces
every solver is built from — the game, the state, the memo, the `Solver`
trait — and the rules they all follow. The searches themselves are
[05](05-solver-vcf.en.md) (VCF) and [06](06-solver-vct.en.md) (VCT).

Assumed:

- [03](03-solver-api.en.md): what a solver is asked, `limit`, `NodeBudget`.
- The board terms of [02](02-board-implementation.en.md): `Four`, `Sword`,
  `eyes()`, forbidden moves, `zobrist_hash`.

## 1. Layout

```
src/mate.rs          module root: re-exports, the overview doc comment
src/mate/
├── solve.rs         solve, SolveMode, SolveLimits, SolveResult, trivial_result
│                    + the solver regression tests
├── solver.rs        trait Solver                      (§4)
├── game.rs          Game, Event, End                  (§2)
├── state.rs         trait State, Key                  (§3)
├── memo.rs          Memo<V>: generational memo        (§5)
├── budget.rs        NodeBudget                        (§6)
├── mate.rs          Mate: the result
├── vcf.rs, vcf/     the VCF solver                    (05)
└── vct.rs, vct/     the VCT solver                    (06)
src/feature/area.rs        Area: a set of points, for the VCF zone (05, §2)
src/feature/shape.rs       ShapeMap: what a move would make, for VCT move ordering (06, §8)
src/feature/stale.rs       StaleLines: the lines ShapeMap and SwordMap have yet to recompute
src/feature/sword.rs       SwordMap: cached swords for VCF (02 §7, 05)
```

How they fit together, bottom up:

```
Board  ──►  Game  ──►  State (VCFState | VCTState)  ──►  Solver::solve(state, budget) ──► Option<Mate>
            stones,    + attacker, remaining limit,       reads and writes Memo<V>s
            moves,     + Key for the memos                 keyed by State::key()
            turn
```

- **`Game`**: a board plus the moves played on it during the search.
- **`State`**: a game plus what the search needs on top: which side the
  search is for, how many attacker moves are left, and (VCT) a shape map
  for move ordering.
- **`Solver`**: searches a state. It owns **`Memo`**s that outlive one
  question, all keyed by the state's **`Key`**.
- **`NodeBudget`**: bounds the work of one question, or of a series.

## 2. `Game`: the board during a search

```rust
pub struct Game { board: Board, moves: Vec<Option<Point>>, pub turn: Player }
```

`Game::new(&board, turn)` clones the board once. After that the search
never clones:

- `play(m)` puts a stone and flips `turn`.
- `undo()` takes it back.
- `with_move(m, f)` does play → `f(self)` → undo, and returns `f`'s result.
  Every solver walks its tree with it.

A move is an `Option<Point>`. `play(None)` is a **pass**: the turn flips,
the board stays. Renju has no passes, but a threat is defined by one ("what
could the other side do if I did nothing?"), so it is a first-class move
here.

### `check_event`: fours already on the board

Before generating anything, every node asks `Game::check_event()`: what do
the fours of the side that *just moved* mean for the side *to move*?

| The opponent's fours have… | `Event` | For the side to move |
| --- | --- | --- |
| two different winning points `p1`, `p2` | `Defeated(Fours(p1, p2))` | lost: one block cannot cover both |
| one winning point `p`, forbidden for the side to move | `Defeated(Forbidden(p))` | lost: the only block is illegal |
| one winning point `p` | `Forced(p)` | must play `p` |
| none | `None` | free to choose |

- Only fours through the last move are examined: an older four would
  already have forced a reply.
  - These are the eyes of `rows_on(last_move, opponent, Four)`, read off
    each line's bitmasks by `four_eyes_on` without building the rows (this
    runs at every node).
- After a pass there is no last move, so every four of the opponent is
  scanned.
- A straight four shows up as two `Four`s with different eyes, handled by
  the same two-point test as a double-four (`first_two_distinct`).

`End` is the `Defeated` payload, and what a `Mate` ends with (03, §5).

## 3. `State`: the limit and the key

```rust
pub trait State {
    fn game(&self) -> &Game;        fn game_mut(&mut self) -> &mut Game;
    fn attacker(&self) -> Player;
    fn limit(&self) -> u8;          fn set_limit(&mut self, limit: u8);
    // provided:
    fn play(&mut self, m: Option<Point>);   fn undo(&mut self);   fn with_move(..);
    fn attacking(&self) -> bool;    // turn == attacker
    fn key(&self) -> Key;           fn zobrist_hash(&self) -> u64;
    fn after_play(..) / after_undo(..)      // hooks: the states keep their maps in step
}
```

`VCFState` and `VCTState` implement it. `State::play` is `Game::play` plus
the limit bookkeeping, the one rule about `limit` to remember:

> `limit` is the number of attacker moves still allowed. It is decremented
> when the turn *returns* to the attacker, i.e. after a defender move. At a
> defender node it still counts the attack just played.

Down a line: root (attacker to move) `limit = 3` → after the first attack,
still `3` → after the defence, `2` → after the second attack, `2` → … →
`0`: the attacker may not move again. The walk-through of `test_vcf_counter`
in 05, §4 shows this on a board.

### `Key`: what every memo is keyed by

```rust
pub struct Key { pub position: u64, pub limit: u8 }
```

`State::key()` is what every memo of every solver uses.

- `position` folds three things into one Zobrist hash:
  - the stones (`Board::zobrist_hash`),
  - the turn (`apply_turn`; a pass changes the turn without touching the
    stones, so it must be in),
  - **the attacker** (`apply_attacker`).
- `limit` is the remaining limit.

The attacker is in the key so one solver can be asked about both sides. The
same stones and side to move can be "Black mates" for one question and
"White has no mate" for another; they must not share an entry
(`test_zobrist_hash_separates_the_attacker`).

The two parts are kept apart because they are remembered differently:

- A **decision** (proven or disproven) is a *bound*, not a fact about one
  limit. A mate within `limit` is a mate within any larger limit; no mate
  within `limit` is no mate within any smaller one. So decisions are
  stored under `position` alone, with the limit as data:
  - `DFSSolver::dead_ends` keeps the largest limit with no VCF.
  - `ProofTable::decided` keeps the smallest proven and the largest
    disproven limit.
- **Anything short of a decision** (a proof number, a candidate list) is
  about the tree at one limit, so it is stored under `Key::hash()`, the two
  combined (`ProofTable::estimates`, the VCT candidate caches).

- For VCF the bound is exact: nothing it generates reads `limit`.
- For VCT it holds from `transfer_from` up (06, §4).
- `test_verdict_is_monotone_in_limit` (`--ignored`) checks that the verdict
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

Every solver has two entry points, split the same way in all three:

| | `solve` | `search` |
| --- | --- | --- |
| Is | one whole question | the search itself |
| Does | `advance_generation()`, then `search` (VCT: then `extract`) | the recursion; no memo housekeeping |
| Called by | the outside: `mate::solve`, an engine | other solvers: `IDDFSSolver` over `DFSSolver::search`, the VCT solver over its nested VCF solvers' `search` |

Why the split: a solver can sit inside another.

- The VCT solver asks its nested VCF solvers hundreds of times per search.
- If each call opened a generation, the nested memo would churn
  constantly.
- Only the outermost question decides where one generation ends.

`clear()` forgets everything; `memo_len()` counts what is held. Both are
for callers; nothing inside the solvers needs them.

## 5. `Memo`: remembering across searches

```rust
pub struct Memo<V> { entries: HashMap<u64, Entry<V>>, generation: u32, carry_capacity: usize }
```

Every table a solver keeps is a `Memo`, or a pair of them (`ProofTable`).
Three rules govern what goes in and when it leaves.

**What is stored stays true.** Every entry is a fact about its key: a dead
end, a proof, a disproof, a proof-number estimate. The key contains
everything the fact depends on (§3), so a memo never needs invalidating,
only bounding.

**Generations bound it.**

- `advance_generation()`, called once per `solve`, opens a new generation.
- If the memo has grown past `carry_capacity`, it first drops every entry
  older than the *previous* generation.
- The search that just finished is always kept: asking the same question
  again is what reuse really buys.
- So the memo settles at about two searches' worth, however many questions
  are asked. `DEFAULT_CARRY_CAPACITY` is `1 << 16` entries per memo.

**Nothing is dropped during a search.** `carry_capacity` is not a hard cap.

- A df-pn node only makes progress once its child's numbers are in the
  table.
- A memo that evicted mid-search could send the expansion loop round again
  on the same child forever.
- Within one search, the node budget is the bound.

And one rule about what is *not* stored, from `NodeBudget`:

**An aborted search writes nothing.**

- When the budget runs out, `None` / "unknown" propagates up, and every
  insert on the way is skipped: no dead end in `DFSSolver`, no node in a
  `ProofTable`, no list in a candidate cache.
- An entry from a search that gave up would be a guess dressed as a fact.
- The checks are the `!budget.is_exhausted()` guards next to every insert.

## 6. `NodeBudget`: counting work

- `NodeBudget::consume()` counts one node and returns `false` once the
  limit is passed. Exhaustion is sticky until `reset()`.
- It is called at the top of each search function (`DFSSolver::search`,
  `search_attacks`, `search_defences`) and nowhere else. So a node is one
  visit of one of those, nested VCF searches included.
- There is no clock anywhere under `src/`: the crate must compile for
  `wasm32-unknown-unknown`.

`VCTSolver::solve` runs `extract` with an unlimited budget. Once the root is
proven, reading the line back is bounded by its length, and giving up on a
proof for want of budget would be pointless.

## 7. Cheat sheet

| Question | Where to look |
| --- | --- |
| What is a solver, minimally? | `Solver` in `solver.rs`: `solve` = `advance_generation` + the solver's own `search` |
| Why did the search stop at depth N? | `limit` counts attacker moves; `State::play` decrements it after each defender move |
| Why is the attacker in the hash? | `State::key`: one solver answers about both sides |
| Why are decisions keyed without the limit? | A decision is a bound over limits (§3); `DFSSolver::dead_ends`, `ProofTable::decided` |
| Why did the memo not grow? | `Memo::advance_generation` drops all but the previous generation once over `carry_capacity` |
| Why is something not memoized? | Aborted searches write nothing: the `!budget.is_exhausted()` guards |
| What is one node? | One call of `DFSSolver::search` / `search_attacks` / `search_defences` |
| How are fours on the board detected? | `Game::check_event` → `Defeated` / `Forced` |
| How is a pass represented? | `Game::play(None)`: the turn flips, the board does not |
| Adding a regression case | ASCII board + expected path string in `solve.rs` tests, one assertion per relevant `SolveMode` |
