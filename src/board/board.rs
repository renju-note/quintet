use super::forbidden::*;
use super::grid::*;
use super::line::*;
use super::player::*;
use super::point::*;
use super::structure::*;
use super::zobrist;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct Board {
    grid: Grid,
    z_hash: u64,
}

impl Board {
    pub fn new() -> Self {
        Self::from_grid(Grid::new(), zobrist::new())
    }

    pub fn from_stones(blacks: &Points, whites: &Points) -> Self {
        let grid = Grid::from_stones(blacks, whites);
        let z_hash = zobrist::from_stones(blacks, whites);
        Self::from_grid(grid, z_hash)
    }

    fn from_grid(grid: Grid, z_hash: u64) -> Self {
        Self { grid, z_hash }
    }

    pub fn put_mut(&mut self, r: Player, p: Point) {
        // Only the hash has to be told about a stone that was already here:
        // `Grid::put_mut` sets this player's bit and clears the other's, so
        // it overwrites whatever was there by itself. Taking the stone off
        // first meant writing all four lines twice, at every move a search
        // makes.
        if let Some(previous) = self.stone(p) {
            self.update_z_hash(previous, p);
        }
        self.grid.put_mut(r, p);
        self.update_z_hash(r, p);
    }

    pub fn remove_mut(&mut self, p: Point) {
        if let Some(r) = self.stone(p) {
            self.update_z_hash(r, p)
        }
        self.grid.remove_mut(p);
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
        self.grid.stone(p)
    }

    pub fn stones(&self, r: Player) -> impl Iterator<Item = Point> + '_ {
        self.grid.stones(r)
    }

    pub fn empties(&self) -> impl Iterator<Item = Point> + '_ {
        self.grid.empties()
    }

    pub fn neighbors(
        &self,
        p: Point,
        distance: u8,
        only_empty: bool,
    ) -> impl Iterator<Item = Point> + '_ {
        self.grid.neighbors(p, distance, only_empty)
    }

    pub fn line(&self, d: Direction, i: u8) -> Option<&Line> {
        self.grid.line(d, i)
    }

    pub fn line_on(&self, p: Point, d: Direction) -> Option<&Line> {
        self.grid.line_on(p, d)
    }

    pub fn lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
        self.grid.lines()
    }

    pub fn lines_on(&self, p: Point) -> impl Iterator<Item = (Direction, u8, &Line)> {
        self.grid.lines_on(p)
    }

    pub fn structures(&self, r: Player, k: StructureKind) -> impl Iterator<Item = Structure> + '_ {
        self.grid.structures(r, k)
    }

    pub fn structures_on(
        &self,
        p: Point,
        r: Player,
        k: StructureKind,
    ) -> impl Iterator<Item = Structure> + '_ {
        self.grid.structures_on(p, r, k)
    }

    pub fn potentials(&self, r: Player, min: u8) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.grid.potentials(r, min)
    }

    pub fn potentials_along(
        &self,
        p: Point,
        r: Player,
        min: u8,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.grid.potentials_along(p, r, min)
    }

    pub fn to_pretty_string(&self) -> String {
        self.grid.to_pretty_string()
    }

    pub fn forbiddens(&self) -> Vec<(ForbiddenKind, Point)> {
        forbiddens(&self.grid)
    }

    pub fn forbidden_strict(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden_strict(&self.grid, p)
    }

    pub fn forbidden(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden(&self.grid, p)
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.z_hash
    }

    pub fn zobrist_hash_n(&self, n: u8) -> u64 {
        zobrist::apply_n(self.z_hash, n)
    }

    fn update_z_hash(&mut self, r: Player, p: Point) {
        self.z_hash = zobrist::apply_move(self.z_hash, r, p);
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.grid.to_string())
    }
}

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s.parse::<Grid>()?;
        let blacks = grid.stones(Black).collect();
        let whites = grid.stones(White).collect();
        let z_hash = zobrist::from_stones(&Points(blacks), &Points(whites));

        Ok(Self::from_grid(grid, z_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hash is kept up to date move by move; it has to stay the hash
    /// of the stones on the board, however they got there.
    fn assert_hash_is_of_the_stones(board: &Board) {
        let blacks = Points(board.stones(Black).collect());
        let whites = Points(board.stones(White).collect());
        let from_scratch = Board::from_stones(&blacks, &whites);
        assert_eq!(board.zobrist_hash(), from_scratch.zobrist_hash());
    }

    #[test]
    fn test_zobrist_hash() {
        let mut board = Board::new();
        let empty = board.zobrist_hash();

        board.put_mut(Black, Point(7, 7));
        board.put_mut(White, Point(8, 8));
        board.put_mut(Black, Point(9, 8));
        assert_hash_is_of_the_stones(&board);
        let three_stones = board.zobrist_hash();
        assert_ne!(three_stones, empty);

        // Taking a stone off and putting it back restores the hash.
        board.remove_mut(Point(8, 8));
        assert_hash_is_of_the_stones(&board);
        assert_ne!(board.zobrist_hash(), three_stones);
        board.put_mut(White, Point(8, 8));
        assert_eq!(board.zobrist_hash(), three_stones);

        // Putting over a stone replaces it, in the hash too.
        board.put_mut(White, Point(7, 7));
        assert_eq!(board.stone(Point(7, 7)), Some(White));
        assert_hash_is_of_the_stones(&board);

        // Removing from an empty point changes nothing.
        let before = board.zobrist_hash();
        board.remove_mut(Point(0, 0));
        assert_eq!(board.zobrist_hash(), before);

        // The copying versions leave the original alone.
        let next = board.put(Black, Point(0, 0));
        assert_eq!(board.zobrist_hash(), before);
        assert_eq!(next.remove(Point(0, 0)).zobrist_hash(), before);
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "H8,J9/I9".parse::<Board>()?;
        let mut expected = Board::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));
        assert_eq!(result.grid, expected.grid);
        assert_eq!(result.z_hash, expected.z_hash);

        Ok(())
    }
}
