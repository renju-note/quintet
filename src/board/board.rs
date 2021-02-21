use super::bits::*;
use super::line::*;
use super::row::*;

pub const BOARD_SIZE: u8 = 15;
const N: u8 = BOARD_SIZE;
const M: u8 = N * 2 - 1 - (4 * 2); // 21

type OrthogonalLines = [Line; N as usize];
type DiagonalLines = [Line; M as usize];

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

    pub fn put(&mut self, black: bool, p: &Point) {
        // Use const generics in the future.
        // fn put_lines<const size: usize>(lines: &[Line; size]) -> [Line; size]
        let vidx = p.to_index(Direction::Vertical);
        self.vlines[vidx.i as usize].put(black, vidx.j);

        let hidx = p.to_index(Direction::Horizontal);
        self.hlines[hidx.i as usize].put(black, hidx.j);

        let aidx = p.to_index(Direction::Ascending);
        if 4 <= aidx.i && aidx.i < M + 4 {
            let i = (aidx.i - 4) as usize;
            self.alines[i].put(black, aidx.j);
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < M + 4 {
            let i = (didx.i - 4) as usize;
            self.dlines[i].put(black, didx.j);
        }
    }

    pub fn iter_mut_lines(
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

    pub fn iter_mut_lines_on(
        &mut self,
        p: &Point,
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
        if 4 <= aidx.i && aidx.i < M + 4 {
            let aline = &mut self.alines[(aidx.i - 4) as usize];
            if aline.check(checker) {
                result.push((Direction::Ascending, aidx.i, aline));
            }
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < M + 4 {
            let dline = &mut self.dlines[(didx.i - 4) as usize];
            if dline.check(checker) {
                result.push((Direction::Descending, didx.i, dline));
            }
        }

        result.into_iter()
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = vec![];
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines(checker) {
            let lrows = l.rows(black, kind);
            let mut brows = lrows
                .iter()
                .map(|lr| BoardRow::from(lr, d, i))
                .collect::<Vec<_>>();
            result.append(&mut brows);
        }
        result
    }

    pub fn rows_on(&mut self, p: &Point, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = vec![];
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines_on(p, checker) {
            let lrows = l.rows(black, kind);
            let mut brows = lrows
                .iter()
                .map(|lr| BoardRow::from(lr, d, i))
                .filter(|br| br.overlap(p))
                .collect::<Vec<_>>();
            result.append(&mut brows);
        }
        result
    }

    pub fn row_eyes(&mut self, black: bool, kind: RowKind) -> PointsMemory {
        let mut result = PointsMemory::default();
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines(checker) {
            let lrows = l.rows(black, kind);
            let brows = lrows.iter().map(|lr| BoardRow::from(lr, d, i));
            for brow in brows {
                brow.eye1.map(|e| result.set(e));
                brow.eye2.map(|e| result.set(e));
            }
        }
        result
    }

    pub fn row_eyes_on(&mut self, p: &Point, black: bool, kind: RowKind) -> PointsMemory {
        let mut result = PointsMemory::default();
        let checker = kind.checker(black);
        for (d, i, l) in self.iter_mut_lines_on(p, checker) {
            let lrows = l.rows(black, kind);
            let brows = lrows
                .iter()
                .map(|lr| BoardRow::from(lr, d, i))
                .filter(|br| br.overlap(p));
            for brow in brows {
                brow.eye1.map(|e| result.set(e));
                brow.eye2.map(|e| result.set(e));
            }
        }
        result
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.x, self.y);
        match direction {
            Direction::Vertical => Index { i: x - 1, j: y - 1 },
            Direction::Horizontal => Index { i: y - 1, j: x - 1 },
            Direction::Ascending => {
                let i = x - 1 + N - y;
                let j = if i < N { x - 1 } else { y - 1 };
                Index { i: i, j: j }
            }
            Direction::Descending => {
                let i = x - 1 + y - 1;
                let j = if i < N { x - 1 } else { N - y };
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
            Direction::Vertical => Point { x: i + 1, y: j + 1 },
            Direction::Horizontal => Point { x: j + 1, y: i + 1 },
            Direction::Ascending => {
                let x = if i < N { j + 1 } else { i + 1 + j + 1 - N };
                let y = if i < N { N - (i + 1) + j + 1 } else { j + 1 };
                Point { x: x, y: y }
            }
            Direction::Descending => {
                let x = if i < N { j + 1 } else { i + 1 + j + 1 - N };
                let y = if i < N { i + 1 - j } else { N - j };
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
    pub fn from(r: &LineRow, d: Direction, i: u8) -> BoardRow {
        BoardRow {
            direction: d,
            start: Index { i: i, j: r.start }.to_point(d),
            end: Index { i: i, j: r.end }.to_point(d),
            eye1: r.eye1.map(|e| Index { i: i, j: e }.to_point(d)),
            eye2: r.eye2.map(|e| Index { i: i, j: e }.to_point(d)),
        }
    }

    pub fn overlap(&self, p: &Point) -> bool {
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

#[derive(Default)]
pub struct PointsMemory {
    pub count: u8,
    memory: [Bits; N as usize],
}

impl PointsMemory {
    pub fn set(&mut self, p: Point) {
        if !self.has(p) {
            self.count += 1;
            self.memory[(p.x - 1) as usize] |= 0b1 << (p.y - 1);
        }
    }

    pub fn has(&self, p: Point) -> bool {
        self.memory[(p.x - 1) as usize] & 0b1 << (p.y - 1) != 0b0
    }

    pub fn to_vec(&self) -> Vec<Point> {
        let mut result = vec![];
        for x in 0..N {
            let xs = self.memory[x as usize];
            for y in 0..N {
                if xs & (0b1 << y) != 0b0 {
                    result.push(Point { x: x + 1, y: y + 1 });
                }
            }
        }
        result
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
