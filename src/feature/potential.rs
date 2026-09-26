use crate::board::Direction::*;
use crate::board::Player::*;
use crate::board::*;

/// How useful a stone at each point would be to one player, for move
/// ordering.
///
/// Updates can be eager ([`Self::update_along`]) or lazy: [`Self::mark_stale`]
/// only notes the point, and [`Self::sync`] recomputes the lines through
/// every point noted since. Read the field ([`Self::get`], [`Self::collect`])
/// only when it is in sync with the board.
#[derive(Clone)]
pub struct PotentialField {
    potentials: PotentialMatrix,
    player: Player,
    min: u8,
    /// Points played or taken back since the last [`Self::sync`], one bit
    /// per point.
    stale: [u64; 4],
}

impl PotentialField {
    pub fn new(player: Player, min: u8) -> Self {
        Self {
            potentials: PotentialMatrix::default(),
            player,
            min,
            stale: [0; 4],
        }
    }

    pub fn init(player: Player, min: u8, board: &Board) -> Self {
        let mut result = Self::new(player, min);
        let os = board.potentials(player, min);
        for (idx, o) in os {
            result.set(idx, o);
        }
        result
    }

    pub fn update_along(&mut self, p: Point, board: &Board) {
        self.reset_along(p);
        let os = board.potentials_along(p, self.player, self.min);
        for (idx, o) in os {
            self.set(idx, o);
        }
    }

    /// Notes that a stone was put on or taken off `p`.
    pub fn mark_stale(&mut self, p: Point) {
        let i = u8::from(p) as usize;
        self.stale[i / 64] |= 1 << (i % 64);
    }

    pub fn is_synced(&self) -> bool {
        self.stale == [0; 4]
    }

    /// Recomputes the lines through every stale point from `board`.
    pub fn sync(&mut self, board: &Board) {
        for w in 0..self.stale.len() {
            while self.stale[w] != 0 {
                let b = self.stale[w].trailing_zeros() as usize;
                self.stale[w] &= self.stale[w] - 1;
                let i = (w * 64 + b) as u8;
                self.update_along(Point(i / RANGE, i % RANGE), board);
            }
        }
    }

    pub fn get(&self, p: Point) -> u8 {
        debug_assert!(self.is_synced());
        self.sum(p)
    }

    pub fn collect(&self, min: u8) -> Vec<(Point, u8)> {
        debug_assert!(self.is_synced());
        (0..RANGE)
            .flat_map(|x| {
                (0..RANGE).map(move |y| {
                    let p = Point(x, y);
                    (p, self.sum(p))
                })
            })
            .filter(|&(_, o)| o >= min)
            .collect()
    }

    pub fn overlay(&self, board: &Board) -> String {
        (0..RANGE)
            .rev()
            .map(|y| {
                (0..RANGE)
                    .map(|x| {
                        let p = Point(x, y);
                        match board.stone(p) {
                            Some(Black) => " o".to_string(),
                            Some(White) => " x".to_string(),
                            None => match self.sum(p) {
                                0 => " .".to_string(),
                                po => format!("{: >2}", po),
                            },
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn reset_along(&mut self, p: Point) {
        let indices = [
            p.to_index(Vertical),
            p.to_index(Horizontal),
            p.to_index(Ascending),
            p.to_index(Descending),
        ];
        let neighbor_indices = indices.iter().flat_map(|idx| {
            (0..RANGE).flat_map(move |j| {
                if j <= idx.maxj() {
                    Some(Index::new(idx.d, idx.i, j))
                } else {
                    None
                }
            })
        });
        for idx in neighbor_indices {
            self.set(idx, 0);
        }
    }

    fn sum(&self, p: Point) -> u8 {
        self.potentials[p.0 as usize][p.1 as usize].sum()
    }

    fn set(&mut self, i: Index, o: u8) {
        let p = i.to_point();
        self.potentials[p.0 as usize][p.1 as usize].set(i.d, o)
    }
}

type PotentialMatrix = [[Potential; RANGE as usize]; RANGE as usize];

#[derive(Default, Clone)]
struct Potential {
    v: u8,
    h: u8,
    a: u8,
    d: u8,
}

impl Potential {
    fn set(&mut self, d: Direction, o: u8) {
        match d {
            Vertical => self.v = o,
            Horizontal => self.h = o,
            Ascending => self.a = o,
            Descending => self.d = o,
        }
    }

    fn sum(&self) -> u8 {
        // TODO: faster computation
        self.v + self.h + self.a + self.d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o . . o . . . . .
         . . . . . . o x x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()
        .unwrap()
    }

    /// A point's value is the sum of its potentials along the four lines
    /// through it (see `potential.rs`).
    #[test]
    fn test_init() {
        let board = board();
        let field = PotentialField::init(Black, 3, &board);
        // G8: three vertical windows hold both G9 and G10 (3 each), and no
        // other line through G8 reaches `min`: 9.
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . 3 . . . . . . . .
             . . . 3 . . 6 . . . . . . . .
             . . . . 3 . 9 . . . . . . . .
             . . . . . 6 o 6 6 o 3 . . . .
             . . . . . . o x x . . . . . .
             . . . . . . 9 o . . . . . . .
             . . . . . . 6 . x . . . . . .
             . . . . . . 3 . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
        ",
        );
        assert_eq!(field.overlay(&board), expected);

        // `collect` lists the same values, point by point.
        let collected = field.collect(6);
        assert_eq!(
            collected,
            [
                (Point(5, 9), 6),
                (Point(6, 6), 6),
                (Point(6, 7), 9),
                (Point(6, 10), 9),
                (Point(6, 11), 6),
                (Point(7, 9), 6),
                (Point(8, 9), 6),
            ]
        );
        assert!(collected.iter().all(|&(p, o)| field.get(p) == o));
    }

    /// `update_along` only recomputes the lines through the move. That has
    /// to leave the field exactly as a fresh `init` would.
    #[test]
    fn test_update_along_matches_init() -> Result<(), String> {
        let moves = "F8,F7,J9,H10,G11,J7".parse::<Points>()?.into_vec();
        for (player, min) in [(Black, 2), (Black, 3), (White, 2), (White, 3)] {
            let mut board = board();
            let mut field = PotentialField::init(player, min, &board);
            let mut turn = Black;
            for &p in &moves {
                board.put_mut(turn, p);
                field.update_along(p, &board);
                let fresh = PotentialField::init(player, min, &board);
                assert_eq!(
                    field.collect(0),
                    fresh.collect(0),
                    "{player:?} {min} put {p}"
                );
                turn = turn.opponent();
            }
            // Taking the moves back, as a search does on the way up.
            for &p in moves.iter().rev() {
                board.remove_mut(p);
                field.update_along(p, &board);
                let fresh = PotentialField::init(player, min, &board);
                assert_eq!(
                    field.collect(0),
                    fresh.collect(0),
                    "{player:?} {min} remove {p}"
                );
            }
        }
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
