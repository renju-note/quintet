# How `src/board/` implements the Renju rules

This document explains the board representation and how the rule concepts
from [01-renju-rules.en.md](01-renju-rules.en.md) — row, five, overline, four, straight
four, three, double-four, double-three, forbidden move — are detected. It is
written against the current code; identifiers in backticks can be grepped.

A note on terms: "row" always means a row as defined in 01 §3 (`Row` in
the code). The Japanese version holds to the same: 連 there always means a
row, although Japanese sometimes uses it for a `Two`, and a `Two` is
called 二連.

Module map (`src/board.rs` declares the modules):

| File | Role |
| --- | --- |
| `player.rs` | `Player` (`Black` / `White`) and its text form (`o` / `x`). |
| `point.rs` | `Point` (x, y), `Points`, `Direction`, `Index` (position along a line). |
| `segment.rs` | `Segment`: five consecutive cells of a line (one place for a five) and the cell on each side. |
| `line.rs` | `Line`: one horizontal, vertical or diagonal line as two bitmasks; its segments, and the rows and potentials found from them. |
| `row.rs` | `RowKind` (Two, Three, Sword, Four, Five, ...) and `Row` (a row of one player's stones as defined in 01 §3, located on the board). |
| `grid.rs` | `Grid`: the whole 15×15 board as four arrays of `Line`s, plus row queries. |
| `forbidden.rs` | Renju forbidden-move detection for Black. |
| `zobrist.rs` | Zobrist hashing for transposition tables. |
| `board.rs` | `Board` = `Grid` + Zobrist hash; the public facade used by the solvers. |

The pieces are layered, and the sections below follow the layers from the
bottom up:

1. `Line` stores one line of the board as bitmasks (§2).
2. A `Segment` is one place on a line where a five can be made, and says
   how far each player is from making it there (§3).
3. `Grid` holds all the lines and answers "which rows are on the
   board / through this point?" (§4).
4. `RowKind` names those rows in rule vocabulary — `Five`, `Four`,
   `Three`, ... — each as one or two segments with the right score (§5).
5. `forbidden.rs` combines a few `rows_on` queries into the
   forbidden-move rules (§6).
6. Potentials reuse the segments' scores for move ordering (§7), and
   `zobrist.rs` hashes the board (§8).

---

## 1. Points and coordinates

A point on the board is a `Point(x, y)`.

- Both `x` and `y` are integers from `0` up to but not including `RANGE`
  (= 15).
- `x` is the vertical line: line `A` is 0 and line `O` is 14.
- `y` is the horizontal line: line `1` is 0 and line `15` is 14.

Conversion to and from text uses the usual Renju notation such as `H8`
(the `Display` / `FromStr` implementations). `Points`, a list of points, is
written comma-separated, as in `H8,H7,F6`.

### Lines and `Index`

Every point lies on exactly four lines: a vertical line, a horizontal
line, an ascending diagonal and a descending diagonal. A line is identified
by its `Direction` and a line number `i`, and a position on the line by `j`:

| `Direction` | Line type | Line number `i` | Position on the line `j` |
| --- | --- | --- | --- |
| `Vertical` | vertical line (`\|`) | `x` | `y` |
| `Horizontal` | horizontal line (`-`) | `y` | `x` |
| `Ascending` | ascending diagonal (`/`) | `x + 14 - y` (0 through 28) | `x` if `i < 14`, else `y` |
| `Descending` | descending diagonal (`\`) | `x + y` (0 through 28) | `x` if `i < 14`, else `14 - y` |

Ascending diagonals are numbered from the top-left corner (`A15` is on
`i = 0`) to the bottom-right corner (`O1` is on `i = 28`), descending
diagonals from `A1` (`i = 0`) to `O15` (`i = 28`). On every diagonal `j`
counts from its leftmost point, so `j` grows with `x`; the two formulas in
the table only differ in whether the diagonal starts on the left edge
(`i < 14`) or on the bottom edge (ascending) / top edge (descending).

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
- `size` is the line length: 15 for horizontal and vertical lines, and anywhere from 5
  to 15 for diagonals (short diagonals are omitted, see §4).

Placing a stone (`put_mut`), removing one (`remove_mut`) and reading a
position (`stone(j)`) are all plain bit operations, which is what keeps the
search loop cheap.

`potential_cap(r)` is a quick upper bound used to skip whole lines. It
returns 0 when the line cannot even fit five stones between the opponent's
stones, and "own stones + 1" otherwise.

`segment(j)` cuts out the segment (§3) whose five cells start at cell `j`,
and `segments()` lists them all, `j` from 0 to `size - 5`. Everything else a
`Line` answers is built on them: `rows(r, kind)` (§3.1, §5) and
`potentials(r, min)` (§7).

## 3. `Segment`: one place for a five

A `Segment` (`segment.rs`) is five consecutive cells of a line — one place
where a five can be made — together with the cell just before them and the
cell just after. It is the single primitive from which every rule concept is
built.

```rust
pub struct Segment { blacks: u8, whites: u8 }
```

Each colour's stones are 7 bits, taken from the line by `Line::segment(j)`:

```
bit  :   6   |  5    4    3    2    1  |   0
cell :  j+5  | j+4  j+3  j+2  j+1   j  |  j-1
     : margin|     the five cells      | margin
```

Cells beyond the edge of the line read as empty. The five cells are
numbered 0-4 when a segment reports positions.

A segment answers, for either player `r`:

| Method | Answer |
| --- | --- |
| `free(r)` | No opponent stone is among the five cells. |
| `alive(r)` | `r` can still make a five here: `free(r)`, and for Black also no black stone in either margin. |
| `count(r)` | How many of the five cells hold `r`'s stones, 0-5, alive or not. |
| `score(r)` | `count(r)` if `alive(r)`, else `-1`. |
| `stones(r)` | The cells (0-4) holding `r`'s stones. `stone_bits(r)` is the same as a 5-bit mask. |
| `eyes(r)` | The cells (0-4) `r` still has to play to make a five here; none when not alive. `eye_bits(r)` is the same as a mask. |

The margins are what make Black's rule work. A five for Black must be
*exactly* five: if a black stone is next to the five cells, filling them in
makes an overline, so the segment is dead for Black. White has no such
restriction; its margins do not matter, and an overline is a win like any
other.

### 3.1 Finding rows on a line, all segments at once

`Line::rows(r, kind)` gives the segments where `r` has a row of
that kind (§5), as `(j, Segment)`. Checking the segments one by one is what
`RowKind::matches` states, but the solvers ask at nearly every node,
so the line finds them all at once with bit operations: bit `j` of `x >> k`
is cell `j + k`, so a condition written over shifted masks is checked for
every segment in parallel.

- `counting(r, n)`: bit `j` is set if segment `j` is `free(r)` and has
  `count(r) == n` — the five cells added up bit-sliced (a full adder, a half
  adder and the carries).
- `scoring(r, n)`: the same with `score(r) == n`, i.e. also no black stone
  at `j - 1` or `j + 5` for Black.
- `row_starts(r, kind)`: bit `j` is set if the row is at segment
  `j`, from the above. `rows` walks its set bits (`Bits`, one
  `trailing_zeros` per row).

A test checks `row_starts` against `RowKind::matches` on every
line of up to nine cells and on random full-length lines.

`rows_on(i, r, kind)` keeps only the rows through cell `i`: the
segment has `i` among its five cells, and for a row of two segments the
earlier one does too, so `i` is in the four cells they share.
`row_starts_on(i, r, kind)` is the same as a mask. It is what
`rows_on(p, …)` uses to ask "which rows does this move touch?".

## 4. `Grid`: the full board

```rust
pub struct Grid {
    vlines: [Line; 15],  // vertical lines,   indexed by x
    hlines: [Line; 15],  // horizontal lines, indexed by y
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
- For the other diagonals `line_idx` returns `None`, and row queries
  simply skip them.

Main queries:

- `stone(p)`, `stones(player)`, `empties()`, `neighbors(p, distance, only_empty)`
  — reading stones and empty points.
- `rows(r, kind)` — every `Row` of kind `kind` for player `r`
  on the whole board.
- `rows_on(p, r, kind)` — only the rows through point `p`
  (§3.1; just the four lines through `p` are examined). This is the hot
  path for "what does playing `p` create?".
- `line(d, i)` / `line_on(p, d)` — the stored `Line` itself, `None` for the
  short diagonals. `lines()` and `lines_on(p)` iterate them as
  `(Direction, i, &Line)`. Consumers that keep their own per-point tables can
  read these lines instead of maintaining a second copy of all 72.
- `potentials(...)` / `potentials_along(...)` — see §7.

Parsing from text (`FromStr for Grid`, reused by `Board`) accepts three
formats:

- A move list such as `H8,H7,F6`, alternating from Black.
- A stone list such as `H8,F6/H7`, written as `blacks/whites`.
- A 15-line ASCII picture: `o` is Black, `x` is White, `.` is empty, and
  horizontal line 15 comes first. This is the format used throughout the
  tests.

## 5. `RowKind`: the rule vocabulary

Each `RowKind` is one segment, or two neighbouring ones (segments
`j - 1` and `j`, together spanning the six cells `j - 1..=j + 4`), with the
right scores. `RowKind::matches(r, prev, cur)` states it for the
segment `cur` and the one before it, `prev`; a row of two is reported at
the later segment.

| `RowKind` | Segments | Pattern (Black shown, `_` = eye) | Rule concept |
| --- | --- | --- | --- |
| `Five` | one scoring 5 | `ooooo` | **Five**. For Black, a score needs empty margins, so overlines are excluded. |
| `Overlined` | two, each `free` with 5 stones | `oooooo` (6+) | **Overline**. |
| `Four` | one scoring 4 | `oooo_`, `ooo_o`, `oo_oo`, … | **Four**: one more stone at the eye makes a five. A straight four appears as **two** adjacent `Four`s. |
| `Straight` | two scoring 4, the stones in the four cells they share | `.oooo.` | **Straight four**. |
| `Sword` | one scoring 3 | `ooo__`, `o_oo_`, … (3 stones in a segment) | A "four-to-be" (Japanese *kensaki*, "sword tip"): playing either eye makes a `Four`. Includes open threes, so it is not the same as a "closed three". Not a rule term; used by VCF/VCT to enumerate four-making moves. |
| `Three` | two scoring 3, the stones in the four cells they share | `.ooo_.`, `.oo_o.`, `.o_oo.`, `._ooo.` | **Three**: playing the single eye makes a `Straight`. |
| `Two` | two scoring 2, the stones in the four cells they share | `.oo__.`, `.o_o_.`, … | A "three-to-be": playing an eye makes a `Three`. |
| `Overlining` | two, each `free` with 4 stones | `oo_ooo`, `ooo_oo`, … | Playing the eye makes an overline (6+). |

For the open rows (`Two`, `Three`, `Straight`) "the stones in the four
shared cells" means the later segment's last cell is empty; since both
segments are alive, the six cells they span then have both ends empty. The
overline rows look at `free` segments rather than `alive` ones: an
overline always has a black stone next to each of its segments, which is
exactly what makes a segment dead for Black. Two `free` segments with four
black stones each are either five stones in six cells (the empty one makes
six) or an open four `.oooo.`; through an empty point, only the former can
be found, since an open four's segments share only stones.

Because a segment is alive for Black only with empty margins, kinds such as
`Four` and `Three` already embody the condition "without at the same time
making an overline". Consider the shape `o.oooo.` as an example:

- Segments `o.ooo` and `.oooo` are dead: a black stone sits in their
  margin, so filling the gap on the left would make six.
- Only segment `oooo.` counts: playing the right end makes exactly five.

A `Row` is where the row's (later) segment starts, an `Index`,
with the masks of its stones and eyes; `stones()` and `eyes()` yield board
`Point`s. For the open rows only the four shared cells can be eyes: the
fifth is an open end, not a point to play.

## 6. Forbidden moves (`forbidden.rs`)

Only Black has forbidden moves. There are three entry points:

```rust
pub fn forbidden_strict(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbiddens(g: &Grid) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` is one of `Overline`, `DoubleFour` or `DoubleThree`.

- `forbidden_strict` first applies the exception in rule 9.2: if `p` is
  already occupied, or playing `p` makes a five (`row_starts_on(p, Black,
  Four)` has a start), the move is *not* forbidden. Otherwise it delegates
  to `forbidden`.
- `forbidden` checks overline, double-four and double-three in that order;
  a point matching several kinds is reported as the first one found.
- `forbiddens` lists every forbidden empty point on the board.

The solvers (`Game::is_forbidden_move` in `src/mate/game.rs`) call the
non-strict `forbidden`, because a five-making move is recognised as a win
by the search itself before the forbidden check matters.

### Overline (rule 9.2 a)

```rust
fn overline(g, p) -> bool { any(g.row_starts_on(p, Black, Overlining)) }
```

`Grid::row_starts_on(p, r, kind)` gives, per line through `p`, the mask
`Line::row_starts_on` of where the rows `rows_on(p, r, kind)` would give
start (§3.1), without building them. Asking whether there is one (`any`),
or more than one (`distinctive_starts`, below), is then a test on bits.

An `Overlining` is two adjacent segments, each holding 4 black stones and
both containing the empty point `p`. Together they span 6 cells with 5
stones, so playing `p` completes a run of six or more.

### Double-four (rule 9.2 b)

```rust
fn double_four(g, p) -> bool {
    distinctive_starts(g.row_starts_on(p, Black, Sword))
}
```

Every `Sword` through `p` becomes a `Four` when `p` is played. `distinctive`
returns true as soon as the iterator yields an index other than the first
segment's `first` and its neighbour `first.walk(1)`: it counts two adjacent
segments as one and asks whether there are at least two segments left. The
neighbour is excluded for the following reason:

- Two `Sword`s in adjacent segments of the same line are the two halves of
  one straight four (`.oo_o.` → `.oooo.`). That is a single four, so they
  are not counted twice.
- Two non-adjacent segments are a genuine double-four. This is obviously the
  case on different lines, but also on the same line when the shape is like
  `o.o_o.o`, which gives two distinct fives-to-be.

`distinctive_starts` asks the same of the masks: a second line with a start,
or, on the first one, a start other than its lowest `j` and `j + 1`
(`s & !(0b11 << s.trailing_zeros()) != 0`). `distinctive` itself is still
used on the `Three`s below, which have to be looked at one by one.

### Double-three (rules 9.2 c and 9.3)

```rust
fn double_three(g, p) -> bool {
    // cheap pre-filter: at least two "three-to-be" rows through p
    if !distinctive_starts(g.row_starts_on(p, Black, Two)) { return false; }
    let mut next = g.clone();
    next.put_mut(Black, p);
    truthy_double_three(&next, p)
}

fn truthy_double_three(next, p) -> bool {
    let truthy_threes = next.rows_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();   // the straight-four point
        forbidden_strict(next, eye).is_none()
    });
    distinctive(&mut truthy_threes.map(|s| s.start_index()))
}
```

The check proceeds in these steps:

1. Having two or more `Two` rows through `p` is a necessary condition
   for a double-three, so it is checked first. Most points are rejected
   here, without cloning the board.
2. The move is played on a copy, and the real `Three`s through `p` are
   enumerated. A `Three` has exactly one eye, which is the
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
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . o . . . . . . .
 . . . . x . o . o . x . . . .
 . . . . . . . o . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

Only the vertical direction has a `Two` through `H8`: `.o_o.` on vertical line H
(it is found at two adjacent segments, so `distinctive` counts it once).
The horizontal `x.o_o.x` is capped by the `x`s, so no pair of segments
fits and it is not a `Two` — it could never become a straight four.
The pre-filter in step 1 therefore rejects the move without cloning the
board, and the result is `None`. Remove the two `x`s and both directions
have a `Two`; after playing `H8` both become `Three`s whose straight-four
points are legal, so the result is `Some(DoubleThree)`. More cases,
including the nested "fake three" positions from the referenced Twitter
thread, are in `forbidden.rs`'s tests.

## 7. Potentials (`Line::potentials`)

Not part of the rules, but built on the same segments. For each empty cell
of a line, `Line::potentials(r, min)` computes a value as follows:

1. Each segment is worth `score(r) + 1`: the number of stones it would hold
   after playing there, or 0 if it is dead (§3). Values below `min` count
   as 0.
2. A cell is worth the best segment through it (at most five), times the
   number of segments through it reaching that best.
3. Cells below `min` are not reported.

It is worked out on the same bit-sliced counts as `counting`: one mask per
score of the segments alive and worth at least `min`, and for each empty
cell, from the best score down, `count_ones` of that mask over the (at
most five) segments through the cell. The first score with any is the
best, and the count is how many reach it. A test checks this against the
segment-by-segment definition above, on the same lines as `row_starts`.

`Grid::potentials` / `potentials_along` expose this per `Index`, and
`src/feature/potential.rs` aggregates it per point for move ordering.
`VICTORY = 5` is the length of a five.

## 8. Zobrist hashing (`zobrist.rs`) and `Board`

`Board` wraps a `Grid` and a `u64` Zobrist hash, and `put_mut` /
`remove_mut` keep the two in sync.

- `CODE_TABLE` holds `2 * 225` random 64-bit codes. The index is
  `2 * u8::from(point) + c`, where `c` is 0 for Black and 1 for White.
- Whenever a stone is placed or removed, the corresponding code is XORed
  into or out of the hash.
- `zobrist_hash_n(n)` XORs in an extra per-depth code (`N_TABLE`) so that
  the solvers can key transposition tables on (position, remaining depth).
- `Board::put` / `remove` return copies; the solvers use the `_mut`
  variants to avoid cloning in the search loop.

The VCF search asks for each player's swords (`Sword`, §5) at nearly
every node, so it keeps them cached in a `SwordMap`
(`src/feature/sword.rs`). `Board` does not hold it: like the VCT's
`PotentialField`, the search states do (`VCFState`, and `VCTState` to hand
to its nested VCFs), marking it from `State::after_play` / `after_undo`
and passing the board along when they sync or read it.

- Per player and per line (by `Grid::line_key`, the line's position in
  `Grid::lines`), a `u16` with bit `j` set if a sword's segment starts at
  cell `j`, and a `u128` of the lines that have any.
- A move only marks the (at most four) lines through the point stale
  (`SwordMap::mark_stale`); `SwordMap::sync` recomputes the stale lines. The searches move
  far more often than they read, so recomputing at every move would cost
  more than the scan it replaces.
- A line is recomputed by `Line::row_starts(r, Sword)`, which checks
  every segment at once with bit operations (§3.1).
- `SwordMap::swords(board, r)` / `swords_on(board, p, r)` read the cache
  and return what `rows(r, Sword)` / `rows_on(p, r, Sword)`
  would, in the same order. They require `sync(board)` first.

## 9. Cheat sheet: rule → code

| Rule | Code |
| --- | --- |
| Five wins | `rows(r, Five)` (checked in `mate::solve` / `Game`). |
| Overline wins for White, not Black | `Five` is exact only for Black, so a White six is still a `Five`; a Black overline is a forbidden move (`Overlining`). `mate::solve::validate` rejects input positions that already contain a five or a Black `Overlined`. |
| Four / straight four | `Four` (a segment scoring 4) / `Straight` (two scoring 4); a straight four = two adjacent `Four`s. |
| Three (must reach a straight four) | `Three` (two segments scoring 3), single eye = the straight-four point. |
| "Without making an overline" for Black | `Segment::alive`: no black stone in the margins. |
| Forbidden: overline / double-four / double-three | `forbidden.rs`: `overline` / `double_four` / `double_three`. |
| 9.2 "unless it makes a five" | `forbidden_strict`. |
| 9.3 real vs. fake threes, recursive | `truthy_double_three` calling `forbidden_strict` on each three's eye. |
