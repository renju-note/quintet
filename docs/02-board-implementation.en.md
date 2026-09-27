# How `src/board/` implements the Renju rules

How the board is represented, and how the rule concepts of
[01-renju-rules.en.md](01-renju-rules.en.md) are detected: row, five,
overline, four, straight four, three, double-four, double-three, forbidden
move. Identifiers in backticks can be grepped.

On terms: "row" always means a row as in 01 §3 (`Row` in the code). The
Japanese version does the same: 連 is always a row, and a `Two` is 二連.

Modules (declared in `src/board.rs`):

| File | Role |
| --- | --- |
| `player.rs` | `Player` (`Black` / `White`) and its text form (`o` / `x`). |
| `point.rs` | `Point` (x, y), `Points`, `Direction`, `Index` (position along a line). |
| `segment.rs` | `Segment`: five consecutive cells of a line (one place for a five) plus the cell on each side. |
| `line.rs` | `Line`: one line as two bitmasks; its segments and the rows found from them. |
| `row.rs` | `RowKind` (Two, Three, Sword, Four, Five, ...) and `Row` (one player's row, located on the board). |
| `grid.rs` | `Grid`: the 15×15 board as four arrays of `Line`s, plus row queries. |
| `forbidden.rs` | Forbidden-move detection for Black. |
| `zobrist.rs` | Zobrist hashing for transposition tables. |
| `board.rs` | `Board` = `Grid` + Zobrist hash; the facade the solvers use. |

The layers, bottom up, and the sections that describe them:

1. `Line` stores one line as bitmasks (§2).
2. `Segment` is one place for a five, and how far each player is from it
   (§3).
3. `Grid` holds all lines and finds rows on the board or through a point
   (§4).
4. `RowKind` names the rows in rule terms — `Five`, `Four`, `Three`, ... —
   each as one or two segments with the right score (§5).
5. `forbidden.rs` builds the forbidden-move rules from a few `rows_on`
   queries (§6).
6. `zobrist.rs` hashes the board (§7).

---

## 1. Points and coordinates

A point is a `Point(x, y)`:

- `x` and `y` are in `0..SIZE` (`SIZE` = 15).
- `x` is the vertical line: `A` is 0, `O` is 14.
- `y` is the horizontal line: `1` is 0, `15` is 14.

Text uses the usual notation, such as `H8` (`Display` / `FromStr`).
`Points` is written comma-separated: `H8,H7,F6`.

### Lines and `Index`

Every point is on four lines: vertical, horizontal, ascending diagonal,
descending diagonal. A line is a `Direction` and a line number `i`; a
position on it is `j`.

| `Direction` | Line | Line number `i` | Position `j` |
| --- | --- | --- | --- |
| `Vertical` | vertical (`\|`) | `x` | `y` |
| `Horizontal` | horizontal (`-`) | `y` | `x` |
| `Ascending` | ascending diagonal (`/`) | `x + 14 - y` (0-28) | `x` if `i < 14`, else `y` |
| `Descending` | descending diagonal (`\`) | `x + y` (0-28) | `x` if `i < 14`, else `14 - y` |

- Ascending diagonals run from the top-left (`A15`, `i = 0`) to the
  bottom-right (`O1`, `i = 28`).
- Descending diagonals run from `A1` (`i = 0`) to `O15` (`i = 28`).
- On every diagonal `j` counts from the leftmost point, so it grows with
  `x`. The two formulas differ only in where the diagonal starts: the left
  edge (`i < 14`), or the bottom (ascending) / top (descending) edge.

The triple `(Direction, i, j)` is an `Index`:

- `Point::to_index(d)` and `Index::to_point()` convert between the two.
- `Index::offset(step)` and `checked_offset(step)` move `step` cells along
  the line.
- `Index::max_j()` is the last position on the line. Diagonals get shorter
  towards the corners, so it varies by line.

## 2. `Line`: one line as bitmasks

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

- `blacks` / `whites`: bit `j` is set when position `j` holds a stone of
  that colour.
- `size`: the length. 15 for horizontal and vertical lines, 5-15 for
  diagonals (short diagonals are omitted, §4).

- `put`, `remove` and `stone(j)` are plain bit operations, which keeps the
  search loop cheap.
- `score_bound(r)` is an upper bound for skipping whole lines: 0 if five
  stones cannot fit between the opponent's, else "own stones + 1".
- `segment(j)` cuts out the segment (§3) whose five cells start at cell
  `j`; `segments()` lists them all, `j` from 0 to `size - 5`. Everything
  else a `Line` answers, such as `rows(r, kind)` (§3.1, §5), is built on
  them.

## 3. `Segment`: one place for a five

A `Segment` (`segment.rs`) is five consecutive cells of a line — one place
where a five can be made — plus the cell just before and just after. Every
rule concept is built from it.

```rust
pub struct Segment { blacks: u8, whites: u8 }
```

Each colour's stones are 7 bits, taken by `Line::segment(j)`:

```
bit  :   6   |  5    4    3    2    1  |   0
cell :  j+5  | j+4  j+3  j+2  j+1   j  |  j-1
     : margin|     the five cells      | margin
```

- Cells beyond the end of the line read as empty.
- When a segment reports positions, the five cells are numbered 0-4.

What a segment answers, for either player `r`:

| Method | Answer |
| --- | --- |
| `is_free(r)` | No opponent stone in the five cells. |
| `is_alive(r)` | `r` can still make a five here: `is_free(r)`, and for Black no black stone in either margin. |
| `count(r)` | How many of the five cells hold `r`'s stones (0-5), alive or not. |
| `score(r)` | `count(r)` if `is_alive(r)`, else `-1`. |
| `stones(r)` | The cells (0-4) holding `r`'s stones. `stone_bits(r)` is the same as a 5-bit mask. |
| `eyes(r)` | The cells (0-4) `r` still has to fill for a five here; none if not alive. `eye_bits(r)` is the same as a mask. |

The margins implement Black's rule:

- A five for Black must be *exactly* five.
- A black stone next to the five cells means filling them makes an
  overline, so the segment is dead for Black.
- White's margins do not matter: an overline is a win for White.

### 3.1 Finding rows on a line, all segments at once

`Line::rows(r, kind)` gives the segments where `r` has a row of that kind
(§5), as `(j, Segment)`.

- `RowKind::matches` states the condition segment by segment.
- The solvers ask at nearly every node, so the line checks all segments at
  once with bit operations. Bit `j` of `x >> k` is cell `j + k`, so a
  condition over shifted masks holds for every segment in parallel.

The masks:

- `count_mask(r, n)`: bit `j` is set if segment `j` is `is_free(r)` and has
  `count(r) == n`. The five cells are added up bit-sliced by `tally`
  (below).
- `score_mask(r, n)`: the same with `score(r) == n`, i.e. for Black also no
  black stone at `j - 1` or `j + 5`.
- `row_starts(r, kind)`: bit `j` is set if the row is at segment `j`.
  - `rows` walks its set bits (`Bits`, one `trailing_zeros` per row).
  - For `Two`, `Three` and `Straight` it is `open_starts(r, n)`, which
    takes any stone count `n`.
- `row_eyes(r, kind)`: the empty cells that are eyes of those rows, as a
  cell mask: where a stone takes a row one step further.
  - `eyes_of(starts, cells)` spreads start bits over the given cells of each
    segment and keeps the empty ones.
  - The VCT move ordering reads it (`ShapeMap`, 06 §8).
- A test checks `row_starts` against `RowKind::matches`, and `row_eyes`
  against the eyes of `rows`, on every line of up to nine cells and on
  random full-length lines.

Rows through one cell:

- `rows_on(i, r, kind)` keeps only the rows through cell `i`. The segment
  has `i` among its five cells; for a row of two segments the earlier one
  does too, so `i` is in the four shared cells.
- `row_starts_on(i, r, kind)` is the same as a mask. `rows_on(p, …)` uses
  it to ask "which rows does this move touch?".

<details>
<summary>How <code>tally</code> adds up the five cells of every segment at once</summary>

`count_mask` starts from `Line::tally(r)`, which returns
`([ones, twos, fours], free)`. Bit `j` of every mask is about segment `j`
(cells `j` to `j + 4`):

| Mask | Bit `j` |
| --- | --- |
| `ones` / `twos` / `fours` | The binary digits 1, 2, 4 of how many of `r`'s stones segment `j` holds (0-5). Three stones are `0b011`: set in `ones` and `twos`. |
| `free` | Segment `j` has no opponent stone and lies within the line. |

```rust
fn tally(&self, r: Player) -> ([u16; 3], u16) {
    let (my, op) = self.my_op(r);
    let segments = (1u16 << (self.size + 1 - FIVE)) - 1;
    let blocked = op | op >> 1 | op >> 2 | op >> 3 | op >> 4;
    let (a, b, c, d, e) = (my, my >> 1, my >> 2, my >> 3, my >> 4);
    let (s1, c1) = (a ^ b ^ c, a & b | c & (a ^ b));
    let (s2, c2) = (d ^ e, d & e);
    let (ones, c3) = (s1 ^ s2, s1 & s2);
    let twos = c1 ^ c2 ^ c3;
    let fours = c1 & c2 | c3 & (c1 ^ c2);
    ([ones, twos, fours], !blocked & segments)
}
```

**Lining the cells up.**

- Bit `j` of `my >> k` is cell `j + k`, so at bit `j` the values `a` to `e`
  are the five cells of segment `j`.
- Their sum is the segment's count. One `u16` holds up to 16 segments, and
  each bit operation adds for all of them at once (bit-sliced addition).

**Adding five 1-bit values.** The sum is 0-5, three binary digits, built
like an adder circuit:

1. Full adder on `a + b + c`: `s1 = a ^ b ^ c` is the ones digit;
   `c1 = a & b | c & (a ^ b)` (a majority of the three) is the carry,
   worth 2.
2. Half adder on `d + e`: `s2 = d ^ e` (ones), `c2 = d & e` (carry, worth 2).
3. The ones digits: `ones = s1 ^ s2` is final; `c3 = s1 & s2` is one more
   carry, worth 2.
4. The three carries, each worth 2: their parity `twos = c1 ^ c2 ^ c3` is
   the twos digit; their majority `fours = c1 & c2 | c3 & (c1 ^ c2)` goes
   into the fours.

That is, the sum is "ones digit + 2 × (`c1` + `c2` + `c3`)", and the second
part is added once more.

**Opponent stones and the edge.**

- `blocked = op | op >> 1 | … | op >> 4`: bit `j` is set if any of cells `j`
  to `j + 4` holds an opponent stone (the same lining up, with OR).
- `segments = (1 << (size - 4)) - 1` keeps the segments starting at 0 to
  `size - 5`. Further right the window runs off the line, where the shifts
  bring in zeros that look empty.
- `!blocked & segments` is `Segment::is_free` for every segment.

**An example.** The line `-oo-o-x--` (size 9) for Black. Masks are written
with cell 0 on the **left**, the reverse of usual binary notation:

```
cell        012345678
my          011010000   Black's stones
a (=my)     011010000
b (>>1)     110100000
c (>>2)     101000000
d (>>3)     010000000
e (>>4)     100000000

s1          000110000   ones digit of a+b+c
c1          111000000   carry of a+b+c
s2          110000000   ones digit of d+e
c2          000000000   carry of d+e
c3          000000000   carry of s1+s2

ones        110110000
twos        111000000
fours       000000000

blocked     001111100   the windows over the x at cell 6
segments    111110000   the windows starting at 0-4
free        110000000
```

| j | Cells | Stones | fours twos ones | Value | free |
| --- | --- | --- | --- | --- | --- |
| 0 | `-oo-o` | 3 | 0 1 1 | 3 | ✓ |
| 1 | `oo-o-` | 3 | 0 1 1 | 3 | ✓ |
| 2 | `o-o-x` | 2 | 0 1 0 | 2 | ✗ (x) |
| 3 | `-o-x-` | 1 | 0 0 1 | 1 | ✗ |
| 4 | `o-x--` | 1 | 0 0 1 | 1 | ✗ |

**How the result is used.**

- `select(digits, n)` picks the segments whose count is exactly `n`: AND of
  each digit mask, inverted where `n` has a 0.
- `count_mask(r, n)` is `select(digits, n) & free`.
- `score_mask(r, n)` also removes, for Black, the segments with a black
  stone just outside them (`overline`).

</details>

## 4. `Grid`: the full board

```rust
pub struct Grid {
    vlines: [Line; 15],  // vertical lines,   indexed by x
    hlines: [Line; 15],  // horizontal lines, indexed by y
    alines: [Line; 21],  // ascending  diagonals with length >= 5
    dlines: [Line; 21],  // descending diagonals with length >= 5
}
```

- A stone is stored in all four lines through its point (`put` updates each
  of them).
- Diagonals shorter than five cells can never hold a five, so the four
  shortest diagonals at each corner are omitted:
  - Diagonal `i` for `i` from 4 to 24 is stored at `alines[i - 4]`
    (`DIAGONAL_LINE_SKIP = 4`, `DIAGONAL_LINE_COUNT = 21`).
  - For the others `line_idx` returns `None`, and row queries skip them.

Main queries:

- `stone(p)`, `stones(player)`, `empty_points()`,
  `neighbors(p, distance, only_empty)`: stones and empty points.
- `rows(r, kind)`: every `Row` of kind `kind` for `r` on the board.
- `rows_on(p, r, kind)`: only the rows through `p` (§3.1; only the four
  lines through `p` are read). The hot path for "what does playing `p`
  make?".
- `line(d, i)` / `line_on(p, d)`: the stored `Line`, `None` for the short
  diagonals. `lines()` and `lines_on(p)` iterate them as
  `(Direction, i, &Line)`. A consumer with its own per-point tables can read
  these instead of keeping a second copy of all 72 lines.

Parsing (`FromStr for Grid`, reused by `Board`) accepts three formats:

- A move list, alternating from Black: `H8,H7,F6`.
- A stone list, `blacks/whites`: `H8,F6/H7`.
- A 15-line ASCII picture: `o` Black, `x` White, `.` empty, line 15 first.
  The format the tests use.

## 5. `RowKind`: the rule vocabulary

A `RowKind` is one segment, or two neighbouring ones, with the right
scores.

- Two segments are `j - 1` and `j`, spanning the six cells
  `j - 1..=j + 4`.
- `RowKind::matches(r, prev, cur)` states the condition for segment `cur`
  and the one before it, `prev`.
- A row of two segments is reported at the later one.

| `RowKind` | Segments | Pattern (Black, `_` = eye) | Rule concept |
| --- | --- | --- | --- |
| `Five` | one scoring 5 | `ooooo` | **Five**. For Black a score needs empty margins, so overlines are excluded. |
| `Overlined` | two, each `is_free` with 5 stones | `oooooo` (6+) | **Overline**. |
| `Four` | one scoring 4 | `oooo_`, `ooo_o`, `oo_oo`, … | **Four**: a stone at the eye makes a five. A straight four shows as **two** adjacent `Four`s. |
| `Straight` | two scoring 4, stones in the four shared cells | `.oooo.` | **Straight four**. |
| `Sword` | one scoring 3 | `ooo__`, `o_oo_`, … (3 stones in a segment) | A "four-to-be" (Japanese *kensaki*, "sword tip"): either eye makes a `Four`. Includes open threes, so it is not a "closed three". Not a rule term; VCF/VCT use it to list four-making moves. |
| `Three` | two scoring 3, stones in the four shared cells | `.ooo_.`, `.oo_o.`, `.o_oo.`, `._ooo.` | **Three**: the single eye makes a `Straight`. |
| `Two` | two scoring 2, stones in the four shared cells | `.oo__.`, `.o_o_.`, … | A "three-to-be": an eye makes a `Three`. |
| `Overlining` | two, each `is_free` with 4 stones | `oo_ooo`, `ooo_oo`, … | The eye makes an overline (6+). |

Notes on the table:

- Open rows (`Two`, `Three`, `Straight`): "stones in the four shared cells"
  means the later segment's last cell is empty. Both segments are alive,
  so the six cells then have both ends empty.
- Overline rows use `is_free` segments, not `is_alive` ones:
  - An overline always has a black stone next to each of its segments,
    which is exactly what makes a segment dead for Black.
  - Two `is_free` segments with four black stones each are either five
    stones in six cells (the empty one makes six) or an open four
    `.oooo.`.
  - Through an empty point only the former can be found: an open four's
    segments share only stones.

A segment is alive for Black only with empty margins, so `Four`, `Three`
etc. already include "without also making an overline". For example,
`o.oooo.`:

- `o.ooo` and `.oooo` are dead: a black stone is in their margin, so
  filling the left gap makes six.
- Only `oooo.` counts: the right end makes exactly five.

A `Row` holds where its (later) segment starts, as an `Index`, and the masks
of its stones and eyes.

- `stones()` and `eyes()` yield board `Point`s.
- For open rows only the four shared cells can be eyes: the fifth is an
  open end, not a point to play.

## 6. Forbidden moves (`forbidden.rs`)

Only Black has forbidden moves. Three entry points:

```rust
pub fn forbidden_strict(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbidden_points(g: &Grid) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` is `Overline`, `DoubleFour` or `DoubleThree`.

- `forbidden_strict` first applies the 9.2 exception, then calls
  `forbidden`. The move is *not* forbidden if:
  - `p` is occupied, or
  - playing `p` makes a five (`row_starts_on(p, Black, Four)` has a start).
- `forbidden` reads each line through `p` once, taking the starts of
  Black's `Overlining`s, `Sword`s and `Two`s together. It checks overline,
  double-four and double-three in that order, and reports the first match.
- `forbidden_points` lists every forbidden empty point.

The solvers (`Game::is_forbidden_move` in `src/mate/game.rs`) call the
non-strict `forbidden`: the search itself sees a five-making move as a win
before the forbidden check matters.

### Overline (rule 9.2 a)

```rust
fn overline(overlinings: [u16; 4]) -> bool { any(overlinings) }
```

How the three checks get their input:

- Per line through `p`, they take the mask
  `Line::row_starts_on(i, Black, kind)`: where the rows of
  `rows_on(p, Black, kind)` start (§3.1), without building them.
- `forbidden` reads all three kinds from each line in one pass
  (`Grid::row_starts_on` gives one kind).
- "Is there one?" (`any`) and "more than one?" (`has_multiple_starts`,
  below) are then tests on bits.

An `Overlining` is two adjacent segments, each with 4 black stones, both
containing the empty point `p`. They span 6 cells with 5 stones, so `p`
completes a run of six or more.

### Double-four (rule 9.2 b)

```rust
fn double_four(swords: [u16; 4]) -> bool {
    has_multiple_starts(swords)
}
```

Every `Sword` through `p` becomes a `Four` when `p` is played.

- `has_multiple_rows` returns true as soon as the iterator yields an index
  other than the first, `first`, and its neighbour `first.offset(1)`. It
  counts two adjacent segments as one.
- Why the neighbour is excluded:
  - Two `Sword`s in adjacent segments of one line are the two halves of one
    straight four (`.oo_o.` → `.oooo.`): a single four.
  - Two non-adjacent segments are a real double-four: on different lines,
    or on one line in a shape like `o.o_o.o`, which gives two fives-to-be.
- `has_multiple_starts` asks the same of the masks: a second line with a
  start, or on the first line a start other than its lowest `j` and `j + 1`
  (`s & !(0b11 << s.trailing_zeros()) != 0`).
- `has_multiple_rows` itself is still used on the `Three`s below, which
  must be checked one by one.

### Double-three (rules 9.2 c and 9.3)

```rust
fn double_three(g, p, twos: [u16; 4]) -> bool {
    // cheap pre-filter: at least two "three-to-be" rows through p
    if !has_multiple_starts(twos) { return false; }
    let mut next = g.clone();
    next.put(Black, p);
    real_double_three(&next, p)
}

fn real_double_three(next, p) -> bool {
    let real_threes = next.rows_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();   // the straight-four point
        forbidden_strict(next, eye).is_none()
    });
    has_multiple_rows(&mut real_threes.map(|s| s.start_index()))
}
```

Steps:

1. Two or more `Two`s through `p` are necessary for a double-three, so this
   is checked first. Most points stop here, without cloning the board.
2. Play the move on a copy and list the `Three`s through `p`. A `Three` has
   exactly one eye: the straight-four point.
3. Rule 9.3: a three counts only if that eye is a legal Black move, decided
   by `forbidden_strict` on the eye in the new position.
4. Two or more *distinct* real threes make a forbidden double-three.

Notes on step 3:

- `forbidden_strict` on the eye covers both 9.3 a (the straight-four move
  is an overline or double-four) and 9.3 b (it is a forbidden
  double-three).
- The recursion `forbidden_strict → forbidden → double_three →
  real_double_three → forbidden_strict → …` handles the rule's "and so on"
  nesting.
- The `_strict` variant matters: if the eye also makes a five on another
  line, the move is legal (and wins) under 9.2 even as a double-four, so
  the three is real (`test_double_three_eye_makes_five`).
- Being recursive, the five can also be one the candidate move itself
  creates; the answer for the candidate then flips
  (`test_double_three_nested_eye_makes_five`).
- RIF 9.3 a) literally says "without … an overline or double-four is
  attained" and does not restate the 9.2 five exception. This code reads it
  as "without making a *forbidden* move", consistent with 9.2 and with the
  Japanese definition of a three.

Example from the tests (`test_double_three`): playing `H8` here.

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

- Vertical: `.o_o.` on line H is a `Two` through `H8`. It is found at two
  adjacent segments, so `has_multiple_rows` counts it once.
- Horizontal: `x.o_o.x` is capped by the `x`s. No pair of segments fits,
  so it is no `Two`: it could never become a straight four.
- So the pre-filter (step 1) rejects the move without cloning, and the
  result is `None`.
- Without the two `x`s, both directions have a `Two`. After `H8` both
  become `Three`s with legal straight-four points: `Some(DoubleThree)`.

More cases, including the nested "fake three" positions from the referenced
Twitter thread, are in `forbidden.rs`'s tests.

## 7. Zobrist hashing (`zobrist.rs`) and `Board`

`Board` wraps a `Grid` and a `u64` Zobrist hash; `put` / `remove` keep the
two in sync.

- `CODE_TABLE` holds `2 * 225` random 64-bit codes, indexed by
  `2 * u8::from(point) + c` (`c` = 0 for Black, 1 for White).
- Placing or removing a stone XORs its code into or out of the hash.
- The turn, the search's attacker and the remaining depth are not
  properties of the board, so the hash leaves them out. The solvers XOR
  them in to key their transposition tables (`apply_turn`,
  `apply_attacker`, `apply_limit` with `LIMIT_TABLE`; `State::key`, 04).
- `Board::put` / `remove` change the board in place. `with_stone` /
  `without_stone` return changed copies; the solvers avoid them so as not
  to clone in the search loop.

The VCF search asks for each player's swords (`Sword`, §5) at nearly every
node, so it caches them in a `SwordMap` (`src/feature/sword.rs`).

- The search states hold it, not `Board` (like the VCT's `ShapeMap`):
  `VCFState`, and `VCTState` to hand to its nested VCFs. They mark it from
  `State::after_play` / `after_undo`, and pass the board when they sync or
  read it.
- Contents, per player and per line (by `Grid::line_key`, the line's
  position in `Grid::lines`): a `u16` with bit `j` set if a sword's segment
  starts at cell `j`, and a `u128` of the lines that have any.
- A move only marks the (at most four) lines through its point stale
  (`SwordMap::mark_stale`, in a `StaleLines` from `src/feature/stale.rs`,
  shared with `ShapeMap`). `SwordMap::sync` recomputes them.
  - The searches move far more often than they read, so recomputing at
    every move would cost more than the scan it replaces.
- A line is recomputed by `Line::row_starts(r, Sword)`, all segments at
  once (§3.1).
- `SwordMap::swords(board, r)` / `swords_on(board, p, r)` return what
  `rows(r, Sword)` / `rows_on(p, r, Sword)` would, in the same order. They
  need `sync(board)` first.

## 8. Cheat sheet: rule → code

| Rule | Code |
| --- | --- |
| Five wins | `rows(r, Five)` (checked in `mate::solve` / `Game`). |
| Overline wins for White, not Black | `Five` is exact only for Black, so a White six is still a `Five`. A Black overline is a forbidden move (`Overlining`). `mate::solve::trivial_result` answers input positions that already contain a five or a Black `Overlined`. |
| Four / straight four | `Four` (a segment scoring 4) / `Straight` (two scoring 4); a straight four = two adjacent `Four`s. |
| Three (must reach a straight four) | `Three` (two segments scoring 3); its single eye is the straight-four point. |
| "Without making an overline" for Black | `Segment::is_alive`: no black stone in the margins. |
| Forbidden: overline / double-four / double-three | `forbidden.rs`: `overline` / `double_four` / `double_three`. |
| 9.2 "unless it makes a five" | `forbidden_strict`. |
| 9.3 real vs. fake threes, recursive | `real_double_three` calling `forbidden_strict` on each three's eye. |
