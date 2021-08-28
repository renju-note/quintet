use super::bits::*;
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

    pub fn put(&mut self, player: Player, p: Point) {
        self.square.put(player, p);
    }

    pub fn rows(&mut self, player: Player, kind: RowKind) -> Vec<RowSegment> {
        self.square.rows(player, kind)
    }

    pub fn rows_on(&mut self, player: Player, kind: RowKind, p: Point) -> Vec<RowSegment> {
        self.square.rows_on(player, kind, p)
    }

    pub fn row_eyes(&mut self, player: Player, kind: RowKind) -> Vec<Point> {
        self.square.row_eyes(player, kind)
    }

    pub fn row_eyes_along(&mut self, player: Player, kind: RowKind, p: Point) -> Vec<Point> {
        self.square.row_eyes_along(player, kind, p)
    }

    pub fn forbidden(&self, p: Point) -> Option<ForbiddenKind> {
        forbidden(&self.square, p)
    }

    pub fn forbiddens(&self) -> Vec<(ForbiddenKind, Point)> {
        (0..BOARD_SIZE)
            .flat_map(|x| (0..BOARD_SIZE).map(move |y| Point { x: x, y: y }))
            .map(|p| (self.forbidden(p), p))
            .filter(|(k, _)| k.is_some())
            .map(|(k, p)| (k.unwrap(), p))
            .collect()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.square)
    }
}

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let codes = s.trim().split('/').collect::<Vec<_>>();
        if codes.len() != 2 {
            return Err("Unknown format.");
        }
        let blacks = parse_points(codes[0])?;
        let whites = parse_points(codes[1])?;
        let mut board = Board::new();
        for p in blacks {
            board.put(Player::Black, p);
        }
        for p in whites {
            board.put(Player::White, p);
        }
        Ok(board)
    }
}

pub fn parse_points(s: &str) -> Result<Vec<Point>, &'static str> {
    s.split(',').map(|m| m.parse::<Point>()).collect()
}
