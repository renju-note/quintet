use crate::board::Direction::*;
use crate::board::StructureKind::Sword;
use crate::board::*;

/// Stored lines: 15 vertical, 15 horizontal, then the 21 ascending and 21
/// descending diagonals long enough to hold a five, in [`Square`]'s order.
const LINE_NUM: usize = 72;
/// The diagonals shorter than five are not stored (see [`Square::line`]).
const D_LINE_OMIT: u8 = VICTORY - 1;
/// Five-cell windows in a full-length line.
const WINDOWS: usize = (RANGE - VICTORY + 1) as usize;

/// Where one player's swords are, kept up to date move by move.
///
/// A sword is three own stones in a five-cell window with no opponent stone
/// (for Black, also no own stone just outside it: `exact`), so it has two
/// empty eyes: playing one makes a four and leaves the other as the only
/// block. The pairs of eyes are what a VCF search tries, and scanning the
/// whole board for them at every node is most of its cost. This keeps them
/// per line instead, so that a move only costs the (at most four) lines
/// through it.
///
/// Updates are lazy: [`Self::mark_stale`] only notes which lines a move
/// touched, and [`Self::sync`] recomputes those lines from the board. Read
/// the pairs ([`Self::move_pairs`], [`Self::move_pairs_on`]) only when the
/// field is in sync with the board.
///
/// The pairs come out in the order [`Board::structures`] /
/// [`Board::structures_on`] would list the swords, so a search that switches
/// between the two explores the same tree.
#[derive(Clone)]
pub struct SwordField {
    player: Player,
    /// Lines holding at least one sword, one bit per line.
    occupied: u128,
    /// Per line, bit `j` is set if a sword's window starts at cell `j`.
    starts: [u16; LINE_NUM],
    /// Per line and window start, the sword's stones: bit `k` for cell
    /// `j + k`. Only meaningful where `starts` has the bit.
    patterns: [[u8; WINDOWS]; LINE_NUM],
    /// Lines that moves have touched since the last [`Self::sync`].
    stale: u128,
}

impl SwordField {
    /// A field that knows nothing yet: every line is stale.
    pub fn new(player: Player) -> Self {
        Self {
            player,
            occupied: 0,
            starts: [0; LINE_NUM],
            patterns: [[0; WINDOWS]; LINE_NUM],
            stale: (1 << LINE_NUM) - 1,
        }
    }

    pub fn init(player: Player, board: &Board) -> Self {
        let mut result = Self::new(player);
        result.sync(board);
        result
    }

    pub fn player(&self) -> Player {
        self.player
    }

    /// Notes that a stone was put on or taken off `p`.
    pub fn mark_stale(&mut self, p: Point) {
        for k in lines_through(p).into_iter().flatten() {
            self.stale |= 1 << k;
        }
    }

    pub fn is_synced(&self) -> bool {
        self.stale == 0
    }

    /// Recomputes the stale lines from `board`.
    pub fn sync(&mut self, board: &Board) {
        while self.stale != 0 {
            let k = self.stale.trailing_zeros() as usize;
            self.stale &= self.stale - 1;
            self.update_line(k, board);
        }
    }

    /// Every sword's eyes, both ways round: `(four-making move, the block
    /// it leaves)`.
    pub fn move_pairs(&self) -> Vec<(Point, Point)> {
        debug_assert!(self.is_synced());
        let mut result = vec![];
        let mut occupied = self.occupied;
        while occupied != 0 {
            let k = occupied.trailing_zeros() as usize;
            occupied &= occupied - 1;
            self.push_pairs(k, self.starts[k], &mut result);
        }
        result
    }

    /// Like [`Self::move_pairs`], for the swords with `p` in their window.
    pub fn move_pairs_on(&self, p: Point) -> Vec<(Point, Point)> {
        debug_assert!(self.is_synced());
        let mut result = vec![];
        for (d, k) in DIRECTIONS.into_iter().zip(lines_through(p)) {
            let Some(k) = k else { continue };
            let j = p.to_index(d).j;
            // Windows starting at `j - 4` to `j`.
            let lo = j.saturating_sub(VICTORY - 1);
            let mask = ((1u32 << (j + 1)) - (1u32 << lo)) as u16;
            let starts = self.starts[k] & mask;
            if starts != 0 {
                self.push_pairs(k, starts, &mut result);
            }
        }
        result
    }

    fn push_pairs(&self, k: usize, mut starts: u16, out: &mut Vec<(Point, Point)>) {
        let (d, i) = line_of(k);
        while starts != 0 {
            let j = starts.trailing_zeros() as u8;
            starts &= starts - 1;
            // Three stones in five cells: the two empty cells are the eyes.
            let mut eyes = !self.patterns[k][j as usize] & 0b11111;
            let e1 = eyes.trailing_zeros() as u8;
            eyes &= eyes - 1;
            let e2 = eyes.trailing_zeros() as u8;
            let p1 = Index::new(d, i, j + e1).to_point();
            let p2 = Index::new(d, i, j + e2).to_point();
            out.push((p1, p2));
            out.push((p2, p1));
        }
    }

    fn update_line(&mut self, k: usize, board: &Board) {
        let (d, i) = line_of(k);
        let mut starts = 0;
        if let Some(line) = board.line(d, i) {
            let (sk, n, exact) = Sword.to_sequence(self.player);
            if line.potential_cap(self.player) > n {
                for (j, s) in line.sequences(self.player, sk, n, exact) {
                    starts |= 1 << j;
                    self.patterns[k][j as usize] = s.0;
                }
            }
        }
        self.starts[k] = starts;
        if starts != 0 {
            self.occupied |= 1 << k;
        } else {
            self.occupied &= !(1 << k);
        }
    }
}

const DIRECTIONS: [Direction; 4] = [Vertical, Horizontal, Ascending, Descending];

/// The stored lines through `p`, in [`DIRECTIONS`] order.
fn lines_through(p: Point) -> [Option<usize>; 4] {
    let r = RANGE as usize;
    let d_num = (RANGE - D_LINE_OMIT) as usize * 2 - 1;
    let diagonal = |i: u8| {
        (D_LINE_OMIT..D_LINE_OMIT + d_num as u8)
            .contains(&i)
            .then(|| (i - D_LINE_OMIT) as usize)
    };
    [
        Some(p.to_index(Vertical).i as usize),
        Some(r + p.to_index(Horizontal).i as usize),
        diagonal(p.to_index(Ascending).i).map(|k| 2 * r + k),
        diagonal(p.to_index(Descending).i).map(|k| 2 * r + d_num + k),
    ]
}

/// The direction and index (as in [`Index`]) of stored line `k`.
fn line_of(k: usize) -> (Direction, u8) {
    let r = RANGE as usize;
    let d_num = (RANGE - D_LINE_OMIT) as usize * 2 - 1;
    if k < r {
        (Vertical, k as u8)
    } else if k < 2 * r {
        (Horizontal, (k - r) as u8)
    } else if k < 2 * r + d_num {
        (Ascending, (k - 2 * r) as u8 + D_LINE_OMIT)
    } else {
        (Descending, (k - 2 * r - d_num) as u8 + D_LINE_OMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};

    /// What a full scan of the board says: the pairs `VCFState` built before
    /// this field existed.
    fn scanned(swords: impl Iterator<Item = Structure>) -> Vec<(Point, Point)> {
        let mut result = vec![];
        for s in swords {
            let eyes: Vec<_> = s.eyes().collect();
            result.push((eyes[0], eyes[1]));
            result.push((eyes[1], eyes[0]));
        }
        result
    }

    fn assert_matches_scan(field: &SwordField, board: &Board) {
        let r = field.player();
        assert_eq!(field.move_pairs(), scanned(board.structures(r, Sword)));
        for p in (0..RANGE).flat_map(|x| (0..RANGE).map(move |y| Point(x, y))) {
            assert_eq!(
                field.move_pairs_on(p),
                scanned(board.structures_on(p, r, Sword)),
                "{r:?} on {p}"
            );
        }
    }

    fn board() -> Board {
        // Swords in all four directions, some blocked, some along the edge
        // and the short diagonals, and Black's overline margin (A1-E5 is
        // next to F6).
        "A1,B2,D4,F6,H8,H9,H11,J8,K8,M8,N14,M13,L12,C12,C13,C15/O1,N2,L4,O14,O13,O11,D12,E8,F8,G8"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_init_matches_a_scan() {
        let board = board();
        for r in [Black, White] {
            assert_matches_scan(&SwordField::init(r, &board), &board);
        }
    }

    /// Only the lines through each move are recomputed, lazily; that has to
    /// leave the field exactly as a full scan of the board sees it.
    #[test]
    fn test_sync_matches_a_scan() -> Result<(), String> {
        let moves = "H10,G9,J10,H7,I8,E4,C14,A15".parse::<Points>()?.into_vec();
        for r in [Black, White] {
            let mut board = board();
            let mut field = SwordField::init(r, &board);
            let mut turn = Black;
            for (n, &p) in moves.iter().enumerate() {
                board.put_mut(turn, p);
                field.mark_stale(p);
                turn = turn.opponent();
                // Sync every other move, so that several moves are stale at
                // once.
                if n % 2 == 1 {
                    assert!(!field.is_synced());
                    field.sync(&board);
                    assert_matches_scan(&field, &board);
                }
            }
            for &p in moves.iter().rev() {
                board.remove_mut(p);
                field.mark_stale(p);
            }
            field.sync(&board);
            assert_matches_scan(&field, &board);
        }
        Ok(())
    }
}
