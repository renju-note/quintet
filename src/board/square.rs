use super::line::*;
use super::row::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Point {
    x: u32,
    y: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Index {
    i: u32,
    j: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

struct Facet {
    direction: Direction,
    lines: Vec<Line>,
}

struct Square {
    size: u32,
    facets: Vec<Facet>,
}

struct SquareRow {
    kind: RowKind,
    direction: Direction,
    start: Point,
    end: Point,
    eyes: Vec<Point>,
}

impl Square {
    pub fn new(size: u32) -> Result<Square, String> {
        if size > INT_SIZE {
            return Err(String::from("Wrong size"));
        }
        Ok(Square {
            size: size,
            facets: vec![
                Facet {
                    direction: Direction::Vertical,
                    lines: new_orthogonal_lines(size),
                },
                Facet {
                    direction: Direction::Horizontal,
                    lines: new_orthogonal_lines(size),
                },
                Facet {
                    direction: Direction::Ascending,
                    lines: new_diagonal_lines(size),
                },
                Facet {
                    direction: Direction::Descending,
                    lines: new_diagonal_lines(size),
                },
            ],
        })
    }

    pub fn put(&self, black: bool, p: Point) -> Square {
        let facets: Vec<Facet> = self
            .facets
            .iter()
            .map(|facet| {
                let idx = to_index(p, facet.direction, self.size);
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

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<SquareRow> {
        self.facets
            .iter()
            .flat_map::<Vec<_>, _>(|facet| {
                facet
                    .lines
                    .iter()
                    .enumerate()
                    .flat_map::<Vec<_>, _>(|(i, line)| {
                        let i: u32 = i as u32;
                        line.rows(black, kind)
                            .iter()
                            .map(|row| SquareRow {
                                kind: kind,
                                direction: facet.direction,
                                start: to_point(
                                    Index { i: i, j: row.start },
                                    facet.direction,
                                    self.size,
                                ),
                                end: to_point(
                                    Index {
                                        i: i,
                                        j: row.start + row.size - 1,
                                    },
                                    facet.direction,
                                    self.size,
                                ),
                                eyes: row
                                    .eyes
                                    .iter()
                                    .map(|j| {
                                        to_point(Index { i: i, j: *j }, facet.direction, self.size)
                                    })
                                    .collect(),
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect::<Vec<_>>()
    }

    pub fn to_string(&self) -> String {
        let hfacet = self
            .facets
            .iter()
            .find(|facet| facet.direction == Direction::Horizontal);
        match hfacet {
            Some(facet) => {
                let mut result = String::new();
                for l in facet.lines.iter().rev() {
                    result.push_str(&l.to_string());
                    result.push('\n')
                }
                result
            }
            None => String::new(),
        }
    }
}

fn to_index(p: Point, direction: Direction, size: u32) -> Index {
    let (x, y) = (p.x, p.y);
    match direction {
        Direction::Vertical => Index { i: x - 1, j: y - 1 },
        Direction::Horizontal => Index { i: y - 1, j: x - 1 },
        Direction::Ascending => {
            let i = x - 1 + (size - y);
            let j = if i < size { x - 1 } else { y - 1 };
            Index { i: i, j: j }
        }
        Direction::Descending => {
            let i = x - 1 + (y - 1);
            let j = if i < size { x - 1 } else { size - y };
            Index { i: i, j: j }
        }
    }
}

fn to_point(idx: Index, direction: Direction, size: u32) -> Point {
    let (i, j) = (idx.i, idx.j);
    match direction {
        Direction::Vertical => Point { x: i + 1, y: j + 1 },
        Direction::Horizontal => Point { x: j + 1, y: i + 1 },
        Direction::Ascending => {
            let x = if i < size {
                j + 1
            } else {
                i + 1 + j + 1 - size
            };
            let y = if i < size {
                size - (i + 1) + (j + 1)
            } else {
                j + 1
            };
            Point { x: x, y: y }
        }
        Direction::Descending => {
            let x = if i < size {
                j + 1
            } else {
                i + 1 + (j + 1) - size
            };
            let y = if i < size { i + 1 - j } else { size - j };
            Point { x: x, y: y }
        }
    }
}

fn new_orthogonal_lines(size: u32) -> Vec<Line> {
    (0..size)
        .map(|_| Line::new(size, 0b0, 0b0).unwrap())
        .collect()
}

fn new_diagonal_lines(size: u32) -> Vec<Line> {
    (0..(size * 2 - 1))
        .map(|i| {
            let size = if i < size { i + 1 } else { size * 2 - 1 - i };
            Line::new(size, 0b0, 0b0).unwrap()
        })
        .collect()
}
