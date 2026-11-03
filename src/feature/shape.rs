use super::stale::StaleLines;
use crate::board::Player::*;
use crate::board::RowKind::*;
use crate::board::*;

/// What a stone of one player at an empty point would make on one line
/// through it, from nothing to a five.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Shape {
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
    /// A [`RowKind::Four`]: the point is an eye of a [`RowKind::Sword`].
    Four,
    /// A five: the point is the eye of a [`RowKind::Four`].
    Five,
}

/// What a stone of one player at an empty cell of a line would make on
/// that line, exactly, weakest first.
///
/// [`Shape`] is what the VCT move ordering needs, read off the rows
/// before the stone is put. This is finer, for a caller that wants the
/// rules' answer on one line: it tells an open four from two fours on the
/// line and from one, Black's overline from what Black's stones make, and a
/// cell where no five can ever be made from open space. It is asked one
/// cell at a time ([`Self::at`]); nothing is kept.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum LineShape {
    /// No five can ever pass through the cell this way: every five cells
    /// through it hold an opponent stone or run off the line.
    Dead,
    #[default]
    Nothing,
    /// A [`RowKind::Two`] through the cell.
    Two,
    /// A [`RowKind::Sword`] through the cell, and no three.
    Sword,
    /// A [`RowKind::Three`] through the cell.
    Three,
    /// One five point on the line ([`RowKind::Four`]).
    Four,
    /// Two or more five points on the line that are not an open four,
    /// e.g. `o.ooo.o`.
    DoubleFour,
    /// A [`RowKind::Straight`]: an open four.
    Straight,
    /// Black only: six or more in a row ([`RowKind::Overlined`]).
    Overline,
    /// A [`RowKind::Five`].
    Five,
}

impl LineShape {
    /// What `r` would make at cell `j` of `line`, which must be empty: the
    /// strongest of the rows through `j` once the stone is put.
    ///
    /// Fours are counted by their five points, not their rows: White's
    /// `xxxx.x` has two rows with the one five point.
    pub fn at(line: &Line, j: u8, r: Player) -> Self {
        debug_assert!(line.stone(j).is_none());
        let mut after = *line;
        after.put(r, j);
        let has = |k: RowKind| after.row_starts_on(j, r, k) != 0;
        if has(Five) {
            return LineShape::Five;
        }
        // Any six through `j`, also with `j` at one of its ends, where
        // `row_starts_on` does not look.
        let overline = after.row_starts(r, Overlined) & starts_around(j, after.size);
        if r.is_black() && overline != 0 {
            return LineShape::Overline;
        }
        if has(Straight) {
            return LineShape::Straight;
        }
        let fives = after.eyes_of(after.row_starts_on(j, r, Four), Four.eye_cells());
        match fives.count_ones() {
            0 => {}
            1 => return LineShape::Four,
            _ => return LineShape::DoubleFour,
        }
        if has(Three) {
            LineShape::Three
        } else if has(Sword) {
            LineShape::Sword
        } else if has(Two) {
            LineShape::Two
        } else if is_dead(line, j, r) {
            LineShape::Dead
        } else {
            LineShape::Nothing
        }
    }
}

/// The starts of the two-segment rows that have cell `j` in either
/// segment: the segments starting at `j - 4` to `j + 1`.
fn starts_around(j: u8, size: u8) -> u16 {
    let first = j.saturating_sub(4);
    let last = (j + 1).min(size.saturating_sub(FIVE));
    if first > last {
        return 0;
    }
    ((1u32 << (last + 1)) - (1u32 << first)) as u16
}

/// Whether every five cells of `line` through `j` hold a stone of `r`'s
/// opponent, or there are none.
fn is_dead(line: &Line, j: u8, r: Player) -> bool {
    if line.size < FIVE {
        return true;
    }
    let op = line.stones(r.opponent()).fold(0u16, |m, i| m | 1 << i);
    (j.saturating_sub(4)..=j.min(line.size - FIVE)).all(|start| op >> start & 0b11111 != 0)
}

/// A point's [`Shape`]s for one player, one per direction.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Shapes([Shape; 4]);

impl Shapes {
    /// The shape made along `d`.
    pub fn along(&self, d: Direction) -> Shape {
        self.0[d as usize]
    }

    /// How many directions make exactly `s`.
    pub fn count(&self, s: Shape) -> u8 {
        self.0.iter().filter(|&&x| x == s).count() as u8
    }

    /// The shapes of the four directions summed, each by its rank: a two
    /// counts 1, a sword 2, a three 3, a four 4 and a five 5.
    pub fn total(&self) -> u8 {
        self.0.iter().map(|&s| s as u8).sum()
    }

    /// The biggest shape made along any direction.
    pub fn best(&self) -> Shape {
        self.0.iter().copied().max().unwrap_or_default()
    }

    /// How many directions make at least `s`.
    pub fn count_at_least(&self, s: Shape) -> u8 {
        self.0.iter().filter(|&&x| x >= s).count() as u8
    }

    /// The shapes with direction `d` left out.
    pub fn except(&self, d: Direction) -> Self {
        let mut result = *self;
        result.0[d as usize] = Shape::Nothing;
        result
    }

    /// For Black, whether the point looks forbidden: two fours or two
    /// threes and no five. Only a guess. It does not see a double-four
    /// on one line nor an overline, and a three here may be no real three
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
                    shapes.along(d) == made
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
            // Lowest first, so that a cell gets the biggest shape it makes.
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
                (Shape::Four, line.row_eyes(r, Sword)),
                (Shape::Five, line.row_eyes(r, Four)),
            ];
            let mut cells = [Shape::Nothing; SIZE as usize];
            for (s, mask) in masks {
                for j in Bits(mask) {
                    cells[j as usize] = s;
                }
            }
            let shapes = &mut self.shapes[r.index()];
            for (j, &s) in cells[..line.size as usize].iter().enumerate() {
                shapes[start + j * step].0[d as usize] = s;
            }
        }
    }
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
    use super::{LineShape, Shape, ShapeMap, cell_codes};
    use crate::board::Direction::Vertical;
    use crate::board::Player::{self, *};
    use crate::board::{Board, Grid, Index, LINE_COUNT, Line, Points};

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
        assert_eq!(shapes(&map, "H8", Black), [Nothing, Four, Nothing, Nothing]);
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
        // White's three, both ends open.
        assert_eq!(shapes(&map, "E3", White), [Nothing, Four, Nothing, Nothing]);
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

    /// `LineShape::at` on one line given as text, `o` Black and `x` White,
    /// at every empty cell, as one character each: `-` dead, `.` nothing,
    /// `2` two, `s` sword, `3` three, `4` four, `d` double four, `S`
    /// straight, `L` overline, `5` five; stones as they are.
    fn line_shapes(line: &str, r: Player) -> String {
        let line: Line = line.parse().unwrap();
        (0..line.size)
            .map(|j| match line.stone(j) {
                Some(p) => char::from(p),
                None => match LineShape::at(&line, j, r) {
                    LineShape::Dead => '-',
                    LineShape::Nothing => '.',
                    LineShape::Two => '2',
                    LineShape::Sword => 's',
                    LineShape::Three => '3',
                    LineShape::Four => '4',
                    LineShape::DoubleFour => 'd',
                    LineShape::Straight => 'S',
                    LineShape::Overline => 'L',
                    LineShape::Five => '5',
                },
            })
            .collect()
    }

    #[test]
    fn test_line_shape_threes_and_fours() {
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
    fn test_line_shape_dead_and_overline() {
        // No five cells free of White through these.
        assert_eq!(line_shapes("x....x.oo.x....", Black), "x----x-oo-x----");
        // A six with the new stone at either end, which a row through it
        // in both segments does not cover; White's six is a five.
        assert_eq!(line_shapes("..ooooo........", Black), ".LoooooL.......");
        assert_eq!(line_shapes("..xxxxx........", White), "45xxxxx54s.....");
        assert_eq!(line_shapes("ooooo..........", Black), "oooooL.........");
    }

    /// `LineShape` and `Shape` read the same rows: where `LineShape` sees
    /// a four or more, `Shape` sees a four or more, and the other way
    /// round; below that, they agree on twos, swords and threes. Black's
    /// overline is left out: `Shape` reads the rows before the stone.
    #[test]
    fn test_line_shape_agrees_with_shape_map() -> Result<(), String> {
        for moves in [
            "H8,I9,J9,H7,G7,K9,H9,F6,G8,I7,J7,G9,F8",
            "H8,H9,I8,G8,J8,I9,J9,K10,G7,F6,I10,H10,J11",
            "A1,O15,B2,N14,C3,M13,A15,O1,B14,N2,H8,H7,I9",
        ] {
            agree_on(&moves.parse::<Board>()?);
        }
        Ok(())
    }

    fn agree_on(board: &Board) {
        let map = ShapeMap::from_board(board);
        for p in board.empty_points() {
            for r in [Black, White] {
                let shapes = map.get(p, r);
                for (d, _, line) in board.lines_on(p) {
                    let shape = shapes.along(d);
                    let fine = LineShape::at(line, p.to_index(d).j, r);
                    let want = match fine {
                        LineShape::Dead | LineShape::Nothing => Nothing,
                        LineShape::Two => Two,
                        LineShape::Sword => Sword,
                        LineShape::Three => Three,
                        LineShape::Four | LineShape::DoubleFour | LineShape::Straight => Four,
                        LineShape::Overline => continue,
                        LineShape::Five => Five,
                    };
                    assert_eq!(shape, want, "{p} {r:?} {d:?} {fine:?}");
                }
            }
        }
    }
}
