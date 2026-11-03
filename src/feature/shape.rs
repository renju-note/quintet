use super::stale::StaleLines;
use crate::board::Player::*;
use crate::board::RowKind::*;
use crate::board::*;

/// What a stone of one player at an empty point would make on one line
/// through it, from nothing to a five, as the rules tell it.
///
/// The move ordering weighs a shape by its [`Self::rank`], how far it is
/// from a five, and so does everything in [`Shapes`]: the three kinds of
/// four rank alike, and a dead cell and Black's overline rank with
/// [`Self::Nothing`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Shape {
    /// No five can ever pass through the point this way: every five cells
    /// through it hold an opponent stone or run off the line.
    Dead,
    #[default]
    Nothing,
    /// Two stones that can become a three: the point is within three cells
    /// of an own stone, on an open stretch of line.
    Two,
    /// A [`RowKind::Sword`], three stones that can become a four: the point
    /// is in a segment already holding two.
    Sword,
    /// A [`RowKind::Three`]: the point is an eye of a [`RowKind::Two`].
    Three,
    /// A [`RowKind::Four`], one five point: the point is an eye of a
    /// [`RowKind::Sword`], and the other eyes of the swords it is in are
    /// all one cell.
    Four,
    /// Two or more five points on the line that are not an open four,
    /// e.g. `o.ooo.o`: the swords the point is in have other eyes in
    /// different cells.
    DoubleFour,
    /// A [`RowKind::Straight`], an open four: the point is an eye of a
    /// [`RowKind::Three`].
    Straight,
    /// Black only: six or more in a row. The point is the eye of a
    /// [`RowKind::Overlining`], or next to a five.
    Overline,
    /// A five: the point is the eye of a [`RowKind::Four`].
    Five,
}

impl Shape {
    /// How far the shape is from a five, as the move ordering counts it:
    /// 1 for a two, 2 for a sword, 3 for a three, 4 for a four of any kind
    /// and 5 for a five; 0 for nothing, a dead cell and an overline, which
    /// make nothing that can become a five.
    #[inline]
    pub const fn rank(self) -> u8 {
        match self {
            Shape::Dead | Shape::Nothing | Shape::Overline => 0,
            Shape::Two => 1,
            Shape::Sword => 2,
            Shape::Three => 3,
            Shape::Four | Shape::DoubleFour | Shape::Straight => 4,
            Shape::Five => 5,
        }
    }

    /// What `r` would make at cell `j` of `line`, which must be empty: the
    /// strongest shape that applies. The same as [`ShapeMap`] keeps for
    /// the point.
    pub fn at(line: &Line, j: u8, r: Player) -> Self {
        debug_assert!(line.stone(j).is_none());
        line_shapes(line, r)[j as usize]
    }
}

/// A point's [`Shape`]s for one player, one per direction.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Shapes([Shape; 4]);

impl Shapes {
    /// The shape made along `d`.
    pub fn along(&self, d: Direction) -> Shape {
        self.0[d as usize]
    }

    /// How many directions make a shape of the same [`Shape::rank`] as
    /// `s`: `count(Shape::Four)` counts fours of every kind.
    pub fn count(&self, s: Shape) -> u8 {
        self.0.iter().filter(|x| x.rank() == s.rank()).count() as u8
    }

    /// The ranks of the four directions summed ([`Shape::rank`]).
    pub fn total(&self) -> u8 {
        self.0.iter().map(|s| s.rank()).sum()
    }

    /// The shape of the highest rank made along any direction (of those of
    /// one rank, the strongest).
    pub fn best(&self) -> Shape {
        self.0
            .iter()
            .copied()
            .max_by_key(|&s| (s.rank(), s))
            .unwrap_or_default()
    }

    /// How many directions make a shape of at least the rank of `s`.
    pub fn count_at_least(&self, s: Shape) -> u8 {
        self.0.iter().filter(|x| x.rank() >= s.rank()).count() as u8
    }

    /// The shapes with direction `d` left out.
    pub fn except(&self, d: Direction) -> Self {
        let mut result = *self;
        result.0[d as usize] = Shape::Nothing;
        result
    }

    /// For Black, whether the point looks forbidden: two fours or two
    /// threes and no five, counted by direction ([`Self::count`]). Only a
    /// guess. It does not count a [`Shape::DoubleFour`] as two fours nor an
    /// [`Shape::Overline`] at all, and a three here may be no real three
    /// because its straight-four point is itself forbidden (see
    /// `board/forbidden.rs`).
    pub fn looks_forbidden(&self) -> bool {
        self.count(Shape::Five) == 0
            && (self.count(Shape::Four) >= 2 || self.count(Shape::Three) >= 2)
    }
}

/// Per player and per empty point, what a stone there would make on each
/// of the four lines through it ([`Shapes`]), kept alongside a [`Board`]
/// for move ordering.
///
/// Updates are lazy in the same way as [`SwordMap`](super::sword::SwordMap):
/// [`Self::mark_stale`] notes the lines through a move, and [`Self::sync`]
/// recomputes them before the next read.
#[derive(Clone)]
pub struct ShapeMap {
    /// Black's and White's, in that order, by point code (`u8::from`).
    shapes: [[Shapes; POINT_COUNT]; 2],
    /// Lines changed since the last [`Self::sync`].
    stale: StaleLines,
}

impl ShapeMap {
    /// A map that knows nothing yet: every line is stale.
    pub fn new() -> Self {
        Self {
            shapes: [[Shapes::default(); POINT_COUNT]; 2],
            stale: StaleLines::all(),
        }
    }

    /// A map in sync with `board`.
    pub fn from_board(board: &Board) -> Self {
        let mut result = Self::new();
        result.sync(board);
        result
    }

    /// Notes that a stone was put on or taken off `p`.
    pub fn mark_stale(&mut self, p: Point) {
        self.stale.mark(p);
    }

    pub fn is_synced(&self) -> bool {
        self.stale.is_empty()
    }

    /// Recomputes the stale lines from `board`.
    pub fn sync(&mut self, board: &Board) {
        for k in self.stale.take() {
            self.update(k as usize, board);
        }
    }

    /// What `r` would make at `p`. The map must be in sync with the board;
    /// an occupied point makes nothing.
    pub fn get(&self, p: Point, r: Player) -> Shapes {
        debug_assert!(self.is_synced());
        self.shapes[r.index()][u8::from(p) as usize]
    }

    /// Of the fours and threes `r` would make at `p`, how many leave an
    /// eye where Black's stone looks forbidden, as `(fours, threes)`.
    ///
    /// For White that is good: Black cannot block such a four, and has
    /// fewer ways to stop such a three. For Black it is bad: the three
    /// cannot become a straight four there, which also means it may be no
    /// real three. (Black's four is never counted: its eye makes a five.)
    /// The map must be in sync with `board`.
    pub fn forbidden_eyes(&self, board: &Board, p: Point, r: Player) -> (u8, u8) {
        debug_assert!(self.is_synced());
        let shapes = self.get(p, r);
        let count = |made: Shape, k: RowKind| {
            if shapes.count(made) == 0 {
                return 0;
            }
            board
                .rows_on(p, r, k)
                .filter(|row| {
                    let d = row.start_index().d;
                    // `p` is on no other line through the eye than `d`,
                    // so Black's shapes along the others stay as they are.
                    shapes.along(d).rank() == made.rank()
                        && row.eyes().any(|e| {
                            let others = self.get(e, Black).except(d);
                            e != p
                                && if r.is_white() {
                                    others.looks_forbidden()
                                } else {
                                    // A straight four along `d`, and a
                                    // four or two threes on the others.
                                    others.count(Shape::Five) == 0
                                        && (others.count(Shape::Four) >= 1
                                            || others.count(Shape::Three) >= 2)
                                }
                        })
                })
                .count() as u8
        };
        if r.is_white() {
            (count(Shape::Four, Sword), count(Shape::Three, Two))
        } else {
            (0, count(Shape::Three, Two))
        }
    }

    fn update(&mut self, k: usize, board: &Board) {
        let (d, i) = Grid::line_from_key(k);
        let line = board.line(d, i).unwrap();
        let (start, step) = cell_codes(d, i);
        for r in [Black, White] {
            let cells = line_shapes(line, r);
            let shapes = &mut self.shapes[r.index()];
            for (j, &s) in cells[..line.size as usize].iter().enumerate() {
                shapes[start + j * step].0[d as usize] = s;
            }
        }
    }
}

/// What `r` would make at each cell of `line`: [`Shape::Nothing`] for the
/// occupied ones, and beyond the line's end.
fn line_shapes(line: &Line, r: Player) -> [Shape; SIZE as usize] {
    let mut cells = [Shape::Nothing; SIZE as usize];
    let empty = line.empty_cells().0;
    for j in Bits(dead_cells(line, r) & empty) {
        cells[j as usize] = Shape::Dead;
    }
    // Lowest first, so that a cell gets the strongest shape it makes.
    let (fours, double_fours) = four_cells(line, r);
    let masks = [
        (
            Shape::Two,
            line.eyes_of(line.open_starts(r, 1), Two.eye_cells()),
        ),
        (
            Shape::Sword,
            line.eyes_of(line.score_mask(r, 2), Sword.eye_cells()),
        ),
        (Shape::Three, line.row_eyes(r, Two)),
        (Shape::Four, fours),
        (Shape::DoubleFour, double_fours),
        (Shape::Straight, line.row_eyes(r, Three)),
        (Shape::Overline, overline_cells(line, r)),
        (Shape::Five, line.row_eyes(r, Four)),
    ];
    for (s, mask) in masks {
        for j in Bits(mask) {
            cells[j as usize] = s;
        }
    }
    cells
}

/// The eyes of `r`'s swords on `line`, as [`Line::row_eyes`] of
/// [`Sword`], split by how many five points a stone there makes: one
/// (the first mask) or more (the second). A stone on one eye of a sword
/// makes the other its five point.
fn four_cells(line: &Line, r: Player) -> (u16, u16) {
    let mut partners = [0u16; SIZE as usize];
    let mut eyes = 0;
    for s in Bits(line.row_starts(r, Sword)) {
        let pair = line.eyes_of(1 << s, Sword.eye_cells());
        eyes |= pair;
        let (a, b) = (pair.trailing_zeros(), 15 - pair.leading_zeros());
        partners[a as usize] |= 1 << b;
        partners[b as usize] |= 1 << a;
    }
    let mut double = 0;
    for j in Bits(eyes) {
        if partners[j as usize].count_ones() >= 2 {
            double |= 1 << j;
        }
    }
    (eyes & !double, double)
}

/// The cells where Black's stone would make six or more in a row: the eyes
/// of [`RowKind::Overlining`], and the cells next to a five. Empty for
/// White, whose six is a five.
fn overline_cells(line: &Line, r: Player) -> u16 {
    if r.is_white() {
        return 0;
    }
    let fives = line.count_mask(Black, FIVE);
    let beside = fives >> 1 | fives << FIVE;
    (line.row_eyes(Black, Overlining) | beside) & line.empty_cells().0
}

/// The cells that no five cells free of `r`'s opponent pass through.
fn dead_cells(line: &Line, r: Player) -> u16 {
    let all = ((1u32 << line.size) - 1) as u16;
    if line.size < FIVE {
        return all;
    }
    let op = line.stones(r.opponent()).0;
    let starts = ((1u32 << (line.size + 1 - FIVE)) - 1) as u16;
    let free = !(op | op >> 1 | op >> 2 | op >> 3 | op >> 4) & starts;
    let covered = free | free << 1 | free << 2 | free << 3 | free << 4;
    all & !covered
}

/// Cell `j` of line `i` in direction `d` is the point whose code
/// (`u8::from`) is `start + j * step`: a code is `x * SIZE + y`, and a step
/// along a line moves `x` and `y` by a fixed amount. Cheaper than
/// [`Index::to_point`] for every cell.
fn cell_codes(d: Direction, i: u8) -> (usize, usize) {
    let start = u8::from(Index::new(d, i, 0).to_point()) as usize;
    let step = match d {
        Direction::Vertical => 1,
        Direction::Horizontal => SIZE as usize,
        Direction::Ascending => SIZE as usize + 1,
        Direction::Descending => SIZE as usize - 1,
    };
    (start, step)
}

impl Default for ShapeMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Shape::*;
    use super::{Shape, ShapeMap, cell_codes};
    use crate::board::Direction::Vertical;
    use crate::board::Player::{self, *};
    use crate::board::RowKind;
    use crate::board::{Bits, Board, Grid, Index, LINE_COUNT, Line, Points};

    fn shapes(map: &ShapeMap, p: &str, r: Player) -> [Shape; 4] {
        map.get(p.parse().unwrap(), r).0
    }

    #[test]
    fn test_cell_codes_match_the_points() {
        let board = Board::new();
        for k in 0..LINE_COUNT {
            let (d, i) = Grid::line_from_key(k);
            let (start, step) = cell_codes(d, i);
            for j in 0..board.line(d, i).unwrap().size {
                let p = Index::new(d, i, j).to_point();
                assert_eq!(
                    start + j as usize * step,
                    u8::from(p) as usize,
                    "{d:?} {i} {j}"
                );
            }
        }
    }

    #[test]
    fn test_shapes() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o o . o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . x x x . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let map = ShapeMap::from_board(&board);
        // [Vertical, Horizontal, Ascending, Descending]
        // F8-I8 open at both ends: an open four.
        assert_eq!(
            shapes(&map, "H8", Black),
            [Nothing, Straight, Nothing, Nothing]
        );
        assert_eq!(shapes(&map, "E8", Black), [Nothing, Four, Nothing, Nothing]);
        assert_eq!(shapes(&map, "H9", Black), [Nothing, Nothing, Two, Two]);
        assert_eq!(shapes(&map, "G9", Black), [Two, Nothing, Two, Nothing]);
        // D8-H8 and H8-L8 are dead for Black, next to I8 and G8: filling
        // them would make an overline. C8-G8 holds two, but alone.
        assert_eq!(
            shapes(&map, "D8", Black),
            [Nothing, Sword, Nothing, Nothing]
        );
        assert_eq!(shapes(&map, "K8", Black), [Nothing; 4]);
        // White's three, both ends open: next to it an open four, one
        // cell away a four.
        assert_eq!(
            shapes(&map, "E3", White),
            [Nothing, Straight, Nothing, Nothing]
        );
        assert_eq!(shapes(&map, "J3", White), [Nothing, Four, Nothing, Nothing]);
        assert_eq!(
            shapes(&map, "C3", White),
            [Nothing, Sword, Nothing, Nothing]
        );
        assert_eq!(shapes(&map, "G4", White), [Two, Nothing, Two, Two]);
        // Occupied points make nothing.
        assert_eq!(shapes(&map, "F8", Black), [Nothing; 4]);
        Ok(())
    }

    #[test]
    fn test_looks_forbidden() -> Result<(), String> {
        // H8 is a double-three for Black, and one of White's four eyes.
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . x . o . . . . .
         . . . . . . . . o . . . . . .
         . . . . . o o . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let map = ShapeMap::from_board(&board);
        let h8 = "H8".parse()?;
        assert!(board.forbidden(h8).is_some());
        assert!(map.get(h8, Black).looks_forbidden());
        assert!(!map.get("E8".parse()?, Black).looks_forbidden());
        // White's H9 makes two fours, H8-H12 and H9-H13: Black cannot
        // block the first at H8.
        let h9 = "H9".parse()?;
        assert_eq!(map.forbidden_eyes(&board, h9, White), (1, 0));
        assert_eq!(map.forbidden_eyes(&board, h9, Black), (0, 0));
        // White's H9 is on no other line through H8 than the vertical one,
        // so Black's two threes stay.
        assert!(map.get(h8, Black).except(Vertical).looks_forbidden());
        Ok(())
    }

    /// The map has to say what a fresh one says after any run of moves.
    #[test]
    fn test_lazy_matches_a_fresh_one() -> Result<(), String> {
        let mut board = "H8,I9,J9,H7".parse::<Board>()?;
        let mut map = ShapeMap::from_board(&board);
        let moves = "G7,K9,H9,F6,G8,I7".parse::<Points>()?.into_vec();
        let mut turn = Black;
        for (n, &p) in moves.iter().enumerate() {
            board.put(turn, p);
            map.mark_stale(p);
            if n % 2 == 1 {
                map.sync(&board);
                assert!(map.shapes == ShapeMap::from_board(&board).shapes);
            }
            turn = turn.opponent();
        }
        for &p in moves.iter().rev() {
            board.remove(p);
            map.mark_stale(p);
        }
        map.sync(&board);
        assert!(map.shapes == ShapeMap::from_board(&board).shapes);
        Ok(())
    }

    /// [`Shape::at`] at every empty cell of a line given as text, `o`
    /// Black and `x` White, one character each: `-` dead, `.` nothing,
    /// `2` two, `s` sword, `3` three, `4` four, `d` double four, `S`
    /// open four, `L` overline, `5` five; stones as they are.
    fn line_shapes(line: &str, r: Player) -> String {
        let line: Line = line.parse().unwrap();
        (0..line.size)
            .map(|j| match line.stone(j) {
                Some(p) => char::from(p),
                None => match Shape::at(&line, j, r) {
                    Dead => '-',
                    Nothing => '.',
                    Two => '2',
                    Sword => 's',
                    Three => '3',
                    Four => '4',
                    DoubleFour => 'd',
                    Straight => 'S',
                    Overline => 'L',
                    Five => '5',
                },
            })
            .collect()
    }

    #[test]
    fn test_shape_at_threes_and_fours() {
        // Black's open three: open fours inside, fours outside, and the
        // cells beyond would make an overline later.
        assert_eq!(line_shapes("......ooo......", Black), "....4SoooS4....");
        assert_eq!(line_shapes(".......o.......", Black), "....222o222....");
        // White's has no overline to fear.
        assert_eq!(line_shapes("....x.xxx.x....", White), ".2s4x5xxx5x4s2.");
        assert_eq!(line_shapes("....x.x.x.x....", White), ".2s3x4xdx4x3s2.");
        // Four by five points: one from `oooo` next to an overline point,
        // two from `o.ooo.o`.
        assert_eq!(line_shapes("...ooo.oo......", Black), ".44oooLoosss...");
        assert_eq!(line_shapes("....o.o.o.o....", Black), "..sso4odo4oss..");
        // `xxxx.x` and the cell after it: one five point.
        assert_eq!(line_shapes(".xxxx.x........", White), "5xxxx5x4s2.....");
    }

    #[test]
    fn test_shape_at_dead_and_overline() {
        // No five cells free of White through these.
        assert_eq!(line_shapes("x....x.oo.x....", Black), "x----x-oo-x----");
        // A six with the new stone at either end of it; White's six is a
        // five.
        assert_eq!(line_shapes("..ooooo........", Black), ".LoooooL.......");
        assert_eq!(line_shapes("..xxxxx........", White), "45xxxxx54s.....");
        assert_eq!(line_shapes("ooooo..........", Black), "oooooL.........");
        // Too short for a five.
        assert_eq!(line_shapes("..o.", Black), "--o-");
    }

    /// [`Shape::at`] against the rules read directly: put the stone and
    /// look at the rows through it, strongest first. Fours are counted by
    /// their five points. A six is any [`RowKind::Overlined`] row holding the cell.
    fn by_rows(line: &Line, j: u8, r: Player) -> Shape {
        let mut after = *line;
        after.put(r, j);
        let has = |k: RowKind| after.row_starts_on(j, r, k) != 0;
        if has(RowKind::Five) {
            return Five;
        }
        let first = j.saturating_sub(4);
        let last = (j + 1).min(after.size.saturating_sub(5));
        let around = if first > last {
            0
        } else {
            ((1u32 << (last + 1)) - (1u32 << first)) as u16
        };
        if r.is_black() && after.row_starts(r, RowKind::Overlined) & around != 0 {
            return Overline;
        }
        if has(RowKind::Straight) {
            return Straight;
        }
        let fives = after.eyes_of(
            after.row_starts_on(j, r, RowKind::Four),
            RowKind::Four.eye_cells(),
        );
        match fives.count_ones() {
            0 => {}
            1 => return Four,
            _ => return DoubleFour,
        }
        if has(RowKind::Three) {
            return Three;
        }
        if has(RowKind::Sword) {
            return Sword;
        }
        if has(RowKind::Two) {
            return Two;
        }
        let op = line.stones(r.opponent()).0;
        let dead = line.size < 5
            || (j.saturating_sub(4)..=j.min(line.size - 5)).all(|s| op >> s & 0b11111 != 0);
        if dead { Dead } else { Nothing }
    }

    /// What the map kept before it told the kinds of four, dead cells and
    /// overlines apart: the move ordering must see the same ranks.
    fn coarse(line: &Line, r: Player) -> [u8; 15] {
        let masks = [
            (
                1,
                line.eyes_of(line.open_starts(r, 1), RowKind::Two.eye_cells()),
            ),
            (
                2,
                line.eyes_of(line.score_mask(r, 2), RowKind::Sword.eye_cells()),
            ),
            (3, line.row_eyes(r, RowKind::Two)),
            (4, line.row_eyes(r, RowKind::Sword)),
            (5, line.row_eyes(r, RowKind::Four)),
        ];
        let mut cells = [0; 15];
        for (rank, mask) in masks {
            for j in Bits(mask) {
                cells[j as usize] = rank;
            }
        }
        cells
    }

    /// Random lines of every length, sparse to crowded: [`Shape::at`]
    /// agrees with the rows, and its rank with the old map's.
    #[test]
    fn test_shape_at_matches_the_rows_and_the_old_ranks() {
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for n in 0..20_000 {
            let size = 1 + (n % 15) as u8;
            let density = 1 + n % 7;
            let mut line = Line::new(size);
            for i in 0..size {
                let v = next() % 10;
                if v < density as u64 {
                    line.put(if next() % 2 == 0 { Black } else { White }, i);
                }
            }
            for r in [Black, White] {
                let old = coarse(&line, r);
                for j in Bits(line.empty_cells().0) {
                    let shape = Shape::at(&line, j, r);
                    assert_eq!(shape, by_rows(&line, j, r), "{line} {r:?} {j}");
                    assert_eq!(shape.rank(), old[j as usize], "{line} {r:?} {j} {shape:?}");
                }
            }
        }
    }
}
