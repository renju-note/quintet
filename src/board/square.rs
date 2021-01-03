use super::line::*;

pub const BOAD_SIZE: u8 = 15;
const N: u8 = BOAD_SIZE;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Index {
    pub i: u8,
    pub j: u8,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Facet {
    pub direction: Direction,
    pub lines: Vec<Line>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Square {
    pub size: u8,
    pub facets: Vec<Facet>,
}

impl Square {
    pub fn new() -> Square {
        Square {
            size: N,
            facets: vec![
                Facet {
                    direction: Direction::Vertical,
                    lines: Square::new_orthogonal_lines(),
                },
                Facet {
                    direction: Direction::Horizontal,
                    lines: Square::new_orthogonal_lines(),
                },
                Facet {
                    direction: Direction::Ascending,
                    lines: Square::new_diagonal_lines(),
                },
                Facet {
                    direction: Direction::Descending,
                    lines: Square::new_diagonal_lines(),
                },
            ],
        }
    }

    pub fn put(&self, black: bool, p: Point) -> Square {
        let facets: Vec<Facet> = self
            .facets
            .iter()
            .map(|facet| {
                let idx = to_index(p, facet.direction);
                let i = idx.i as usize;
                let l = facet.lines.len();
                let new_line = facet.lines[i].put(black, idx.j);
                let mut new_lines = Vec::new();
                new_lines.append(&mut facet.lines[..i].to_vec());
                new_lines.push(new_line);
                new_lines.append(&mut facet.lines[i + 1..l].to_vec());
                Facet {
                    direction: facet.direction,
                    lines: new_lines,
                }
            })
            .collect();
        Square {
            size: self.size,
            facets: facets,
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = self.facets[1]
            .lines
            .iter()
            .rev()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        result.push('\n');
        result
    }

    fn new_orthogonal_lines() -> Vec<Line> {
        (0..N).map(|_| Line::new(N)).collect()
    }

    fn new_diagonal_lines() -> Vec<Line> {
        let m = N * 2 - 1;
        (0..m)
            .map(|i| {
                let size = if i < N { i + 1 } else { m - i };
                Line::new(size)
            })
            .collect()
    }
}

pub fn to_index(p: Point, direction: Direction) -> Index {
    let (x, y) = (p.x, p.y);
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

pub fn to_point(idx: Index, direction: Direction) -> Point {
    let (i, j) = (idx.i, idx.j);
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
