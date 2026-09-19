# How `src/board/` implements the Renju rules

This document explains the board representation and how the rule concepts
from [01-renju-rules.en.md](01-renju-rules.en.md) — row, five, overline, four, straight
four, three, double-four, double-three, forbidden move — are detected. It is
written against the current code; identifiers in backticks can be grepped.

Module map (`src/board/mod.rs`):

| File | Role |
| --- | --- |
| `player.rs` | `Player` (`Black` / `White`) and its text form (`o` / `x`). |
| `point.rs` | `Point` (x, y), `Points`, `Direction`, `Index` (position along a line), the `u8` wasm encoding. |
| `line.rs` | `Line`: one row/column/diagonal as two bitmasks. |
| `sequence.rs` | `Sequences`: sliding-window scanner over a `Line` that finds stone patterns. |
| `structure.rs` | `StructureKind` (Two, Three, Sword, Four, Five, ...) and `Structure` (a pattern located on the board). |
| `square.rs` | `Square`: the whole 15×15 board as four arrays of `Line`s, plus pattern queries. |
| `forbidden.rs` | Renju forbidden-move detection for Black. |
| `potential.rs` | Per-point "potential" scoring used for move ordering. |
| `zobrist.rs` | Zobrist hashing for transposition tables. |
| `board.rs` | `Board` = `Square` + Zobrist hash; the public facade used by the solvers. |

---

## 1. Points and coordinates

`Point(x, y)` with `0 <= x, y < RANGE (15)`. `x` is the column (`A`=0 …
`O`=14) and `y` is the row (`1`=0 … `15`=14). `Display`/`FromStr` use the
`H8` notation; `Points` is a comma-separated list (`H8,H7,F6`).

The wasm/JS boundary encodes a point as a single `u8`:
`code = x * 15 + y` (`From<Point> for u8`, `TryFrom<u8> for Point`). This is
public API and must not change.

### Lines and `Index`

Every point lies on exactly four lines, one per `Direction`:

| `Direction` | Line identifier `i` | Position on the line `j` |
| --- | --- | --- |
| `Vertical` | `x` | `y` |
| `Horizontal` | `y` | `x` |
| `Ascending` (`/`) | `x + 14 - y` (0..=28) | `x` if `i < 14`, else `y` |
| `Descending` (`\`) | `x + y` (0..=28) | `x` if `i < 14`, else `14 - y` |

`Point::to_index(d) -> Index { d, i, j }` and `Index::to_point()` convert
back and forth. `Index::walk(step)` / `walk_checked(step)` move along the
line; `Index::maxj()` gives the last valid position (diagonals get shorter
towards the corners).

## 2. `Line`: one line as bitmasks

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

Bit `j` of `blacks` / `whites` is set when position `j` holds a stone of
that colour. `size` is the line length (15 for rows/columns, 5..=15 for the
diagonals that are kept — see §4). `put_mut` / `remove_mut` / `stone(j)` are
plain bit operations, which is what makes the search loop cheap.

`potential_cap(r)` is a quick upper bound used to skip lines: if the line
cannot even fit five stones next to the opponent's stones it returns 0,
otherwise `own stones + 1`.

## 3. `Sequences`: finding patterns in a line

`Sequences` (`sequence.rs`) is an iterator that slides a **5-cell window**
over a line and reports windows matching a pattern. It is the single
primitive from which every rule concept is built.

For each window start `i` it looks at 7 bits of each colour's mask:

```
bit:   6      5 4 3 2 1     0
cell:  i+5    i+4 … i       i-1
       ^ right margin  ^ target (5 cells)  ^ left margin
```

(`my`/`op` are shifted left by one so that `i-1` exists for `i = 0`;
outside the board the margins read as empty.)

A window is **valid** when:

- no opponent stone is in the 5 target cells (`op & TARGET_MASK == 0`), and
- if `strict`, no *own* stone is in either margin (`my & MARGIN_MASK == 0`).

`strict` is set exactly when the player is **Black** (see
`StructureKind::to_sequence`). It implements the rule that for Black a
completed five must be *exactly* five: an own stone adjacent to the window
would turn the five into an overline, so such a window does not count as a
five-to-be. White has no such restriction, so its windows are scanned
non-strictly and an overline is treated like any other win.

Within a valid window the number of own stones is compared with `n`. Three
`SequenceKind`s decide what is reported:

| `SequenceKind` | Condition | Meaning |
| --- | --- | --- |
| `Single` | window `i` has exactly `n` own stones | A place where adding `5 - n` stones makes a five. |
| `Double` | windows `i-1` **and** `i` both have `n` own stones | Two overlapping fives-to-be, i.e. a 6-cell span with `n + 1` stones in it. |
| `Compact` | the `n` own stones all lie in the 4 cells `i..=i+3`, and both `i-1` and `i+4` are empty (window `i-1` and window `i` are both valid with `n` stones) | A pattern with **open ends** on both sides. |

The reported item is `(i, Sequence)` where `Sequence` is a 5-bit mask of
the target cells. `Sequence::stones()` / `eyes()` map that mask to offsets
`0..5`: stones are set bits, eyes are the empty cells. For `Compact` the
5th bit is forced on (`LAST_MASK`) so that the closing empty cell `i+4` is
*not* reported as an eye; as a side effect it does show up in `stones()`,
so treat `stones()` of a `Compact` structure as "cells that are not eyes".

`Sequences::new_on(j, …)` restricts the scan to windows containing position
`j`; it is what `structures_on(p, …)` uses to ask "which patterns would this
move touch?".

## 4. `Square`: the full board

```rust
pub struct Square {
    vlines: [Line; 15],  // columns,   indexed by x
    hlines: [Line; 15],  // rows,      indexed by y
    alines: [Line; 21],  // ascending  diagonals with length >= 5
    dlines: [Line; 21],  // descending diagonals with length >= 5
}
```

A stone is stored redundantly in all four of its lines (`put_mut` updates
each). Diagonals shorter than five cells can never contain a five, so the
four shortest at each corner are omitted: diagonal `i` is stored at
`alines[i - 4]` for `i` in `4..=24` (`D_LINE_OMIT = 4`, `D_LINE_NUM = 21`);
`line_idx` returns `None` for the others and pattern queries simply skip
them.

Queries:

- `stone(p)`, `stones(player)`, `empties()`, `neighbors(p, distance, only_empty)`.
- `structures(r, kind)` — every `Structure` of `kind` for player `r` on the
  whole board.
- `structures_on(p, r, kind)` — only structures whose 5-window contains
  point `p` (the four lines through `p`). This is the hot path for
  "what does playing `p` create?".
- `potentials(...)` / `potentials_along(...)` — see §7.

Parsing (`FromStr for Square`, reused by `Board`) accepts three formats:
a move list `H8,H7,F6` (alternating from Black), a stone list
`H8,F6/H7` (`blacks/whites`), or the 15-line ASCII picture used throughout
the tests (`o` Black, `x` White, `.` empty, row 15 first).

## 5. `StructureKind`: the rule vocabulary

`StructureKind::to_sequence(r)` maps each kind to `(SequenceKind, n, strict)`
with `strict = r.is_black()`:

| `StructureKind` | Sequence | Pattern (Black shown, `_` = eye) | Rule concept |
| --- | --- | --- | --- |
| `Five` | `Single`, 5 | `ooooo` | **Five** (§3). For Black, strict margins exclude overlines. |
| `OverFive` | `Double`, 5, never strict | `oooooo` (6+) | **Overline**. |
| `Four` | `Single`, 4 | `oooo_`, `ooo_o`, `oo_oo`, … | **Four**: one more stone at the eye makes a five. A straight four appears as **two** adjacent `Four`s. |
| `OpenFour` | `Compact`, 4 | `.oooo.` | **Straight four**. |
| `Sword` | `Single`, 3 | `ooo__`, `o_oo_`, … (3 stones in a 5-window) | A "four-to-be": playing either eye makes a `Four`. Not a rule term; used by VCF/VCT to enumerate four-making moves. |
| `Three` | `Compact`, 3 | `.ooo_.`, `.oo_o.`, `.o_oo.`, `._ooo.` | **Three**: playing the single eye makes an `OpenFour`. |
| `Two` | `Compact`, 2 | `.oo__.`, `.o_o_.`, … | A "three-to-be": playing an eye makes a `Three`. |
| `NextOverFive` | `Double`, 4, never strict | `oo_ooo`, `ooo_oo`, … | Playing the eye makes an overline (6+). |

Because `strict` is applied for Black, `Four`/`Three` etc. already embody
"without at the same time making an overline". Example: in
`o.oooo.` the windows `o.ooo` and `.oooo` are rejected (a black stone sits in
their margin), and only `oooo.` counts — filling the other gap would make
six, filling the right end makes exactly five.

A `Structure` is `(start: Index, sequence: Sequence)`; `stones()` and
`eyes()` yield board `Point`s.

## 6. Forbidden moves (`forbidden.rs`)

Only Black has forbidden moves. Entry points:

```rust
pub fn forbidden_strict(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbiddens(q: &Square) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` is `Overline | DoubleFour | DoubleThree`. `forbidden_strict`
first applies rule 9.2's exception — if `p` is occupied, or playing `p`
makes a five (`structures_on(p, Black, Four)` is non-empty), it is *not*
forbidden — and then delegates to `forbidden`. The solvers
(`Game::is_forbidden_move` in `src/mate/game.rs`) call the non-strict
`forbidden`; a five-making move is recognised as a win by the search itself
before the forbidden check matters. `forbiddens` lists every forbidden empty
point on the board.

`forbidden` checks the three kinds in the order overline, double-four,
double-three (a point matching several is reported as the first).

### Overline (rule 9.2 a)

```rust
fn overline(q, p) -> bool { q.structures_on(p, Black, NextOverFive).next().is_some() }
```

`NextOverFive` = two adjacent 5-windows each holding 4 black stones, both
containing the empty `p`; together they span 6 cells with 5 stones, so `p`
would complete a run of six or more.

### Double-four (rule 9.2 b)

```rust
fn double_four(q, p) -> bool {
    distinctive(&mut q.structures_on(p, Black, Sword).map(|s| s.start_index()))
}
```

Every `Sword` through `p` becomes a `Four` when `p` is played. `distinctive`
returns true when the iterator yields at least two indices that are not
simply `first` and `first.walk(1)`. Two `Sword`s in adjacent windows of the
same line are the two halves of one straight four (`.oo_o.` → `.oooo.`),
which is a single four, so they are not counted twice. Two non-adjacent
windows — on different lines, or on the same line as in `o.o_o.o` → two
distinct fives-to-be — are a genuine double-four.

### Double-three (rules 9.2 c and 9.3)

```rust
fn double_three(q, p) -> bool {
    // cheap pre-filter: at least two "three-to-be" patterns through p
    if !distinctive(q.structures_on(p, Black, Two)) { return false; }
    let mut next = q.clone();
    next.put_mut(Black, p);
    truthy_double_three(&next, p)
}

fn truthy_double_three(next, p) -> bool {
    let truthy_threes = next.structures_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();   // the straight-four point
        forbidden_strict(next, eye).is_none()
    });
    distinctive(&mut truthy_threes.map(|s| s.start_index()))
}
```

1. `Two` structures through `p` are a necessary condition, so most points
   are rejected without cloning the board.
2. The move is played on a copy, and the real `Three`s through `p` are
   enumerated. A `Three` (`Compact`, 3) has exactly one eye — the point
   that would make the straight four.
3. Rule 9.3: a three only counts if that eye is itself a legal Black move.
   This is decided by calling `forbidden_strict` on the eye in the new
   position, which covers 9.3 a (the straight-four move would be an overline
   or double-four) and 9.3 b (it would be a forbidden double-three), and the
   recursion through `forbidden_strict → forbidden → double_three →
   truthy_double_three → forbidden_strict …` handles the "and so on" nesting
   the rule describes. Using the `_strict` variant matters: if the eye also
   completes a five on another line it is a legal (winning) move under 9.2
   even when it forms a double-four, so the three is real
   (`test_double_three_eye_makes_five`). Because the check is recursive, the five can also be one that the candidate move itself creates, in which case the answer for the candidate move flips (`test_double_three_nested_eye_makes_five`).
4. The move is a forbidden double-three when two or more *distinct* real
   threes remain.

Worked example from the tests (`test_double_three`): at `H8` in

```
. . . . x . o . o . x . . . .   <- row 8
```

plus stones above/below, the horizontal `x.o_o.x` cannot become a straight
four (the `x`s cap it), so only one real three passes through `H8` and the
move is legal. The Black-only shapes `.o_o.` in two directions give
`Some(DoubleThree)`. More cases, including the nested "fake three" positions
from the referenced Twitter thread, are in `forbidden.rs`'s tests.

## 7. Potentials (`potential.rs`)

Not part of the rules, but built on the same window scan. For each empty
cell of a line, `Potentials` looks at the five 5-windows containing it,
scores each valid window as `own stones + 1` (the number of stones it would
hold after playing there), and reports `max_score * (number of windows
achieving it)`. Windows are subject to the same `strict` margin rule as
sequences. `Square::potentials` / `potentials_along` expose this per
`Index`; `src/analysis/field.rs` aggregates it per point for move ordering.
`VICTORY = 5` is the score of a window that becomes a five.

## 8. Zobrist hashing (`zobrist.rs`) and `Board`

`Board` wraps a `Square` and a `u64` Zobrist hash kept in sync by
`put_mut`/`remove_mut`. `CODE_TABLE` has `2 * 225` random 64-bit codes,
indexed by `2 * u8::from(point) + (0 for Black | 1 for White)`, XORed in and
out on every change. `zobrist_hash_n(n)` XORs in an extra per-depth code
(`N_TABLE`) so that the solvers can key transposition tables on
(position, remaining depth). `Board::put`/`remove` return copies; the
solvers use the `_mut` variants to avoid cloning in the search loop.

## 9. Cheat sheet: rule → code

| Rule | Code |
| --- | --- |
| Five wins | `structures(r, Five)` (checked in `mate::solve` / `Game`). |
| Overline wins for White, not Black | `Five` is strict only for Black, so a White six is still a `Five`; a Black overline is a forbidden move (`NextOverFive`). `mate::solve::validate` rejects input positions that already contain a five or a Black `OverFive`. |
| Four / straight four | `Four` (`Single`, 4) / `OpenFour` (`Compact`, 4); a straight four = two adjacent `Four`s. |
| Three (must reach a straight four) | `Three` (`Compact`, 3), single eye = the straight-four point. |
| "Without making an overline" for Black | `strict` margins in `Sequences`. |
| Forbidden: overline / double-four / double-three | `forbidden.rs`: `overline` / `double_four` / `double_three`. |
| 9.2 "unless it makes a five" | `forbidden_strict`. |
| 9.3 real vs. fake threes, recursive | `truthy_double_three` calling `forbidden_strict` on each three's eye. |
