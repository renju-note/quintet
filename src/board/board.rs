use super::line::*;
use super::row::*;

pub const BOARD_SIZE: u8 = 15;

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub i: u8,
    pub j: u8,
}

#[derive(Debug, Clone)]
pub struct BoardRow {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
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
        let vidx = p.to_index(Direction::Vertical);
        self.vlines[vidx.i as usize].put(black, vidx.j);

        let hidx = p.to_index(Direction::Horizontal);
        self.hlines[hidx.i as usize].put(black, hidx.j);

        let aidx = p.to_index(Direction::Ascending);
        if bw(4, aidx.i, D_LINE_NUM + 3) {
            let i = (aidx.i - 4) as usize;
            self.alines[i].put(black, aidx.j);
        }

        let didx = p.to_index(Direction::Descending);
        if bw(4, didx.i, D_LINE_NUM + 3) {
            let i = (didx.i - 4) as usize;
            self.dlines[i].put(black, didx.j);
        }
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<BoardRow> {
        self.iter_mut_lines()
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |lr| BoardRow::from(&lr, d, i))
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn rows_on(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<BoardRow> {
        self.iter_mut_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |lr| BoardRow::from(&lr, d, i))
                    .filter(|br| br.overlap(p))
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn row_eyes(&mut self, black: bool, kind: RowKind) -> Vec<Point> {
        let mut result = self
            .iter_mut_lines()
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |lr| BoardRow::from(&lr, d, i))
                    .map(|br| br.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_unstable();
        result.dedup();
        result
    }

    pub fn row_eyes_along(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<Point> {
        let mut result = self
            .iter_mut_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |lr| BoardRow::from(&lr, d, i))
                    .map(|br| br.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_unstable();
        result.dedup();
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

    fn iter_mut_lines(&mut self) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let viter = self
            .vlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l));
        let diter = self
            .dlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
    }

    fn iter_mut_lines_along(
        &mut self,
        p: Point,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let vidx = p.to_index(Direction::Vertical);
        let vline = &mut self.vlines[vidx.i as usize];
        let viter = Some((Direction::Vertical, vidx.i, vline)).into_iter();

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &mut self.hlines[hidx.i as usize];
        let hiter = Some((Direction::Horizontal, hidx.i, hline)).into_iter();

        let aidx = p.to_index(Direction::Ascending);
        let aiter = if bw(4, aidx.i, D_LINE_NUM + 3) {
            let aline = &mut self.alines[(aidx.i - 4) as usize];
            Some((Direction::Ascending, aidx.i, aline))
        } else {
            None
        }
        .into_iter();

        let didx = p.to_index(Direction::Descending);
        let diter = if bw(4, didx.i, D_LINE_NUM + 3) {
            let dline = &mut self.dlines[(didx.i - 4) as usize];
            Some((Direction::Descending, didx.i, dline))
        } else {
            None
        }
        .into_iter();

        viter.chain(hiter).chain(aiter).chain(diter)
    }
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

    pub fn into_iter_eyes(&self) -> impl IntoIterator<Item = Point> {
        self.eye1.into_iter().chain(self.eye2.into_iter())
    }
}

const N: u8 = BOARD_SIZE - 1;
const O_LINE_NUM: u8 = BOARD_SIZE;
const D_LINE_NUM: u8 = BOARD_SIZE * 2 - 1 - (4 * 2); // 21

type OrthogonalLines = [Line; O_LINE_NUM as usize];
type DiagonalLines = [Line; D_LINE_NUM as usize];

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
