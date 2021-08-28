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

    pub fn put(&mut self, black: bool, p: Point) {
        self.square.put(black, p);
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<RowSegment> {
        self.square.rows(black, kind)
    }

    pub fn rows_on(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<RowSegment> {
        self.square.rows_on(p, black, kind)
    }

    pub fn row_eyes(&mut self, black: bool, kind: RowKind) -> Vec<Point> {
        self.square.row_eyes(black, kind)
    }

    pub fn row_eyes_along(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<Point> {
        self.square.row_eyes_along(p, black, kind)
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let codes = s.trim().split('/').collect::<Vec<_>>();
        if codes.len() != 2 {
            return Err("Unknown format.".to_string());
        }
        let blacks = parse_points(codes[0])?;
        let whites = parse_points(codes[1])?;
        let mut board = Board::new();
        for p in blacks {
            board.put(true, p);
        }
        for p in whites {
            board.put(false, p);
        }
        Ok(board)
    }
}

pub fn parse_points(s: &str) -> Result<Vec<Point>, String> {
    s.split(',').map(|m| m.parse::<Point>()).collect()
}
