use super::forbidden::*;
use super::point::*;
use super::row::*;
use super::square::*;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct Board {
    square: Square,
}

impl Board {
    pub fn new() -> Board {
        Board {
            square: Square::new(),
        }
    }

    pub fn from_points(blacks: Points, whites: Points) -> Board {
        let square = Square::from_points(blacks, whites);
        Board { square: square }
    }

    pub fn put(&mut self, player: Player, p: Point) {
        self.square.put(player, p);
    }

    pub fn rows(&self, player: Player, kind: RowKind) -> Vec<RowSegment> {
        self.square.rows(player, kind)
    }

    pub fn rows_on(&self, player: Player, kind: RowKind, p: Point) -> Vec<RowSegment> {
        self.square.rows_on(player, kind, p)
    }

    pub fn row_eyes(&self, player: Player, kind: RowKind) -> Vec<Point> {
        self.square.row_eyes(player, kind)
    }

    pub fn row_eyes_along(&self, player: Player, kind: RowKind, p: Point) -> Vec<Point> {
        self.square.row_eyes_along(player, kind, p)
    }

    pub fn forbiddens(&self) -> Vec<(ForbiddenKind, Point)> {
        forbiddens(&self.square)
    }

    pub fn forbidden(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden(&self.square, p)
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
        Ok(Board { square: square })
    }
}

#[cfg(test)]
mod tests {
    use super::Direction::*;
    use super::ForbiddenKind::*;
    use super::Player::*;
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test() -> Result<(), String> {
        let mut board = Board::new();
        board.put(Black, Point(7, 7));
        board.put(White, Point(7, 8));
        board.put(Black, Point(9, 9));
        board.put(White, Point(8, 8));
        board.put(Black, Point(6, 8));
        board.put(White, Point(8, 6));
        board.put(Black, Point(6, 9));
        board.put(White, Point(8, 9));
        board.put(Black, Point(8, 7));
        board.put(White, Point(5, 6));

        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ------o-xo-----
            ------oxx------
            -------oo------
            -----x--x------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        ",
        );
        assert_eq!(board.to_string(), expected);

        let black_twos = [
            RowSegment::new(
                Vertical,
                Point(6, 5),
                Point(6, 10),
                Some(Point(6, 6)),
                Some(Point(6, 7)),
            ),
            RowSegment::new(
                Vertical,
                Point(6, 6),
                Point(6, 11),
                Some(Point(6, 7)),
                Some(Point(6, 10)),
            ),
            RowSegment::new(
                Vertical,
                Point(6, 7),
                Point(6, 12),
                Some(Point(6, 10)),
                Some(Point(6, 11)),
            ),
            RowSegment::new(
                Horizontal,
                Point(4, 7),
                Point(9, 7),
                Some(Point(5, 7)),
                Some(Point(6, 7)),
            ),
            RowSegment::new(
                Horizontal,
                Point(5, 7),
                Point(10, 7),
                Some(Point(6, 7)),
                Some(Point(9, 7)),
            ),
            RowSegment::new(
                Horizontal,
                Point(6, 7),
                Point(11, 7),
                Some(Point(9, 7)),
                Some(Point(10, 7)),
            ),
        ];
        let white_twos = [
            RowSegment::new(
                Horizontal,
                Point(4, 6),
                Point(9, 6),
                Some(Point(6, 6)),
                Some(Point(7, 6)),
            ),
            RowSegment::new(
                Ascending,
                Point(6, 7),
                Point(11, 12),
                Some(Point(9, 10)),
                Some(Point(10, 11)),
            ),
        ];
        let white_threes = [RowSegment::new(
            Ascending,
            Point(4, 5),
            Point(9, 10),
            Some(Point(6, 7)),
            None,
        )];

        assert_eq!(board.rows(Black, Two), black_twos);
        assert_eq!(board.rows(White, Two), white_twos);
        assert_eq!(board.rows(White, Three), white_threes);
        assert_eq!(
            board.rows_on(Black, Two, Point(7, 7)),
            [
                black_twos[3].clone(),
                black_twos[4].clone(),
                black_twos[5].clone()
            ]
        );
        assert_eq!(
            board.row_eyes(Black, Two),
            [
                Point(5, 7),
                Point(6, 6),
                Point(6, 7),
                Point(6, 10),
                Point(6, 11),
                Point(9, 7),
                Point(10, 7)
            ]
        );
        assert_eq!(
            board.row_eyes_along(Black, Two, Point(6, 6)),
            [Point(6, 6), Point(6, 7), Point(6, 10), Point(6, 11)]
        );

        assert_eq!(board.forbiddens(), [(DoubleThree, Point(6, 7))]);
        assert_eq!(board.forbidden(Point(6, 7)), Some(DoubleThree));

        Ok(())
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
