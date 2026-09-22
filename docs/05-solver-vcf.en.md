# VCF: the four-chasing search (`src/mate/vcf/`)

A **VCF** (victory by continuous fours) is a line in which every attacking
move makes a four. The defender never has a choice — each four must be
blocked at once — so the whole line is forced from the first move, and the
tree is a chain of `(attack, block)` pairs. This makes VCF the simplest
solver in the crate and the one the VCT solver leans on to recognise
threats.

Assumes [04](04-solver-framework.en.md): `Game`, `State`, `Key`, `Memo`,
`Solver`.

```
src/mate/vcf.rs         module doc, re-exports
src/mate/vcf/
├── state.rs            VCFState: Game + attacker + limit; four-making move pairs   (§1)
├── dfs.rs              DFSSolver: the depth-first search and its deadend memo      (§2)
└── iddfs.rs            IDDFSSolver: DFSSolver at increasing limits                (§3)
```

## 1. `VCFState`: where the fours are

```rust
pub struct VCFState { game: Game, pub attacker: Player, pub limit: u8 }
```

`VCFState::init(&board, attacker, limit)` starts with the attacker to move;
`VCFState::new(game, limit)` takes an existing game (the VCT solver builds
its nested VCF states this way, and the attacker is then whoever is to move
in `game`).

A four is made by playing into a **`Sword`**: three own stones in a
five-window with two empty *eyes*. Playing one eye makes a `Four` whose
remaining winning point is the other eye, so each sword yields two
`(attack, defence)` pairs (`sword_eyes_pairs`):

```
column H:  H7 = x (White),  H8 H9 H10 = o (Black),  H11 H12 = empty
            → Sword with eyes H11, H12
pairs:  (H11, H12)   play H11: four H8–H11, White must answer H12
        (H12, H11)   play H12: four H8,H9,H10,_,H12, White must answer H11
```

Three generators, in the order the solver tries them:

| Generator | Pairs from | When |
| --- | --- | --- |
| `forced_move_pair(p)` | the sword whose attack eye is exactly `p` | the attacker is `Forced(p)`: the defender's block made a four of its own (a counter-four), and the attacker's reply must block it *and* be a four, or the VCF is over |
| `neighbor_move_pairs()` | swords through `last2_move`, the attacker's previous stone | first — extending the stones just played is the likeliest way to keep making fours |
| `move_pairs()` | every sword of the side to move | then everything else |

Note that the block is taken from the sword, not re-derived from the board
after the attack. If the attack happens to make *two* fours, the defender's
`check_event` reports `Defeated(Fours(..))` before the stored block is ever
played, so the stored eye only matters for a single four.

## 2. `DFSSolver`

```
solve(state):                                  # Solver::solve — one question
    advance_generation(); return search(state)

search(state):                                 # attacker to move
    if limit == 0:                       return None
    if !budget.consume():                return None
    if deadends[key.position] >= limit:  return None        # known: no VCF this deep
    result = search_move_pairs(state)
    if result is None and budget not exhausted:
        deadends[key.position] = max(known, limit)
    return result

search_move_pairs(state):
    match check_event():
        Defeated(_) → None                                   # the defender has a double four
        Forced(p)   → forced_move_pair(p) → search_attack, or None
        None        → for (a, d) in neighbor pairs, then the rest:
                          if search_attack(a, d) is Some → return it
                      None

search_attack(state, attack, defence):
    if attack is forbidden: return None
    into_play(attack): search_defence(defence)   then prepend attack

search_defence(state, defence):                # defender to move
    if check_event() is Defeated(end): return Mate { end, path: [] }
    into_play(defence): search(state)            then prepend defence
                                                 # limit decrements inside into_play
```

Things to notice:

- **Only failures are memoized.** A success is returned immediately; a
  position shown to have no VCF is recorded in `deadends: Memo<u8>` as the
  largest limit it was shown for. Because nothing here reads `limit` when
  generating moves, the tree at limit *n* is the tree at limit *n+1* cut
  short, so "no VCF within *n*" is exact for every limit up to *n*, and the
  memo is keyed by `Key::position` alone (04, §3).
- **Forbidden attacks are skipped**, so Black never plays a double-four or
  overline here. That is also why Black's `Fours` end is always a straight
  four (03, §5).
- **Counter-fours are ordinary.** When the block makes a four, the attacker's
  next node sees `Forced(p)`; `forced_move_pair(p)` succeeds only if `p` is
  itself a four-making eye. `test_vcf_counter` and
  `test_vcf_not_opponent_double_four` cover this.
- **`Defeated` at an attacker node ends the branch.** The defender's block
  made a double four (or a four Black cannot block); the attacker has no
  four that answers, so the line fails.

## 3. `IDDFSSolver`

`IDDFSSolver::init(limits)` runs `DFSSolver::search` at each `limit` in
`limits` that is below the state's own, then at the state's own limit, and
returns the first result. Shallow passes find short VCFs without walking the
whole tree, and since the deadend memo is exact per limit they cost the
deeper passes nothing.

It implements `Solver` with the same `solve` / `search` split. The VCT
solver keeps one per side with `limits = [1]` — "check for a one-move win
first" — and calls its `search` (06, §2).

## 4. Walk-through

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

`solve(VCFDFS, 3, &board, Black, 0)` returns `I8,G8,I10,I9,J9` with end
`Fours(H11, M6)`:

| Move | What it makes | `limit` after it |
| --- | --- | --- |
| root | | 3 |
| `I8` (Black) | four `H8,I8,J8,K8` from the sword `H8,J8,K8` (eyes `G8`, `I8`) | 3 — the attack just played still counts |
| `G8` (White) | forced block | 2 |
| `I10` (Black) | four `I6,I7,I8,_,I10` | 2 |
| `I9` (White) | forced block | 1 |
| `J9` (Black) | `I10,J9,K8,L7` with both `H11` and `M6` open — a straight four, two `Four`s | 1 |
| | White's `check_event` → `Defeated(Fours(H11, M6))` | |

With `limit = 2` the same call returns `None`: after `I9` the limit is 0
and `search` stops before `J9` is tried. Three fours need `limit >= 3`.

## 5. Cheat sheet

| Question | Where to look |
| --- | --- |
| A four-making move is not generated | `VCFState::move_pairs` sees only `Sword` eyes; for Black, the `exact` margins exclude fours that would be overlines |
| A counter-four is mishandled | `Game::check_event` at the attacker's node → `Forced(p)`, then `VCFState::forced_move_pair` |
| Which fours are tried first? | `neighbor_move_pairs` (through the attacker's last stone), then `move_pairs` |
| The memo | `DFSSolver::deadends`: largest limit with no VCF, keyed by `Key::position`; never written after the budget ran out |
| Why `solve` and `search`? | `solve` opens a generation; `search` is what `IDDFSSolver` and the VCT solver call repeatedly (04, §4) |
| How deep may a VCF be? | `limit` attacker moves; each is a four, so a VCF of *k* fours needs `limit >= k` |
