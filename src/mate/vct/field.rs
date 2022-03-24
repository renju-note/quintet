use crate::board::Direction::*;
use crate::board::Player::*;
use crate::board::*;

#[derive(Default, Clone)]
pub struct Potential {
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

type Potentials = [[Potential; RANGE as usize]; RANGE as usize];

#[derive(Clone)]
pub struct PotentialField {
    potentials: Potentials,
    player: Player,
    min: u8,
}

impl PotentialField {
    pub fn new(player: Player, min: u8) -> Self {
        Self {
            potentials: Potentials::default(),
            player: player,
            min: min,
        }
    }

    pub fn init(player: Player, min: u8, board: &Board) -> Self {
        let mut result = Self::new(player, min);
        let os = board.potentials(player, min, player.is_black());
        for (idx, o) in os {
            result.set(idx, o);
        }
        result
    }

    pub fn update_along(&mut self, p: Point, board: &Board) {
        self.reset_along(p);
        let os = board.potentials_along(p, self.player, self.min, self.player.is_black());
        for (idx, o) in os {
            self.set(idx, o);
        }
    }

    pub fn get(&self, p: Point) -> u8 {
        self.sum(p)
    }

    pub fn collect(&self, min: u8) -> Vec<(Point, u8)> {
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

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), String> {
        let board = "
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
        .parse::<Board>()?;

        let field = PotentialField::init(Black, 3, &board);
        let result = field.overlay(&board);
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
        assert_eq!(result, expected);

        let field = PotentialField::init(Black, 2, &board);
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . 2 . . 2 . . . 2 2 . . 2 .
             . . . 2 . . 7 . . 8 2 . 2 . .
             . . . 3 2 . 6 6 610 . 2 . . .
             . . . . 3 2 9 814 8 2 . . . .
             . . 2 4 4 6 o14 6 o 3 4 4 2 .
             . . 2 2 210 o x x 8 8 . . . .
             . . . 2101417 o 812 4 8 . . .
             . . . 4 6 . 8 2 x 4 . . 4 . .
             . . 2 4 . 2 3 2 . 2 . . . 2 .
             . . 2 . 2 . . 2 . . . . . . .
             . . . 2 . . . 2 . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let field = PotentialField::init(White, 3, &board);
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . 3 . . . . . .
             . . . . . . o . 6 o . . . . .
             . . . . . . o x x 3 3 3 . . .
             . . . . . . . o 9 . . . . . .
             . . . . . . . . x . . . . . .
             . . . . . . . . 6 . . . . . .
             . . . . . . . . 3 . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
        ",
        );
        assert_eq!(result, expected);

        let field = PotentialField::init(White, 2, &board);
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . 2 . . 2 2 . . 2 . . .
             . . . . . 4 . 2 4 . 4 . . . .
             . . . . . . 6 2 3 6 . . 2 . .
             . . . . . . o1014 o . 4 . . .
             . . . . . . o x x 3 9 3 2 . .
             . . . . . . 8 o1116 . . . . .
             . . . . 210 6 8 x1012 4 2 . .
             . . . . 4 . . 8 6 2 2 4 . . .
             . . . 2 . . 6 . 3 . 2 2 2 . .
             . . . . . 4 . . 4 . . 2 . . .
             . . . . 2 . . . 2 . . . 2 . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
        ",
        );
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_() -> Result<(), String> {
        let mut board = "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . o . x o . . . . .
             . . . . . . o x x . . . . . .
             . . . . . . . o o . . . . . .
             . . . . . . . . x . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let mut field = PotentialField::init(White, 3, &board);
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . 3 . . 3 . . .
             . . . . . . . . 3 . 6 . . . .
             . . . . . . . . 3 9 . . . . .
             . . . . . . o . x o . . . . .
             . . . . . . o x x 3 3 3 . . .
             . . . . . . 9 o o . . . . . .
             . . . . . 6 . . x . . . . . .
             . . . . 3 . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let p = Point(5, 6);
        board.put_mut(White, p);
        field.update_along(p, &board);
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . 3 . . 3 . . .
             . . . . . . . . 3 . 6 . . . .
             . . . . . . . . 3 4 . . . . .
             . . . . . . o . x o . . . . .
             . . . . . . o x x 3 3 3 . . .
             . . . . . . 8 o o . . . . . .
             . . . . 3 x 6 6 x 3 . . . . .
             . . . . 4 . . . . . . . . . .
             . . . 3 . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = field.collect(3);
        let expected = [
            (Point(3, 4), 3),
            (Point(4, 5), 4),
            (Point(4, 6), 3),
            (Point(6, 6), 6),
            (Point(6, 7), 8),
            (Point(7, 6), 6),
            (Point(8, 10), 3),
            (Point(8, 11), 3),
            (Point(8, 12), 3),
            (Point(9, 6), 3),
            (Point(9, 8), 3),
            (Point(9, 10), 4),
            (Point(10, 8), 3),
            (Point(10, 11), 6),
            (Point(11, 8), 3),
            (Point(11, 12), 3),
        ];
        assert_eq!(result, expected);

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
