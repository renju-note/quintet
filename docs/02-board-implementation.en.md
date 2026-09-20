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

A point on the board is a `Point(x, y)`.

- Both `x` and `y` are integers from `0` up to but not including `RANGE`
  (= 15).
- `x` is the column: column `A` is 0 and column `O` is 14.
- `y` is the row: row `1` is 0 and row `15` is 14.

Conversion to and from text uses the usual Renju notation such as `H8`
(the `Display` / `FromStr` implementations). `Points`, a list of points, is
written comma-separated, as in `H8,H7,F6`.

At the wasm/JS boundary a point is passed as a single byte. The encoding is
`code = x * 15 + y`, implemented by `From<Point> for u8` and
`TryFrom<u8> for Point`. This encoding is part of the public API and must
not change.

### Lines and `Index`

Every point lies on exactly four lines: a column, a row, an ascending
diagonal and a descending diagonal. A line is identified by its `Direction`
and a line number `i`, and a position on the line by `j`:

| `Direction` | Line type | Line number `i` | Position on the line `j` |
| --- | --- | --- | --- |
| `Vertical` | column (`\|`) | `x` | `y` |
| `Horizontal` | row (`-`) | `y` | `x` |
| `Ascending` | ascending diagonal (`/`) | `x + 14 - y` (0 through 28) | `x` if `i < 14`, else `y` |
| `Descending` | descending diagonal (`\`) | `x + y` (0 through 28) | `x` if `i < 14`, else `14 - y` |

The triple `(Direction, i, j)` is an `Index`. Related operations:

- `Point::to_index(d)` converts a point to the `Index` for direction `d`,
  and `Index::to_point()` converts back.
- `Index::walk(step)` and `walk_checked(step)` move `step` cells along the
  same line.
- `Index::maxj()` gives the last valid position on the line. Diagonals get
  shorter towards the corners, so the value differs from line to line.

## 2. `Line`: one line as bitmasks

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

- `blacks` and `whites` are bitmasks of the black and white stones: bit `j`
  is set when position `j` holds a stone of that colour.
- `size` is the line length: 15 for rows and columns, and anywhere from 5
  to 15 for diagonals (short diagonals are omitted, see §4).

Placing a stone (`put_mut`), removing one (`remove_mut`) and reading a
position (`stone(j)`) are all plain bit operations, which is what keeps the
search loop cheap.

`potential_cap(r)` is a quick upper bound used to skip whole lines. It
returns 0 when the line cannot even fit five stones between the opponent's
stones, and "own stones + 1" otherwise.

## 3. `Sequences`: finding patterns in a line

`Sequences` (`sequence.rs`) is an iterator that slides a **5-cell window**
over a line and reports the windows that match a pattern. It is the single
primitive from which every rule concept is built.

For each window start `i` it takes 7 bits from each colour's mask:

```
bit  :   6   |  5    4    3    2    1  |   0
cell :  i+5  | i+4  i+3  i+2  i+1   i  |  i-1
     : right |          target         | left
     : margin|        (5 cells)        | margin
```

The own-stone mask `my` and the opponent mask `op` are shifted left by one
in advance, so the bit for `i-1` exists even when `i = 0`. Margins that fall
outside the board read as empty.

A window is considered **valid** when all of the following hold:

- No opponent stone is in the 5 target cells (`op & TARGET_MASK == 0`).
- In `strict` mode, additionally no *own* stone is in either margin
  (`my & MARGIN_MASK == 0`).

`strict` is set exactly when the player is **Black** (see
`StructureKind::to_sequence`). It implements the rule that a completed five
for Black must be *exactly* five: if an own stone is adjacent to the window,
making a five in that window would produce an overline, so the window does
not count as a five-to-be. White has no such restriction, so its windows are
scanned non-strictly and an overline is treated like any other win.

For each valid window the number of own stones is compared with `n`. Three
`SequenceKind`s decide what is reported:

| `SequenceKind` | Condition | Meaning |
| --- | --- | --- |
| `Single` | window `i` has exactly `n` own stones | A place where adding `5 - n` stones makes a five. |
| `Double` | window `i-1` **and** window `i` both have `n` own stones | Two overlapping fives-to-be, i.e. a 6-cell span `i-1..=i+4`. Its two ends are either both stones (`n + 1` stones in 6 cells) or both empty (the `Open` case below). |
| `Open` | the `Double` case whose ends `i-1` and `i+4` are both empty, so the `n` own stones all lie in the 4 cells `i..=i+3` | A pattern with **open ends** on both sides, as in *open three* / *open four*. |

The reported item is a pair `(i, Sequence)`, where `Sequence` is a 5-bit
mask describing the target cells.

- `Sequence::stones()` and `eyes()` map that mask to offsets `0..5`: set
  bits are stones, empty cells are "eyes".
- For `Open` the 5th bit is forced on (`LAST_MASK`) so that the closing
  empty cell `i+4` is *not* reported as an eye. As a side effect that cell
  does show up in `stones()`, so read `stones()` of a `Open` structure as
  "cells that are not eyes".

`Sequences::new_on(j, …)` is a variant that scans only the windows
containing position `j`. It is what `structures_on(p, …)` uses to ask
"which patterns does this move touch?".

## 4. `Square`: the full board

```rust
pub struct Square {
    vlines: [Line; 15],  // columns,   indexed by x
    hlines: [Line; 15],  // rows,      indexed by y
    alines: [Line; 21],  // ascending  diagonals with length >= 5
    dlines: [Line; 21],  // descending diagonals with length >= 5
}
```

A stone is stored redundantly in all four lines through its point
(`put_mut` updates each of them).

Diagonals shorter than five cells can never contain a five, so the four
shortest diagonals at each corner are omitted:

- Diagonal `i` for `i` from `4` through `24` is stored at `alines[i - 4]`
  (`D_LINE_OMIT = 4`, `D_LINE_NUM = 21`).
- For the other diagonals `line_idx` returns `None`, and pattern queries
  simply skip them.

Main queries:

- `stone(p)`, `stones(player)`, `empties()`, `neighbors(p, distance, only_empty)`
  — reading stones and empty points.
- `structures(r, kind)` — every `Structure` of kind `kind` for player `r`
  on the whole board.
- `structures_on(p, r, kind)` — only the structures whose 5-window contains
  point `p` (just the four lines through `p` are examined). This is the hot
  path for "what does playing `p` create?".
- `potentials(...)` / `potentials_along(...)` — see §7.

Parsing from text (`FromStr for Square`, reused by `Board`) accepts three
formats:

- A move list such as `H8,H7,F6`, alternating from Black.
- A stone list such as `H8,F6/H7`, written as `blacks/whites`.
- A 15-line ASCII picture: `o` is Black, `x` is White, `.` is empty, and
  row 15 comes first. This is the format used throughout the tests.

## 5. `StructureKind`: the rule vocabulary

`StructureKind::to_sequence(r)` maps each kind to a triple
`(SequenceKind, n, strict)`. `strict` is normally `r.is_black()` (true for
Black only); the overline kinds are never strict:

| `StructureKind` | `SequenceKind` | `n` | `strict` | Pattern (Black shown, `_` = eye) | Rule concept |
| --- | --- | --- | --- | --- | --- |
| `Five` | `Single` | 5 | Black only | `ooooo` | **Five** (§3). For Black, strict margins exclude overlines. |
| `OverFive` | `Double` | 5 | never | `oooooo` (6+) | **Overline**. |
| `Four` | `Single` | 4 | Black only | `oooo_`, `ooo_o`, `oo_oo`, … | **Four**: one more stone at the eye makes a five. A straight four appears as **two** adjacent `Four`s. |
| `OpenFour` | `Open` | 4 | Black only | `.oooo.` | **Straight four**. |
| `Sword` | `Single` | 3 | Black only | `ooo__`, `o_oo_`, … (3 stones in a 5-window) | A "four-to-be": playing either eye makes a `Four`. Not a rule term; used by VCF/VCT to enumerate four-making moves. |
| `Three` | `Open` | 3 | Black only | `.ooo_.`, `.oo_o.`, `.o_oo.`, `._ooo.` | **Three**: playing the single eye makes an `OpenFour`. |
| `Two` | `Open` | 2 | Black only | `.oo__.`, `.o_o_.`, … | A "three-to-be": playing an eye makes a `Three`. |
| `NextOverFive` | `Double` | 4 | never | `oo_ooo`, `ooo_oo`, … | Playing the eye makes an overline (6+). |

Because `strict` is applied for Black, kinds such as `Four` and `Three`
already embody the condition "without at the same time making an overline".
Consider the shape `o.oooo.` as an example:

- Window `o.ooo` and window `.oooo` are rejected because a black stone sits
  in their margin: filling the gap on the left would make six.
- Only window `oooo.` counts: playing the right end makes exactly five.

A `Structure` is a pair `(start: Index, sequence: Sequence)`; `stones()` and
`eyes()` yield board `Point`s.

## 6. Forbidden moves (`forbidden.rs`)

Only Black has forbidden moves. There are three entry points:

```rust
pub fn forbidden_strict(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbiddens(q: &Square) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` is one of `Overline`, `DoubleFour` or `DoubleThree`.

- `forbidden_strict` first applies the exception in rule 9.2: if `p` is
  already occupied, or playing `p` makes a five (`structures_on(p, Black,
  Four)` is non-empty), the move is *not* forbidden. Otherwise it delegates
  to `forbidden`.
- `forbidden` checks overline, double-four and double-three in that order;
  a point matching several kinds is reported as the first one found.
- `forbiddens` lists every forbidden empty point on the board.

The solvers (`Game::is_forbidden_move` in `src/mate/game.rs`) call the
non-strict `forbidden`, because a five-making move is recognised as a win
by the search itself before the forbidden check matters.

### Overline (rule 9.2 a)

```rust
fn overline(q, p) -> bool { q.structures_on(p, Black, NextOverFive).next().is_some() }
```

A `NextOverFive` is two adjacent 5-windows, each holding 4 black stones and
both containing the empty point `p`. Together they span 6 cells with 5
stones, so playing `p` completes a run of six or more.

### Double-four (rule 9.2 b)

```rust
fn double_four(q, p) -> bool {
    distinctive(&mut q.structures_on(p, Black, Sword).map(|s| s.start_index()))
}
```

Every `Sword` through `p` becomes a `Four` when `p` is played. `distinctive`
returns true when the indices yielded by the iterator contain at least two
that are not simply `first` and `first.walk(1)`. That exclusion is needed
for the following reason:

- Two `Sword`s in adjacent windows of the same line are the two halves of
  one straight four (`.oo_o.` → `.oooo.`). That is a single four, so they
  are not counted twice.
- Two non-adjacent windows are a genuine double-four. This is obviously the
  case on different lines, but also on the same line when the shape is like
  `o.o_o.o`, which gives two distinct fives-to-be.

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

The check proceeds in these steps:

1. Having two or more `Two` structures through `p` is a necessary condition
   for a double-three, so it is checked first. Most points are rejected
   here, without cloning the board.
2. The move is played on a copy, and the real `Three`s through `p` are
   enumerated. A `Three` (`Open`, 3) has exactly one eye, which is the
   straight-four point.
3. Following rule 9.3, a three only counts if that eye is itself a legal
   Black move. This is decided by calling `forbidden_strict` on the eye in
   the new position (details below).
4. The move is a forbidden double-three when two or more *distinct* real
   threes remain.

Some notes on the check in step 3:

- Calling `forbidden_strict` on the eye covers both 9.3 a (the straight-four
  move would be an overline or double-four) and 9.3 b (it would be a
  forbidden double-three).
- The recursion `forbidden_strict → forbidden → double_three →
  truthy_double_three → forbidden_strict → …` handles the "and so on"
  nesting the rule describes.
- Using the `_strict` variant matters: if the eye also completes a five on
  another line, that move is legal (and winning) under 9.2 even when it
  forms a double-four, so the three is real
  (`test_double_three_eye_makes_five`).
- Because the check is recursive, the five can also be one created by the
  candidate move itself, in which case the answer for the candidate move
  flips (`test_double_three_nested_eye_makes_five`).
- Note that RIF 9.3 a) literally says "without … an overline or double-four
  is attained" and does not restate the 9.2 five exception. This
  implementation reads it as "without making a *forbidden* move", which is
  consistent with 9.2 and with the Japanese rule definition of a three.

As a worked example from the tests (`test_double_three`), consider playing
`H8` in this position:

```
. . . . x . o . o . x . . . .   <- row 8
```

with further stones above and below. The horizontal `x.o_o.x` cannot become
a straight four because the `x`s cap it, so only one real three passes
through `H8` and the move is legal. If both directions were the Black-only
shape `.o_o.` instead, the result would be `Some(DoubleThree)`. More cases,
including the nested "fake three" positions from the referenced Twitter
thread, are in `forbidden.rs`'s tests.

## 7. Potentials (`potential.rs`)

Not part of the rules, but built on the same window scan. For each empty
cell of a line, `Potentials` computes a score as follows:

1. Look at the five 5-windows containing the cell. Windows are subject to
   the same `strict` margin rule as in `Sequences`.
2. Score each valid window as "own stones + 1", the number of stones it
   would hold after playing there.
3. Report "max score × number of windows achieving that max" as the value of
   the cell.

`Square::potentials` / `potentials_along` expose this per `Index`, and
`src/analysis/field.rs` aggregates it per point for move ordering.
`VICTORY = 5` is the score of a window that becomes a five.

## 8. Zobrist hashing (`zobrist.rs`) and `Board`

`Board` wraps a `Square` and a `u64` Zobrist hash, and `put_mut` /
`remove_mut` keep the two in sync.

- `CODE_TABLE` holds `2 * 225` random 64-bit codes. The index is
  `2 * u8::from(point) + c`, where `c` is 0 for Black and 1 for White.
- Whenever a stone is placed or removed, the corresponding code is XORed
  into or out of the hash.
- `zobrist_hash_n(n)` XORs in an extra per-depth code (`N_TABLE`) so that
  the solvers can key transposition tables on (position, remaining depth).
- `Board::put` / `remove` return copies; the solvers use the `_mut`
  variants to avoid cloning in the search loop.

## 9. Cheat sheet: rule → code

| Rule | Code |
| --- | --- |
| Five wins | `structures(r, Five)` (checked in `mate::solve` / `Game`). |
| Overline wins for White, not Black | `Five` is strict only for Black, so a White six is still a `Five`; a Black overline is a forbidden move (`NextOverFive`). `mate::solve::validate` rejects input positions that already contain a five or a Black `OverFive`. |
| Four / straight four | `Four` (`Single`, 4) / `OpenFour` (`Open`, 4); a straight four = two adjacent `Four`s. |
| Three (must reach a straight four) | `Three` (`Open`, 3), single eye = the straight-four point. |
| "Without making an overline" for Black | `strict` margins in `Sequences`. |
| Forbidden: overline / double-four / double-three | `forbidden.rs`: `overline` / `double_four` / `double_three`. |
| 9.2 "unless it makes a five" | `forbidden_strict`. |
| 9.3 real vs. fake threes, recursive | `truthy_double_three` calling `forbidden_strict` on each three's eye. |
