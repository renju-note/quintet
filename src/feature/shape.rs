use crate::board::Direction::*;
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

    /// How many directions make at least `s`.
    pub fn count_from(&self, s: Shape) -> u8 {
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
    /// Black's and White's, in that order, by point `x * RANGE + y`.
    shapes: [[Shapes; POINTS]; 2],
    /// Lines changed since the last [`Self::sync`], one bit per line.
    stale: u128,
}

const POINTS: usize = RANGE as usize * RANGE as usize;

impl ShapeMap {
    /// A map that knows nothing yet: every line is stale.
    pub fn new() -> Self {
        Self {
            shapes: [[Shapes::default(); POINTS]; 2],
            stale: (1 << LINE_NUM) - 1,
        }
    }

    /// A map in sync with `board`.
    pub fn init(board: &Board) -> Self {
        let mut result = Self::new();
        result.sync(board);
        result
    }

    /// Notes that a stone was put on or taken off `p`.
    pub fn mark_stale(&mut self, p: Point) {
        for d in [Vertical, Horizontal, Ascending, Descending] {
            if let Some(k) = Grid::line_key(d, p.to_index(d).i) {
                self.stale |= 1 << k;
            }
        }
    }

    pub fn is_synced(&self) -> bool {
        self.stale == 0
    }

    /// Recomputes the stale lines from `board`.
    pub fn sync(&mut self, board: &Board) {
        for k in Bits(std::mem::take(&mut self.stale)) {
            self.update(k as usize, board);
        }
    }

    /// What `r` would make at `p`. The map must be in sync with the board;
    /// an occupied point makes nothing.
    pub fn get(&self, p: Point, r: Player) -> Shapes {
        debug_assert!(self.is_synced());
        self.shapes[player_index(r)][point_index(p)]
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
        let (d, i) = Grid::line_of_key(k);
        let line = board.line(d, i).unwrap();
        for r in [Black, White] {
            // Lowest first, so that a cell gets the biggest shape it makes.
            let masks = [
                (
                    Shape::Two,
                    line.eyes_of(line.open_starts(r, 1), Two.eye_cells()),
                ),
                (
                    Shape::Sword,
                    line.eyes_of(line.scoring(r, 2), Sword.eye_cells()),
                ),
                (Shape::Three, line.row_eyes(r, Two)),
                (Shape::Four, line.row_eyes(r, Sword)),
                (Shape::Five, line.row_eyes(r, Four)),
            ];
            let shapes = &mut self.shapes[player_index(r)];
            for j in 0..line.size {
                let p = Index::new(d, i, j).to_point();
                let s = masks
                    .iter()
                    .rev()
                    .find(|(_, mask)| mask & 1 << j != 0)
                    .map_or(Shape::Nothing, |&(s, _)| s);
                shapes[point_index(p)].0[d as usize] = s;
            }
        }
    }
}

impl Default for ShapeMap {
    fn default() -> Self {
        Self::new()
    }
}

fn point_index(p: Point) -> usize {
    p.0 as usize * RANGE as usize + p.1 as usize
}

fn player_index(r: Player) -> usize {
    if r.is_black() { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::Shape::*;
    use super::{Shape, ShapeMap};
    use crate::board::Direction::Vertical;
    use crate::board::Player::{self, *};
    use crate::board::{Board, Points};

    fn shapes(map: &ShapeMap, p: &str, r: Player) -> [Shape; 4] {
        map.get(p.parse().unwrap(), r).0
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
        let map = ShapeMap::init(&board);
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
        let map = ShapeMap::init(&board);
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
        let mut map = ShapeMap::init(&board);
        let moves = "G7,K9,H9,F6,G8,I7".parse::<Points>()?.into_vec();
        let mut turn = Black;
        for (n, &p) in moves.iter().enumerate() {
            board.put_mut(turn, p);
            map.mark_stale(p);
            if n % 2 == 1 {
                map.sync(&board);
                assert!(map.shapes == ShapeMap::init(&board).shapes);
            }
            turn = turn.opponent();
        }
        for &p in moves.iter().rev() {
            board.remove_mut(p);
            map.mark_stale(p);
        }
        map.sync(&board);
        assert!(map.shapes == ShapeMap::init(&board).shapes);
        Ok(())
    }
}
