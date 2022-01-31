use super::super::board::Direction::*;
use super::super::board::Player::*;
use super::super::board::*;

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
        self.v + self.h + self.a + self.d
    }
}

#[derive(Default, Clone)]
pub struct PotentialField {
    potentials: [[Potential; RANGE as usize]; RANGE as usize],
}

impl PotentialField {
    pub fn new(potentials: impl Iterator<Item = (Index, u8)>) -> Self {
        let mut result = Self::default();
        for (idx, o) in potentials {
            result.set(idx, o);
        }
        result
    }

    pub fn collect_nonzeros(&self) -> Vec<(Point, u8)> {
        (0..RANGE)
            .flat_map(|x| {
                (0..RANGE).map(move |y| {
                    let p = Point(x, y);
                    (p, self.sum(p))
                })
            })
            .filter(|&(_, o)| o > 0)
            .collect()
    }

    pub fn update_along(&mut self, p: Point, potentials: impl Iterator<Item = (Index, u8)>) {
        self.reset_along(p);
        for (idx, o) in potentials {
            self.set(idx, o);
        }
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

        let field = PotentialField::new(board.potentials(Black, 3, true));
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . 3 . . . . . . . .
             . . . 3 . . 6 . . . . . . . .
             . . . . 3 . 9 . . . . . . . .
             . . . . . 3 o . x o . . . . .
             . . . . . . o x x . . . . . .
             . . . . 3 618 o o 9 6 3 . . .
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

        let mut field = PotentialField::new(board.potentials(White, 3, false));
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
        field.update_along(p, board.potentials_along(p, White, 3, false));
        let result = field.overlay(&board);
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . 3 . . 3 . . .
             . . . . . . . . 3 . 6 . . . .
             . . . . . . . . 310 . . . . .
             . . . . . . o . x o . . . . .
             . . . . . . o x x 3 3 3 . . .
             . . . . . .14 o o . . . . . .
             . . . . 3 x 6 6 x 3 . . . . .
             . . . . 7 . . . . . . . . . .
             . . . 3 . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = field.collect_nonzeros();
        let expected = [
            (Point(3, 4), 3),
            (Point(4, 5), 7),
            (Point(4, 6), 3),
            (Point(6, 6), 6),
            (Point(6, 7), 14),
            (Point(7, 6), 6),
            (Point(8, 10), 3),
            (Point(8, 11), 3),
            (Point(8, 12), 3),
            (Point(9, 6), 3),
            (Point(9, 8), 3),
            (Point(9, 10), 10),
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
