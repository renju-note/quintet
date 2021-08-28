use super::bits::*;
use super::coordinates::*;
use super::forbidden::*;
use super::row::*;
use super::square::*;
use std::fmt;

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
