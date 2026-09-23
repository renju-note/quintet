use super::forbidden::*;
use super::line::*;
use super::player::*;
use super::point::Direction::*;
use super::point::*;
use super::potential::VICTORY;
use super::square::*;
use super::structure::*;
use super::zobrist;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct Board {
    square: Square,
    z_hash: u64,
    /// Where each player's swords are: Black's and White's, in that order.
    ///
    /// The VCF search asks for the swords at nearly every node, and scanning
    /// the board for them was most of its cost. A move only changes the
    /// (at most four) lines through it, so a move just marks them `stale`
    /// and [`Self::sync_swords`] recomputes those before the next read.
    /// Marking rather than recomputing at once matters: the searches move
    /// far more often than they read.
    swords: [Swords; 2],
    /// Lines changed since the last [`Self::sync_swords`], by
    /// [`Square::line_key`].
    stale: u128,
}

/// One player's swords: for each line (by [`Square::line_key`]), bit `j` is
/// set if a sword's window starts at cell `j`.
#[derive(Clone)]
struct Swords {
    starts: [u16; LINE_NUM],
    /// The lines with at least one sword, one bit per line.
    lines: u128,
}

impl Board {
    pub fn new() -> Self {
        Self::from_square(Square::new(), zobrist::new())
    }

    pub fn from_stones(blacks: &Points, whites: &Points) -> Self {
        let square = Square::from_stones(blacks, whites);
        let z_hash = zobrist::from_stones(blacks, whites);
        Self::from_square(square, z_hash)
    }

    fn from_square(square: Square, z_hash: u64) -> Self {
        let empty = Swords {
            starts: [0; LINE_NUM],
            lines: 0,
        };
        let mut result = Self {
            square,
            z_hash,
            swords: [empty.clone(), empty],
            stale: (1 << LINE_NUM) - 1,
        };
        result.sync_swords();
        result
    }

    pub fn put_mut(&mut self, r: Player, p: Point) {
        // Only the hash has to be told about a stone that was already here:
        // `Square::put_mut` sets this player's bit and clears the other's, so
        // it overwrites whatever was there by itself. Taking the stone off
        // first meant writing all four lines twice, at every move a search
        // makes.
        if let Some(previous) = self.stone(p) {
            self.update_z_hash(previous, p);
        }
        self.square.put_mut(r, p);
        self.update_z_hash(r, p);
        self.mark_swords_stale(p);
    }

    pub fn remove_mut(&mut self, p: Point) {
        if let Some(r) = self.stone(p) {
            self.update_z_hash(r, p)
        }
        self.square.remove_mut(p);
        self.mark_swords_stale(p);
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

    pub fn empties(&self) -> impl Iterator<Item = Point> + '_ {
        self.square.empties()
    }

    pub fn neighbors(
        &self,
        p: Point,
        distance: u8,
        only_empty: bool,
    ) -> impl Iterator<Item = Point> + '_ {
        self.square.neighbors(p, distance, only_empty)
    }

    pub fn line(&self, d: Direction, i: u8) -> Option<&Line> {
        self.square.line(d, i)
    }

    pub fn line_on(&self, p: Point, d: Direction) -> Option<&Line> {
        self.square.line_on(p, d)
    }

    pub fn lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
        self.square.lines()
    }

    pub fn lines_on(&self, p: Point) -> impl Iterator<Item = (Direction, u8, &Line)> {
        self.square.lines_on(p)
    }

    pub fn structures(&self, r: Player, k: StructureKind) -> impl Iterator<Item = Structure> + '_ {
        self.square.structures(r, k)
    }

    pub fn structures_on(
        &self,
        p: Point,
        r: Player,
        k: StructureKind,
    ) -> impl Iterator<Item = Structure> + '_ {
        self.square.structures_on(p, r, k)
    }

    pub fn potentials(
        &self,
        r: Player,
        min: u8,
        exact: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.square.potentials(r, min, exact)
    }

    pub fn potentials_along(
        &self,
        p: Point,
        r: Player,
        min: u8,
        exact: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.square.potentials_along(p, r, min, exact)
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

    pub fn zobrist_hash_n(&self, n: u8) -> u64 {
        zobrist::apply_n(self.z_hash, n)
    }

    /// Brings the swords up to date with the stones. Call it before
    /// [`Self::swords`] or [`Self::swords_on`].
    pub fn sync_swords(&mut self) {
        while self.stale != 0 {
            let k = self.stale.trailing_zeros() as usize;
            self.stale &= self.stale - 1;
            self.update_swords(k);
        }
    }

    pub fn swords_synced(&self) -> bool {
        self.stale == 0
    }

    /// The same as `structures(r, Sword)`, in the same order, without
    /// scanning the board. The swords must be in sync.
    pub fn swords(&self, r: Player) -> impl Iterator<Item = Structure> + '_ {
        debug_assert!(self.swords_synced());
        let swords = &self.swords[player_index(r)];
        bits(swords.lines).flat_map(move |k| self.swords_in(r, k, swords.starts[k as usize]))
    }

    /// The same as `structures_on(p, r, Sword)`, in the same order, without
    /// scanning the lines. The swords must be in sync.
    pub fn swords_on(&self, p: Point, r: Player) -> impl Iterator<Item = Structure> + '_ {
        debug_assert!(self.swords_synced());
        let swords = &self.swords[player_index(r)];
        [Vertical, Horizontal, Ascending, Descending]
            .into_iter()
            .filter_map(move |d| {
                let index = p.to_index(d);
                self.square.line(d, index.i)?;
                // Windows starting at `j - 4` to `j`.
                let j = index.j;
                let lo = j.saturating_sub(VICTORY - 1);
                let mask = ((1u32 << (j + 1)) - (1u32 << lo)) as u16;
                let k = Square::line_key(d, index.i);
                Some(self.swords_in(r, k as u8, swords.starts[k] & mask))
            })
            .flatten()
    }

    fn swords_in(&self, r: Player, k: u8, starts: u16) -> impl Iterator<Item = Structure> + '_ {
        let (d, i) = Square::line_of_key(k as usize);
        let line = self.square.line(d, i).unwrap();
        bits(starts as u128).map(move |j| Structure::new(Index::new(d, i, j), line.window(r, j)))
    }

    fn mark_swords_stale(&mut self, p: Point) {
        for d in [Vertical, Horizontal, Ascending, Descending] {
            let i = p.to_index(d).i;
            if self.square.line(d, i).is_some() {
                self.stale |= 1 << Square::line_key(d, i);
            }
        }
    }

    fn update_swords(&mut self, k: usize) {
        let (d, i) = Square::line_of_key(k);
        let line = self.square.line(d, i).unwrap();
        for r in [Black, White] {
            let swords = &mut self.swords[player_index(r)];
            swords.starts[k] = line.sword_starts(r);
            if swords.starts[k] != 0 {
                swords.lines |= 1 << k;
            } else {
                swords.lines &= !(1 << k);
            }
        }
    }

    fn update_z_hash(&mut self, r: Player, p: Point) {
        self.z_hash = zobrist::apply_move(self.z_hash, r, p);
    }
}

fn player_index(r: Player) -> usize {
    if r.is_black() { 0 } else { 1 }
}

/// The positions of the set bits of `x`, lowest first.
fn bits(mut x: u128) -> impl Iterator<Item = u8> {
    std::iter::from_fn(move || {
        if x == 0 {
            return None;
        }
        let b = x.trailing_zeros() as u8;
        x &= x - 1;
        Some(b)
    })
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
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

        Ok(Self::from_square(square, z_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cached swords have to say what a scan of the board says, point
    /// for point and in the same order.
    fn assert_swords_match_a_scan(board: &mut Board) {
        board.sync_swords();
        for r in [Black, White] {
            let scanned: Vec<_> = board.structures(r, StructureKind::Sword).collect();
            assert_eq!(board.swords(r).collect::<Vec<_>>(), scanned, "{r:?}");
            for x in 0..RANGE {
                for y in 0..RANGE {
                    let p = Point(x, y);
                    let scanned: Vec<_> = board.structures_on(p, r, StructureKind::Sword).collect();
                    assert_eq!(
                        board.swords_on(p, r).collect::<Vec<_>>(),
                        scanned,
                        "{r:?} on {p}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_swords() -> Result<(), String> {
        // Swords in all four directions, some blocked, some along the edges
        // and the short diagonals, and Black's overline margin (A1-E5 is
        // next to F6).
        let mut board = "A1,B2,D4,F6,H8,H9,H11,J8,K8,M8,N14,M13,L12,C12,C13,C15/O1,N2,L4,O14,O13,O11,D12,E8,F8,G8"
            .parse::<Board>()?;
        assert!(board.swords(Black).count() >= 7);
        assert!(board.swords(White).count() >= 4);
        assert_swords_match_a_scan(&mut board);

        // Kept up to date as stones come and go, synced only every other
        // move so that several are stale at once.
        let moves = "H10,G9,J10,H7,I8,E4,C14,A15".parse::<Points>()?.into_vec();
        let mut turn = Black;
        for (n, &p) in moves.iter().enumerate() {
            board.put_mut(turn, p);
            assert!(!board.swords_synced());
            if n % 2 == 1 {
                assert_swords_match_a_scan(&mut board);
            }
            turn = turn.opponent();
        }
        for (n, &p) in moves.iter().enumerate().rev() {
            board.remove_mut(p);
            if n % 3 == 0 {
                assert_swords_match_a_scan(&mut board);
            }
        }
        assert_swords_match_a_scan(&mut board);
        Ok(())
    }

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
        assert_eq!(result.square, expected.square);
        assert_eq!(result.z_hash, expected.z_hash);

        Ok(())
    }
}
