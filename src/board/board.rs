use super::line::*;

pub const BOARD_SIZE: u8 = 15;
const N: u8 = BOARD_SIZE;
const M: u8 = N * 2 - 1; // 29

type OrthogonalLines = [Line; N as usize];
type DiagonalLines = [Line; M as usize];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Board {
    hlines: OrthogonalLines,
    vlines: OrthogonalLines,
    alines: DiagonalLines,
    dlines: DiagonalLines,
}

impl Board {
    pub fn new() -> Board {
        Board {
            hlines: Board::orthogonal_lines(),
            vlines: Board::orthogonal_lines(),
            alines: Board::diagonal_lines(),
            dlines: Board::diagonal_lines(),
        }
    }

    pub fn put(&self, black: bool, p: Point) -> Board {
        // Use const generics in the future.
        // fn put_lines<const size: usize>(lines: &[Line; size]) -> [Line; size]
        let mut vlines = self.vlines.clone();
        let vidx = p.to_index(Direction::Vertical);
        vlines[vidx.i as usize] = self.vlines[vidx.i as usize].put(black, vidx.j);
        let mut hlines = self.hlines.clone();
        let hidx = p.to_index(Direction::Horizontal);
        hlines[hidx.i as usize] = self.hlines[hidx.i as usize].put(black, hidx.j);
        let mut alines = self.alines.clone();
        let aidx = p.to_index(Direction::Ascending);
        alines[aidx.i as usize] = self.alines[aidx.i as usize].put(black, aidx.j);
        let mut dlines = self.dlines.clone();
        let didx = p.to_index(Direction::Descending);
        dlines[didx.i as usize] = self.dlines[didx.i as usize].put(black, didx.j);
        Board {
            vlines: vlines,
            hlines: hlines,
            alines: alines,
            dlines: dlines,
        }
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
        let viter = self
            .vlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Ascending, i as u8, l));
        let diter = self
            .dlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Descending, i as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
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

    fn orthogonal_lines() -> OrthogonalLines {
        [Line::new(N); N as usize]
    }

    fn diagonal_lines() -> DiagonalLines {
        [
            Line::new(1),
            Line::new(2),
            Line::new(3),
            Line::new(4),
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
            Line::new(4),
            Line::new(3),
            Line::new(2),
            Line::new(1),
        ]
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
