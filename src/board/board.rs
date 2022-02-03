use super::forbidden::*;
use super::player::*;
use super::point::*;
use super::sequence::*;
use super::square::*;
use super::zobrist;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct Board {
    square: Square,
    z_hash: u64,
}

impl Board {
    pub fn new() -> Self {
        Self {
            square: Square::new(),
            z_hash: zobrist::new(),
        }
    }

    pub fn from_stones(blacks: &Points, whites: &Points) -> Self {
        let square = Square::from_stones(blacks, whites);
        let z_hash = zobrist::from_stones(blacks, whites);
        Self {
            square: square,
            z_hash: z_hash,
        }
    }

    pub fn put_mut(&mut self, r: Player, p: Point) {
        self.remove_mut(p);
        self.square.put_mut(r, p);
        self.update_z_hash(r, p);
    }

    pub fn remove_mut(&mut self, p: Point) {
        self.stone(p).map(|r| self.update_z_hash(r, p));
        self.square.remove_mut(p);
    }

    pub fn put(&self, r: Player, p: Point) -> Self {
        let mut result = self.clone();
        result.put_mut(r, p);
        result
    }

    pub fn remove(&self, p: Point) -> Self {
        let mut result = self.clone();
        result.remove_mut(p);
        result
    }

    pub fn stone(&self, p: Point) -> Option<Player> {
        self.square.stone(p)
    }

    pub fn stones(&self, r: Player) -> impl Iterator<Item = Point> + '_ {
        self.square.stones(r)
    }

    pub fn points_along(&self, p: Point, limit: u8) -> impl Iterator<Item = Point> + '_ {
        self.square.points_along(p, limit)
    }

    pub fn sequences(
        &self,
        r: Player,
        k: SequenceKind,
        n: u8,
        strict: bool,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        self.square.sequences(r, k, n, strict)
    }

    pub fn sequences_on(
        &self,
        p: Point,
        r: Player,
        k: SequenceKind,
        n: u8,
        strict: bool,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        self.square.sequences_on(p, r, k, n, strict)
    }

    pub fn potentials(
        &self,
        r: Player,
        min: u8,
        strict: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.square.potentials(r, min, strict)
    }

    pub fn potentials_along(
        &self,
        p: Point,
        r: Player,
        min: u8,
        strict: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.square.potentials_along(p, r, min, strict)
    }

    pub fn to_pretty_string(&self) -> String {
        self.square.to_pretty_string()
    }

    pub fn forbiddens(&self) -> Vec<(ForbiddenKind, Point)> {
        forbiddens(&self.square)
    }

    pub fn forbidden_strict(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden_strict(&self.square, p)
    }

    pub fn forbidden(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden(&self.square, p)
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.z_hash
    }

    fn update_z_hash(&mut self, r: Player, p: Point) {
        self.z_hash = zobrist::apply(self.z_hash, r, p);
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.square.to_string())
    }
}

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let square = s.parse::<Square>()?;
        let blacks = square.stones(Black).collect();
        let whites = square.stones(White).collect();
        let z_hash = zobrist::from_stones(&Points(blacks), &Points(whites));

        Ok(Self {
            square: square,
            z_hash: z_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), String> {
        let mut board = Board::new();
        board.put_mut(Black, Point(7, 7));
        board.put_mut(White, Point(7, 8));
        board.put_mut(Black, Point(9, 9));
        board.put_mut(White, Point(8, 8));
        board.put_mut(Black, Point(6, 8));
        board.put_mut(White, Point(8, 6));
        board.put_mut(Black, Point(6, 9));
        board.put_mut(White, Point(8, 9));
        board.put_mut(Black, Point(8, 7));
        board.put_mut(White, Point(5, 6));

        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . o . x o . . . . .
             . . . . . . o x x . . . . . .
             . . . . . . . o o . . . . . .
             . . . . . x . . x . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(board.to_string(), expected);

        // stones
        let result = board.stones(Black).collect::<Vec<_>>();
        let expected = vec![
            Point(6, 8),
            Point(6, 9),
            Point(7, 7),
            Point(8, 7),
            Point(9, 9),
        ];
        assert_eq!(result, expected);

        let result = board.stones(White).collect::<Vec<_>>();
        let expected = vec![
            Point(5, 6),
            Point(7, 8),
            Point(8, 6),
            Point(8, 8),
            Point(8, 9),
        ];
        assert_eq!(result, expected);

        // forbiddens
        assert_eq!(board.forbiddens(), [(DoubleThree, Point(6, 7))]);
        assert_eq!(board.forbidden(Point(6, 7)), Some(DoubleThree));

        // sequences
        let result: Vec<_> = board.sequences(Black, Compact, 2, true).collect();
        let expected = [
            (Index::new(Vertical, 6, 6), Sequence(0b00011100)),
            (Index::new(Vertical, 6, 7), Sequence(0b00010110)),
            (Index::new(Vertical, 6, 8), Sequence(0b00010011)),
            (Index::new(Horizontal, 7, 5), Sequence(0b00011100)),
            (Index::new(Horizontal, 7, 6), Sequence(0b00010110)),
            (Index::new(Horizontal, 7, 7), Sequence(0b00010011)),
        ];
        assert_eq!(result, expected);

        let result: Vec<_> = board.sequences(White, Compact, 2, false).collect();
        let expected = [
            (Index::new(Horizontal, 6, 5), Sequence(0b00011001)),
            (Index::new(Ascending, 13, 6), Sequence(0b00010110)),
            (Index::new(Ascending, 13, 7), Sequence(0b00010011)),
        ];
        assert_eq!(result, expected);

        let result: Vec<_> = board.sequences(White, Compact, 3, false).collect();
        let expected = [(Index::new(Ascending, 13, 5), Sequence(0b00011101))];
        assert_eq!(result, expected);

        // potentials
        let result: Vec<_> = board.potentials(Black, 3, true).collect();
        let expected = [
            (Index::new(Vertical, 6, 5), 3),
            (Index::new(Vertical, 6, 6), 6),
            (Index::new(Vertical, 6, 7), 9),
            (Index::new(Vertical, 6, 10), 9),
            (Index::new(Vertical, 6, 11), 6),
            (Index::new(Vertical, 6, 12), 3),
            (Index::new(Horizontal, 7, 4), 3),
            (Index::new(Horizontal, 7, 5), 6),
            (Index::new(Horizontal, 7, 6), 9),
            (Index::new(Horizontal, 7, 9), 9),
            (Index::new(Horizontal, 7, 10), 6),
            (Index::new(Horizontal, 7, 11), 3),
            (Index::new(Descending, 14, 3), 3),
            (Index::new(Descending, 14, 4), 3),
            (Index::new(Descending, 14, 5), 3),
        ];
        assert_eq!(result, expected);

        let result: Vec<_> = board.potentials(White, 3, true).collect();
        let expected = [
            (Index::new(Vertical, 8, 10), 3),
            (Index::new(Vertical, 8, 11), 3),
            (Index::new(Vertical, 8, 12), 3),
            (Index::new(Horizontal, 6, 4), 3),
            (Index::new(Horizontal, 6, 6), 6),
            (Index::new(Horizontal, 6, 7), 6),
            (Index::new(Horizontal, 6, 9), 3),
            (Index::new(Horizontal, 8, 9), 3),
            (Index::new(Horizontal, 8, 10), 3),
            (Index::new(Horizontal, 8, 11), 3),
            (Index::new(Ascending, 13, 4), 4),
            (Index::new(Ascending, 13, 6), 8),
            (Index::new(Ascending, 13, 9), 7),
            (Index::new(Ascending, 13, 10), 3),
            (Index::new(Ascending, 13, 11), 3),
        ];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_zobrist_hash() {
        let mut board = Board::new();
        let hash0 = board.zobrist_hash();

        board.put_mut(Black, Point(7, 7));
        let hash1 = board.zobrist_hash();
        assert_ne!(hash1, hash0);

        board.put_mut(White, Point(8, 8));
        let hash2 = board.zobrist_hash();
        assert_ne!(hash2, hash0);
        assert_ne!(hash2, hash1);

        board.put_mut(Black, Point(9, 8));
        let hash3 = board.zobrist_hash();
        assert_ne!(hash3, hash0);
        assert_ne!(hash3, hash1);
        assert_ne!(hash3, hash2);

        board.remove_mut(Point(9, 8));
        let hash4 = board.zobrist_hash();
        assert_ne!(hash4, hash3);
        assert_eq!(hash4, hash2);

        board.put_mut(Black, Point(9, 8));
        let hash5 = board.zobrist_hash();
        assert_eq!(hash5, hash3);

        board.remove_mut(Point(8, 8));
        let hash6 = board.zobrist_hash();
        assert_ne!(hash6, hash5);

        board.put_mut(White, Point(8, 8));
        let hash7 = board.zobrist_hash();
        assert_eq!(hash7, hash5);
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "H8,J9/I9".parse::<Board>()?;
        let mut expected = Board::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));
        assert_eq!(result.square, expected.square);
        assert_eq!(result.z_hash, expected.z_hash);

        Ok(())
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| " ".to_string() + ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
