use crate::board::Direction::*;
use crate::board::Player::*;
use crate::board::RowKind::Sword;
use crate::board::*;

/// Where each player's swords are, kept alongside a [`Board`].
///
/// A sword is three own stones in a five-cell window with no opponent stone
/// (see [`RowKind::Sword`]); its two empty eyes are what a VCF search
/// plays. The search asks for them at nearly every node, and scanning every
/// line for them was most of its cost. This keeps, per player and per line
/// (by [`Grid::line_key`]), the cells where a sword's window starts.
///
/// Updates are lazy. A move only changes the (at most four) lines through
/// it, so [`Self::mark_stale`] just notes those, and [`Self::sync`]
/// recomputes them from the board before the next read. Marking rather than recomputing at
/// once matters: the searches move far more often than they read.
///
/// [`Self::swords`] / [`Self::swords_on`] return what
/// `Board::rows(r, Sword)` / `rows_on(p, r, Sword)` would, in
/// the same order, so a search reading either explores the same tree.
#[derive(Clone)]
pub struct SwordMap {
    /// Black's and White's, in that order.
    players: [Swords; 2],
    /// Lines changed since the last [`Self::sync`], one bit per line.
    stale: u128,
}

/// One player's swords.
#[derive(Clone)]
struct Swords {
    /// Per line, bit `j` is set if a sword's window starts at cell `j`.
    starts: [u16; LINE_NUM],
    /// The lines with at least one sword, one bit per line.
    lines: u128,
}

impl SwordMap {
    /// A map that knows nothing yet: every line is stale.
    pub fn new() -> Self {
        let empty = Swords {
            starts: [0; LINE_NUM],
            lines: 0,
        };
        Self {
            players: [empty.clone(), empty],
            stale: (1 << LINE_NUM) - 1,
        }
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

    /// A map in sync with `board`.
    pub fn init(board: &Board) -> Self {
        let mut result = Self::new();
        result.sync(board);
        result
    }

    /// Recomputes the stale lines from `board`.
    pub fn sync(&mut self, board: &Board) {
        while self.stale != 0 {
            let k = self.stale.trailing_zeros() as usize;
            self.stale &= self.stale - 1;
            self.update(k, board);
        }
    }

    /// `r`'s swords. The map must be in sync with `board`.
    pub fn swords<'a>(&'a self, board: &'a Board, r: Player) -> impl Iterator<Item = Row> + 'a {
        debug_assert!(self.is_synced());
        let swords = &self.players[player_index(r)];
        bits(swords.lines).flat_map(move |k| {
            let k = k as usize;
            swords_in(board, r, k, swords.starts[k])
        })
    }

    /// `r`'s swords with `p` in their window. The map must be in sync with
    /// `board`.
    pub fn swords_on<'a>(
        &'a self,
        board: &'a Board,
        p: Point,
        r: Player,
    ) -> impl Iterator<Item = Row> + 'a {
        debug_assert!(self.is_synced());
        let swords = &self.players[player_index(r)];
        [Vertical, Horizontal, Ascending, Descending]
            .into_iter()
            .filter_map(move |d| {
                let index = p.to_index(d);
                let k = Grid::line_key(d, index.i)?;
                // Windows starting at `j - 4` to `j`.
                let j = index.j;
                let lo = j.saturating_sub(VICTORY - 1);
                let mask = ((1u32 << (j + 1)) - (1u32 << lo)) as u16;
                Some(swords_in(board, r, k, swords.starts[k] & mask))
            })
            .flatten()
    }

    fn update(&mut self, k: usize, board: &Board) {
        let (d, i) = Grid::line_of_key(k);
        let line = board.line(d, i).unwrap();
        for r in [Black, White] {
            let swords = &mut self.players[player_index(r)];
            swords.starts[k] = line.row_starts(r, Sword);
            if swords.starts[k] != 0 {
                swords.lines |= 1 << k;
            } else {
                swords.lines &= !(1 << k);
            }
        }
    }
}

impl Default for SwordMap {
    fn default() -> Self {
        Self::new()
    }
}

/// The swords of line `k` whose windows start at the bits of `starts`.
fn swords_in(board: &Board, r: Player, k: usize, starts: u16) -> impl Iterator<Item = Row> + '_ {
    let (d, i) = Grid::line_of_key(k);
    let line = board.line(d, i).unwrap();
    bits(starts as u128).map(move |j| Row::new(Index::new(d, i, j), r, Sword, line.segment(j)))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The map has to say what a scan of the grid says, point for point
    /// and in the same order.
    fn assert_matches_a_scan(map: &mut SwordMap, board: &Board) {
        map.sync(board);
        for r in [Black, White] {
            let scanned: Vec<_> = board.rows(r, Sword).collect();
            assert_eq!(map.swords(board, r).collect::<Vec<_>>(), scanned, "{r:?}");
            for x in 0..RANGE {
                for y in 0..RANGE {
                    let p = Point(x, y);
                    let scanned: Vec<_> = board.rows_on(p, r, Sword).collect();
                    assert_eq!(
                        map.swords_on(board, p, r).collect::<Vec<_>>(),
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
        let mut map = SwordMap::new();
        assert!(!map.is_synced());
        map.sync(&board);
        assert!(map.swords(&board, Black).count() >= 7);
        assert!(map.swords(&board, White).count() >= 4);
        assert_matches_a_scan(&mut map, &board);

        // Kept up to date as stones come and go, synced only every other
        // move so that several are stale at once.
        let moves = "H10,G9,J10,H7,I8,E4,C14,A15".parse::<Points>()?.into_vec();
        let mut turn = Black;
        for (n, &p) in moves.iter().enumerate() {
            board.put_mut(turn, p);
            map.mark_stale(p);
            assert!(!map.is_synced());
            if n % 2 == 1 {
                assert_matches_a_scan(&mut map, &board);
            }
            turn = turn.opponent();
        }
        for (n, &p) in moves.iter().enumerate().rev() {
            board.remove_mut(p);
            map.mark_stale(p);
            if n % 3 == 0 {
                assert_matches_a_scan(&mut map, &board);
            }
        }
        assert_matches_a_scan(&mut map, &board);
        Ok(())
    }
}
