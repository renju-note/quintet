use crate::board::Direction::*;
use crate::board::StructureKind::*;
use crate::board::*;

const DIRECTIONS: [Direction; 4] = [Vertical, Horizontal, Ascending, Descending];

/// The diagonals shorter than five hold no five-cell window and are not
/// stored, exactly as in `Square`.
const D_LINE_OMIT: u8 = VICTORY - 1;
const D_LINE_NUM: usize = ((RANGE - D_LINE_OMIT) * 2 - 1) as usize;

/// The lines, laid out in the order `Square` walks them, so that reading
/// them back in index order reads them in that order too.
const H0: usize = RANGE as usize;
const A0: usize = 2 * RANGE as usize;
const D0: usize = A0 + D_LINE_NUM;
const LINES: usize = D0 + D_LINE_NUM;

/// How many five-cell windows a line of `RANGE` cells holds, and so how many
/// swords it can hold at once — one per window, tracked in a `u16` of bits.
const WINDOWS: usize = (RANGE - VICTORY + 1) as usize;

/// Where line `(d, i)` is kept, or `None` for the diagonals `Square` does
/// not store either.
fn line_index(d: Direction, i: u8) -> Option<usize> {
    let i = i as usize;
    match d {
        Vertical => (i < RANGE as usize).then_some(i),
        Horizontal => (i < RANGE as usize).then_some(H0 + i),
        Ascending => diagonal_index(i).map(|k| A0 + k),
        Descending => diagonal_index(i).map(|k| D0 + k),
    }
}

fn diagonal_index(i: usize) -> Option<usize> {
    let k = i.checked_sub(D_LINE_OMIT as usize)?;
    (k < D_LINE_NUM).then_some(k)
}

/// [`line_index`] the other way round.
fn line_of(k: usize) -> (Direction, u8) {
    match k {
        _ if k < H0 => (Vertical, k as u8),
        _ if k < A0 => (Horizontal, (k - H0) as u8),
        _ if k < D0 => (Ascending, (k - A0) as u8 + D_LINE_OMIT),
        _ => (Descending, (k - D0) as u8 + D_LINE_OMIT),
    }
}

/// Every window of a line.
const ALL_WINDOWS: u16 = (1 << WINDOWS) - 1;

/// The windows starting in `from..=to`.
fn windows_between(from: u8, to: u8) -> u16 {
    let to = (to as usize).min(WINDOWS - 1) as u8;
    if from > to {
        return 0;
    }
    (ALL_WINDOWS >> (WINDOWS as u8 - 1 - (to - from))) << from
}

/// The windows a stone on cell `j` sits in.
fn windows_over(j: u8) -> u16 {
    windows_between(j.saturating_sub(VICTORY - 1), j)
}

/// The windows a stone on cell `j` can change: the ones it sits in, and the
/// two it is only the margin of.
fn windows_around(j: u8) -> u16 {
    windows_between(j.saturating_sub(VICTORY), j + 1)
}

/// The first and last window of a non-empty mask.
fn window_bounds(windows: u16) -> (u8, u8) {
    (
        windows.trailing_zeros() as u8,
        (u16::BITS - 1 - windows.leading_zeros()) as u8,
    )
}

/// A sword — three own stones in a five-cell window with two empty eyes —
/// reduced to what the VCF search asks of it.
///
/// Playing either eye makes a four whose one winning point is the other eye,
/// so the two of them are an `(attack, defence)` pair either way round. They
/// are held as point codes rather than [`Point`]s to halve what a new cache
/// has to blank out, which a VCT search does at nearly every node.
#[derive(Clone, Copy)]
struct SwordEyes(u8, u8);

impl SwordEyes {
    fn of(sword: &Structure) -> Self {
        let mut eyes = sword.eyes();
        // A `Sword` is three stones in five cells with no opponent stone, so
        // it has exactly two eyes.
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        Self(e1.into(), e2.into())
    }

    fn eyes(&self) -> (Point, Point) {
        (decode(self.0), decode(self.1))
    }

    fn pairs(&self) -> [(Point, Point); 2] {
        let (e1, e2) = self.eyes();
        [(e1, e2), (e2, e1)]
    }
}

/// `Point::try_from`, which cannot fail on a code this module encoded.
fn decode(code: u8) -> Point {
    Point(code / RANGE, code % RANGE)
}

/// The swords on one line, kept in the window they sit in, with `used`
/// saying which windows hold one. Walking the set bits walks the swords in
/// window order, which is the order `Square` reports them in.
#[derive(Clone, Copy)]
struct LineSwords {
    used: u16,
    swords: [SwordEyes; WINDOWS],
}

impl Default for LineSwords {
    fn default() -> Self {
        const UNUSED: SwordEyes = SwordEyes(0, 0);
        Self {
            used: 0,
            swords: [UNUSED; WINDOWS],
        }
    }
}

impl LineSwords {
    fn clear(&mut self, windows: u16) {
        self.used &= !windows;
    }

    fn set(&mut self, window: u8, sword: SwordEyes) {
        self.swords[window as usize] = sword;
        self.used |= 1 << window;
    }

    /// The swords in `windows`, in window order.
    fn iter(&self, windows: u16) -> impl Iterator<Item = SwordEyes> + '_ {
        let mut used = self.used & windows;
        std::iter::from_fn(move || {
            if used == 0 {
                return None;
            }
            let window = used.trailing_zeros() as usize;
            used &= used - 1;
            Some(self.swords[window])
        })
    }
}

/// Where a player can make a four: the eyes of every `Sword` on the board,
/// kept per line and per window.
///
/// This is [`PotentialField`](super::field::PotentialField) for fours, and
/// is maintained the same way — read the board once, then follow the moves —
/// but it keeps the swords themselves rather than a per-point number,
/// because the VCF search wants both eyes of each: play one and the four's
/// one winning point is the other, which is where the defender is then
/// forced to answer.
///
/// Following a move is deliberately only half the work. [`Self::play_along`]
/// merely notes which windows the move can have changed; the board is read
/// again when the cache is next asked something, for those windows alone. A VCF search plays far more moves than it asks for, so
/// the moves it plays and takes back without ever asking — everything below
/// its depth limit, and every state a caller drops unread — costs it four
/// `or`s and nothing else. For the same reason the first read, not the
/// constructor, is what scans the whole board.
#[derive(Clone)]
pub struct SwordEyeCache {
    player: Player,
    lines: [LineSwords; LINES],
    /// Which lines hold a sword, so that reading the whole board skips the
    /// empty lines — on most positions, nearly all of them.
    filled: u128,
    /// Which lines have windows waiting to be read off the board again.
    stale: u128,
    /// Per line, which windows those are. Only meaningful where `stale`.
    stale_windows: [u16; LINES],
    /// Whether the board has been read in full yet.
    read_in: bool,
}

impl SwordEyeCache {
    pub fn new(player: Player) -> Self {
        Self {
            player,
            lines: [LineSwords::default(); LINES],
            filled: 0,
            stale: 0,
            stale_windows: [0; LINES],
            read_in: false,
        }
    }

    pub fn init(player: Player, board: &Board) -> Self {
        let mut result = Self::new(player);
        result.read_in(board);
        result
    }

    /// Note that `p` has been played on or taken back off. Either way the
    /// windows a stone on `p` is read for are the only ones that can have
    /// changed, so they are all this has to mark.
    pub fn play_along(&mut self, p: Point) {
        if !self.read_in {
            return;
        }
        for d in DIRECTIONS {
            let idx = p.to_index(d);
            if let Some(k) = line_index(d, idx.i) {
                self.stale_windows[k] |= windows_around(idx.j);
                self.stale |= 1 << k;
            }
        }
    }

    /// Append every `(attack, defence)` of every sword: play `attack` to
    /// make a four, and `defence` is the point that would complete it, so
    /// the one the defender has to take.
    pub fn extend_eye_pairs(&mut self, board: &Board, out: &mut Vec<(Point, Point)>) {
        self.refresh(board);
        let mut filled = self.filled;
        while filled != 0 {
            let k = filled.trailing_zeros() as usize;
            filled &= filled - 1;
            for sword in self.lines[k].iter(ALL_WINDOWS) {
                out.extend(sword.pairs());
            }
        }
    }

    /// The same, from the swords `p` is one of the five stones-or-eyes of.
    pub fn extend_eye_pairs_on(&mut self, p: Point, board: &Board, out: &mut Vec<(Point, Point)>) {
        self.refresh(board);
        for (j, line) in Self::lines_on(&self.lines, p) {
            for sword in line.iter(windows_over(j)) {
                out.extend(sword.pairs());
            }
        }
    }

    /// The four `p` itself makes: the other eye of a sword `p` is an eye of,
    /// or `None` if playing `p` makes no four. It is the first such sword in
    /// the order [`Self::extend_eye_pairs_on`] reports them in.
    pub fn eye_partner(&mut self, p: Point, board: &Board) -> Option<Point> {
        self.refresh(board);
        Self::lines_on(&self.lines, p)
            .flat_map(|(j, line)| line.iter(windows_over(j)))
            .find_map(|sword| match sword.eyes() {
                (eye, other) | (other, eye) if eye == p => Some(other),
                _ => None,
            })
    }

    /// The (at most four) lines through `p`, each with `p`'s cell on it.
    fn lines_on(lines: &[LineSwords; LINES], p: Point) -> impl Iterator<Item = (u8, &LineSwords)> {
        DIRECTIONS.into_iter().filter_map(move |d| {
            let idx = p.to_index(d);
            line_index(d, idx.i).map(move |k| (idx.j, &lines[k]))
        })
    }

    /// Read the board again wherever [`Self::play_along`] said to.
    fn refresh(&mut self, board: &Board) {
        if !self.read_in {
            return self.read_in(board);
        }
        while self.stale != 0 {
            let k = self.stale.trailing_zeros() as usize;
            self.stale &= self.stale - 1;
            let (from, to) = window_bounds(std::mem::take(&mut self.stale_windows[k]));
            // Every sword between the two is read again, so clearing the
            // whole span and not just the marked windows loses nothing.
            self.lines[k].clear(windows_between(from, to));
            let (d, i) = line_of(k);
            for sword in board.structures_between(d, i, from, to, self.player, Sword) {
                self.lines[k].set(sword.start_index().j, SwordEyes::of(&sword));
            }
            self.mark(k);
        }
    }

    fn read_in(&mut self, board: &Board) {
        self.read_in = true;
        for sword in board.structures(self.player, Sword) {
            let start = sword.start_index();
            if let Some(k) = line_index(start.d, start.i) {
                self.lines[k].set(start.j, SwordEyes::of(&sword));
                self.filled |= 1 << k;
            }
        }
    }

    fn mark(&mut self, k: usize) {
        if self.lines[k].used == 0 {
            self.filled &= !(1 << k);
        } else {
            self.filled |= 1 << k;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;

    /// What the cache should hold, read straight off the board.
    fn from_board(player: Player, board: &Board) -> Vec<(Point, Point)> {
        board
            .structures(player, Sword)
            .flat_map(|s| {
                let mut eyes = s.eyes();
                let e1 = eyes.next().unwrap();
                let e2 = eyes.next().unwrap();
                [(e1, e2), (e2, e1)]
            })
            .collect()
    }

    fn from_board_on(player: Player, board: &Board, p: Point) -> Vec<(Point, Point)> {
        board
            .structures_on(p, player, Sword)
            .flat_map(|s| {
                let mut eyes = s.eyes();
                let e1 = eyes.next().unwrap();
                let e2 = eyes.next().unwrap();
                [(e1, e2), (e2, e1)]
            })
            .collect()
    }

    fn eye_pairs(cache: &mut SwordEyeCache, board: &Board) -> Vec<(Point, Point)> {
        let mut result = vec![];
        cache.extend_eye_pairs(board, &mut result);
        result
    }

    fn eye_pairs_on(cache: &mut SwordEyeCache, board: &Board, p: Point) -> Vec<(Point, Point)> {
        let mut result = vec![];
        cache.extend_eye_pairs_on(p, board, &mut result);
        result
    }

    fn board() -> Board {
        "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o x o o . . . . .
         . . . . . . . x x . . . . . .
         . . . . . . x o x o . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . . x . x . . . . .
         . . . . . . x o o o . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()
        .unwrap()
    }

    #[test]
    fn test_matches_the_board() {
        let board = board();
        for player in [Black, White] {
            let mut cache = SwordEyeCache::new(player);
            assert_eq!(eye_pairs(&mut cache, &board), from_board(player, &board));
            for p in board.empties() {
                assert_eq!(
                    eye_pairs_on(&mut cache, &board, p),
                    from_board_on(player, &board, p),
                    "{player:?} {p}"
                );
            }
        }
    }

    /// The point of the thing: after a move is played, and after it is taken
    /// back again, the cache says what a fresh read of the board would.
    #[test]
    fn test_follows_moves() {
        let mut board = board();
        let moves: Vec<Point> = ["J9", "K9", "H12", "E5", "O1", "A15"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        for player in [Black, White] {
            for mover in [Black, White] {
                for &p in &moves {
                    let mut cache = SwordEyeCache::init(player, &board);
                    // ... and one that only ever hears about the move, to
                    // pin down that reading early changes nothing.
                    let mut unread = SwordEyeCache::new(player);

                    board.put_mut(mover, p);
                    cache.play_along(p);
                    unread.play_along(p);
                    let expected = from_board(player, &board);
                    assert_eq!(eye_pairs(&mut cache, &board), expected, "{mover:?} {p}");
                    assert_eq!(eye_pairs(&mut unread, &board), expected, "{mover:?} {p}");

                    board.remove_mut(p);
                    cache.play_along(p);
                    let expected = from_board(player, &board);
                    assert_eq!(
                        eye_pairs(&mut cache, &board),
                        expected,
                        "undo {mover:?} {p}"
                    );
                }
            }
        }
    }

    /// Several moves may pile up before anything reads the cache, and each
    /// of them only marks its own windows.
    #[test]
    fn test_follows_moves_without_reading_in_between() {
        let mut board = board();
        let moves: Vec<Point> = ["J9", "K9", "H12", "G12", "F12", "E12"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        for player in [Black, White] {
            let mut cache = SwordEyeCache::init(player, &board);
            let mut mover = Black;
            for &p in &moves {
                board.put_mut(mover, p);
                cache.play_along(p);
                mover = mover.opponent();
            }
            assert_eq!(eye_pairs(&mut cache, &board), from_board(player, &board));
            for &p in moves.iter().rev() {
                board.remove_mut(p);
                cache.play_along(p);
            }
            assert_eq!(eye_pairs(&mut cache, &board), from_board(player, &board));
        }
    }

    /// `eye_partner` is `extend_eye_pairs_on` looking for its own point.
    #[test]
    fn test_eye_partner() {
        let board = board();
        for player in [Black, White] {
            let mut cache = SwordEyeCache::new(player);
            for p in board.empties() {
                let expected = eye_pairs_on(&mut cache, &board, p)
                    .into_iter()
                    .find(|&(eye, _)| eye == p)
                    .map(|(_, other)| other);
                assert_eq!(cache.eye_partner(p, &board), expected, "{player:?} {p}");
            }
        }
    }

    #[test]
    fn test_window_masks() {
        assert_eq!(windows_between(0, 0), 0b000_0000_0001);
        assert_eq!(windows_between(3, 5), 0b000_0011_1000);
        // Clamped to the windows a line actually has.
        assert_eq!(windows_between(8, 14), 0b111_0000_0000);
        assert_eq!(windows_between(5, 4), 0);

        // Cell 7 is in the windows starting at 3..=7 ...
        assert_eq!(windows_over(7), 0b000_1111_1000);
        // ... and is read for those, plus 2 and 8, as a margin.
        assert_eq!(windows_around(7), 0b001_1111_1100);
        // Both clamp at the ends of the line.
        assert_eq!(windows_over(1), 0b000_0000_0011);
        assert_eq!(windows_around(1), 0b000_0000_0111);
        // Cell 14 is only ever in the last two windows.
        assert_eq!(windows_around(14), 0b110_0000_0000);
    }

    #[test]
    fn test_line_indices_round_trip() {
        let mut seen = vec![];
        for d in DIRECTIONS {
            for i in 0..(2 * RANGE - 1) {
                let Some(k) = line_index(d, i) else { continue };
                assert_eq!(line_of(k), (d, i));
                seen.push(k);
            }
        }
        seen.sort();
        assert_eq!(seen, (0..LINES).collect::<Vec<_>>());
    }
}
