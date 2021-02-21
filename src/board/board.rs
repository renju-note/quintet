use super::line::*;
use super::row::*;
use std::collections::HashSet;

pub const BOARD_SIZE: u8 = 15;
const N: u8 = BOARD_SIZE;
const M: u8 = N * 2 - 1 - (4 * 2); // 21

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
        min_bcount: u8,
        min_wcount: u8,
        min_ncount: u8,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let viter = self
            .vlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l));
        let diter = self
            .dlines
            .iter_mut()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
    }

    pub fn iter_mut_lines_on(
        &mut self,
        p: &Point,
        min_bcount: u8,
        min_wcount: u8,
        min_ncount: u8,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let mut result = vec![];

        let vidx = p.to_index(Direction::Vertical);
        let vline = &mut self.vlines[vidx.i as usize];
        if vline.check(min_bcount, min_wcount, min_ncount) {
            result.push((Direction::Vertical, vidx.i, vline))
        }

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &mut self.hlines[hidx.i as usize];
        if hline.check(min_bcount, min_wcount, min_ncount) {
            result.push((Direction::Horizontal, hidx.i, hline))
        }

        let aidx = p.to_index(Direction::Ascending);
        if 4 <= aidx.i && aidx.i < M + 4 {
            let aline = &mut self.alines[(aidx.i - 4) as usize];
            if aline.check(min_bcount, min_wcount, min_ncount) {
                result.push((Direction::Ascending, aidx.i, aline));
            }
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < M + 4 {
            let dline = &mut self.dlines[(didx.i - 4) as usize];
            if dline.check(min_bcount, min_wcount, min_ncount) {
                result.push((Direction::Descending, didx.i, dline));
            }
        }

        result.into_iter()
    }

    pub fn lines(
        &self,
        min_bcount: u8,
        min_wcount: u8,
        min_ncount: u8,
    ) -> Vec<(Direction, u8, &Line)> {
        let mut result = vec![];
        let mut vresult = self
            .vlines
            .iter()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Vertical, i as u8, l))
            .collect();
        result.append(&mut vresult);
        let mut hresult = self
            .hlines
            .iter()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Horizontal, i as u8, l))
            .collect();
        result.append(&mut hresult);
        let mut aresult = self
            .alines
            .iter()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l))
            .collect();
        result.append(&mut aresult);
        let mut dresult = self
            .dlines
            .iter()
            .enumerate()
            .filter(move |(_, l)| l.check(min_bcount, min_wcount, min_ncount))
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l))
            .collect();
        result.append(&mut dresult);
        result
    }

    pub fn lines_on(
        &self,
        p: &Point,
        min_bcount: u8,
        min_wcount: u8,
        min_ncount: u8,
    ) -> Vec<(Direction, u8, &Line)> {
        let mut result = vec![];

        let vidx = p.to_index(Direction::Vertical);
        let vline = &self.vlines[vidx.i as usize];
        if vline.check(min_bcount, min_wcount, min_ncount) {
            result.push((Direction::Vertical, vidx.i, vline))
        }

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &self.hlines[hidx.i as usize];
        if hline.check(min_bcount, min_wcount, min_ncount) {
            result.push((Direction::Horizontal, hidx.i, hline))
        }

        let aidx = p.to_index(Direction::Ascending);
        if 4 <= aidx.i && aidx.i < M + 4 {
            let aline = &self.alines[(aidx.i - 4) as usize];
            if aline.check(min_bcount, min_wcount, min_ncount) {
                result.push((Direction::Ascending, aidx.i, aline));
            }
        }

        let didx = p.to_index(Direction::Descending);
        if 4 <= didx.i && didx.i < M + 4 {
            let dline = &self.dlines[(didx.i - 4) as usize];
            if dline.check(min_bcount, min_wcount, min_ncount) {
                result.push((Direction::Descending, didx.i, dline));
            }
        }

        result
    }

    pub fn row_eyes(&mut self, black: bool, kind: RowKind) -> Vec<Point> {
        let mut result = vec![];
        let (min_scount, min_ncount) = kind.min_scount_ncount();
        let min_bcount = if black { min_scount } else { 0 };
        let min_wcount = if black { 0 } else { min_scount };
        for (d, i, l) in self.iter_mut_lines(min_bcount, min_wcount, min_ncount) {
            let is = l.row_eyes(black, kind);
            let ps = is.iter().map(|&j| Index { i: i, j: j }.to_point(d));
            result.append(&mut ps.collect::<Vec<_>>());
        }
        result
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn row_eyes_on(&mut self, p: &Point, black: bool, kind: RowKind) -> Vec<Point> {
        let mut result = vec![];
        let (min_scount, min_ncount) = kind.min_scount_ncount();
        let min_bcount = if black { min_scount } else { 0 };
        let min_wcount = if black { 0 } else { min_scount };
        for (d, i, l) in self.iter_mut_lines_on(p, min_bcount, min_wcount, min_ncount) {
            let is = l.row_eyes(black, kind);
            let ps = is.iter().map(|&j| Index { i: i, j: j }.to_point(d));
            result.append(&mut ps.collect::<Vec<_>>());
        }
        result
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

type OrthogonalLines = [Line; N as usize];
type DiagonalLines = [Line; M as usize];

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
