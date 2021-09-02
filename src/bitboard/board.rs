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

    pub fn from_points(blacks: &[Point], whites: &[Point]) -> Board {
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

    pub fn forbidden(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden(&self.square, p)
    }

    pub fn forbiddens(&self) -> Vec<(ForbiddenKind, Point)> {
        forbiddens(&self.square)
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
