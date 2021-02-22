use super::line::*;
use super::row::*;
use std::collections::HashSet;

pub const BOARD_SIZE: u8 = 15;
const O_LINE_NUM: u8 = BOARD_SIZE;
const D_LINE_NUM: u8 = BOARD_SIZE * 2 - 1 - (4 * 2); // 21
const N: u8 = BOARD_SIZE - 1;

type OrthogonalLines = [Line; O_LINE_NUM as usize];
type DiagonalLines = [Line; D_LINE_NUM as usize];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

#[derive(Clone)]
pub struct Board {
    vlines: OrthogonalLines,
    hlines: OrthogonalLines,
    alines: DiagonalLines,
    dlines: DiagonalLines,
}

impl Board {
    pub fn new() -> Board {
        Board {
            vlines: orthogonal_lines(),
            hlines: orthogonal_lines(),
            alines: diagonal_lines(),
            dlines: diagonal_lines(),
        }
    }

    pub fn put(&mut self, black: bool, p: Point) {
        // Use const generics in the future.
        // fn put_lines<const size: usize>(lines: &[Line; size]) -> [Line; size]
        let vidx = p.to_index(Direction::Vertical);
        self.vlines[vidx.i as usize].put(black, vidx.j);

        let hidx = p.to_index(Direction::Horizontal);
        self.hlines[hidx.i as usize].put(black, hidx.j);

        let aidx = p.to_index(Direction::Ascending);
        if 4 <= aidx.i && aidx.i < D_LINE_NUM + 4 {
            let i = (aidx.i - 4) as usize;
            self.alines[i].put(black, aidx.j);
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < D_LINE_NUM + 4 {
            let i = (didx.i - 4) as usize;
            self.dlines[i].put(black, didx.j);
        }
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = vec![];
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines(checker) {
            let lrows = l.rows(black, kind);
            let mut brows = lrows.map(|lr| BoardRow::from(lr, d, i)).collect::<Vec<_>>();
            result.append(&mut brows);
        }
        result
    }

    pub fn rows_on(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = vec![];
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines_along(p, checker) {
            let lrows = l.rows(black, kind);
            let mut brows = lrows
                .map(|lr| BoardRow::from(lr, d, i))
                .filter(|br| br.overlap(p))
                .collect::<Vec<_>>();
            result.append(&mut brows);
        }
        result
    }

    pub fn row_eyes(&mut self, black: bool, kind: RowKind) -> HashSet<Point> {
        let mut result = HashSet::new();
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines(checker) {
            let lrows = l.rows(black, kind);
            let brows = lrows.map(|lr| BoardRow::from(lr, d, i));
            for brow in brows {
                brow.eye1.map(|e| result.insert(e));
                brow.eye2.map(|e| result.insert(e));
            }
        }
        result
    }

    pub fn row_eyes_along(&mut self, p: Point, black: bool, kind: RowKind) -> HashSet<Point> {
        let mut result = HashSet::new();
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines_along(p, checker) {
            let lrows = l.rows(black, kind);
            let brows = lrows.map(|lr| BoardRow::from(lr, d, i));
            for brow in brows {
                brow.eye1.map(|e| result.insert(e));
                brow.eye2.map(|e| result.insert(e));
            }
        }
        result
    }

    fn iter_mut_lines(
        &mut self,
        checker: Checker,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let viter = self
            .vlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(checker))
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(checker))
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(checker))
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l));
        let diter = self
            .dlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(checker))
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
    }

    fn iter_mut_lines_along(
        &mut self,
        p: Point,
        checker: Checker,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let mut result = vec![];
        let vidx = p.to_index(Direction::Vertical);
        let vline = &mut self.vlines[vidx.i as usize];
        if vline.check(checker) {
            result.push((Direction::Vertical, vidx.i, vline))
        }

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &mut self.hlines[hidx.i as usize];
        if hline.check(checker) {
            result.push((Direction::Horizontal, hidx.i, hline))
        }

        let aidx = p.to_index(Direction::Ascending);
        if 4 <= aidx.i && aidx.i < D_LINE_NUM + 4 {
            let aline = &mut self.alines[(aidx.i - 4) as usize];
            if aline.check(checker) {
                result.push((Direction::Ascending, aidx.i, aline));
            }
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < D_LINE_NUM + 4 {
            let dline = &mut self.dlines[(didx.i - 4) as usize];
            if dline.check(checker) {
                result.push((Direction::Descending, didx.i, dline));
            }
        }

        result.into_iter()
    }

    pub fn to_string(&self) -> String {
        let mut result = self
            .hlines
            .iter()
            .rev()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        result.push('\n');
        result
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.x, self.y);
        match direction {
            Direction::Vertical => Index { i: x, j: y },
            Direction::Horizontal => Index { i: y, j: x },
            Direction::Ascending => {
                let i = x + N - y;
                let j = if i < N { x } else { y };
                Index { i: i, j: j }
            }
            Direction::Descending => {
                let i = x + y;
                let j = if i < N { x } else { N - y };
                Index { i: i, j: j }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub i: u8,
    pub j: u8,
}

impl Index {
    pub fn to_point(&self, direction: Direction) -> Point {
        let (i, j) = (self.i, self.j);
        match direction {
            Direction::Vertical => Point { x: i, y: j },
            Direction::Horizontal => Point { x: j, y: i },
            Direction::Ascending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { N - i + j } else { j };
                Point { x: x, y: y }
            }
            Direction::Descending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { i - j } else { N - j };
                Point { x: x, y: y }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoardRow {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
}

impl BoardRow {
    pub fn from(r: &Row, d: Direction, i: u8) -> BoardRow {
        BoardRow {
            direction: d,
            start: Index { i: i, j: r.start }.to_point(d),
            end: Index { i: i, j: r.end }.to_point(d),
            eye1: r.eye1.map(|e| Index { i: i, j: e }.to_point(d)),
            eye2: r.eye2.map(|e| Index { i: i, j: e }.to_point(d)),
        }
    }

    pub fn overlap(&self, p: Point) -> bool {
        let (px, py) = (p.x, p.y);
        let (sx, sy) = (self.start.x, self.start.y);
        let (ex, ey) = (self.end.x, self.end.y);
        match self.direction {
            Direction::Vertical => px == sx && bw(sy, py, ey),
            Direction::Horizontal => py == sy && bw(sx, px, ex),
            Direction::Ascending => bw(sx, px, ex) && bw(sy, py, ey) && px - sx == py - sy,
            Direction::Descending => bw(sx, px, ex) && bw(ey, py, sy) && px - sx == sy - py,
        }
    }
}

fn orthogonal_lines() -> OrthogonalLines {
    [
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
        Line::new(15),
    ]
}

fn diagonal_lines() -> DiagonalLines {
    [
        Line::new(5),
        Line::new(6),
        Line::new(7),
        Line::new(8),
        Line::new(9),
        Line::new(10),
        Line::new(11),
        Line::new(12),
        Line::new(13),
        Line::new(14),
        Line::new(15),
        Line::new(14),
        Line::new(13),
        Line::new(12),
        Line::new(11),
        Line::new(10),
        Line::new(9),
        Line::new(8),
        Line::new(7),
        Line::new(6),
        Line::new(5),
    ]
}

fn bw(a: u8, x: u8, b: u8) -> bool {
    a <= x && x <= b
}
