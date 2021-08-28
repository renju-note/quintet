use super::bits::*;
use super::coordinates::*;
use super::line::*;
use super::row::*;
use std::fmt;

#[derive(Clone)]
pub struct Square {
    vlines: OrthogonalLines,
    hlines: OrthogonalLines,
    alines: DiagonalLines,
    dlines: DiagonalLines,
}

#[derive(Debug, Clone)]
pub struct RowSegment {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
}

impl Square {
    pub fn new() -> Square {
        Square {
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

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<RowSegment> {
        self.iter_mut_lines()
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |r| RowSegment::from(&r, d, i))
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn rows_on(&mut self, p: Point, black: bool, kind: RowKind) -> Vec<RowSegment> {
        self.iter_mut_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(black, kind)
                    .into_iter()
                    .map(move |r| RowSegment::from(&r, d, i))
                    .filter(|r| r.overlap(p))
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
                    .map(move |r| RowSegment::from(&r, d, i))
                    .map(|r| r.into_iter_eyes())
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
                    .map(move |r| RowSegment::from(&r, d, i))
                    .map(|r| r.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_unstable();
        result.dedup();
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

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .hlines
            .iter()
            .rev()
            .map(|l| format!("{}", l))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", s)
    }
}

impl RowSegment {
    pub fn from(r: &Row, d: Direction, i: u8) -> RowSegment {
        RowSegment {
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

const O_LINE_NUM: u8 = BOARD_SIZE;
const D_LINE_NUM: u8 = BOARD_SIZE * 2 - 1 - (4 * 2); // 21

type OrthogonalLines = [Line; O_LINE_NUM as usize];
type DiagonalLines = [Line; D_LINE_NUM as usize];

fn orthogonal_lines() -> OrthogonalLines {
    [
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
    ]
}

fn diagonal_lines() -> DiagonalLines {
    [
        Line::new(BOARD_SIZE - 10),
        Line::new(BOARD_SIZE - 9),
        Line::new(BOARD_SIZE - 8),
        Line::new(BOARD_SIZE - 7),
        Line::new(BOARD_SIZE - 6),
        Line::new(BOARD_SIZE - 5),
        Line::new(BOARD_SIZE - 4),
        Line::new(BOARD_SIZE - 3),
        Line::new(BOARD_SIZE - 2),
        Line::new(BOARD_SIZE - 1),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE - 1),
        Line::new(BOARD_SIZE - 2),
        Line::new(BOARD_SIZE - 3),
        Line::new(BOARD_SIZE - 4),
        Line::new(BOARD_SIZE - 5),
        Line::new(BOARD_SIZE - 6),
        Line::new(BOARD_SIZE - 7),
        Line::new(BOARD_SIZE - 8),
        Line::new(BOARD_SIZE - 9),
        Line::new(BOARD_SIZE - 10),
    ]
}

fn bw(a: u8, x: u8, b: u8) -> bool {
    a <= x && x <= b
}
