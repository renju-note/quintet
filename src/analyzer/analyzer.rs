use super::super::board::*;
use super::row::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub struct BoardRow {
    pub kind: RowKind,
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eyes: Vec<Point>,
}

pub struct Analyzer {
    rows_cache: HashMap<(Line, bool, RowKind), Vec<Row>>,
    forbiddens_cache: HashMap<String, Vec<(Point, ForbiddenKind)>>,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            rows_cache: HashMap::new(),
            forbiddens_cache: HashMap::new(),
        }
    }

    pub fn get_rows(&mut self, board: &Board, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = Vec::new();
        for (direction, i, line) in board.iter_lines() {
            let key = (*line, black, kind);
            let mut rows = self
                .rows_cache
                .entry(key)
                .or_insert_with(|| line_rows(line, black, kind))
                .iter()
                .map(|r| board_row(direction, i as u8, r, kind))
                .collect::<Vec<_>>();
            result.append(&mut rows);
        }
        result
    }

    pub fn get_forbiddens(&mut self, board: &Board) -> Vec<(Point, ForbiddenKind)> {
        let key = board.to_string();
        match self.forbiddens_cache.get(&key) {
            Some(result) => result.to_vec(),
            None => {
                let result = self.forbiddens(board);
                self.forbiddens_cache.insert(key, result.to_vec());
                result
            }
        }
    }

    pub fn forbiddens(&mut self, board: &Board) -> Vec<(Point, ForbiddenKind)> {
        (1..=BOARD_SIZE)
            .flat_map(|x| (1..=BOARD_SIZE).map(move |y| Point { x: x, y: y }))
            .map(|p| (p, self.forbidden(board, p)))
            .filter(|(_, okind)| okind.is_some())
            .map(|(p, okind)| (p, okind.unwrap()))
            .collect()
    }

    pub fn forbidden(&mut self, board: &Board, p: Point) -> Option<ForbiddenKind> {
        if self.overline(board, p) {
            Some(ForbiddenKind::Overline)
        } else if self.double_four(board, p) {
            Some(ForbiddenKind::DoubleFour)
        } else if self.double_three(board, p) {
            Some(ForbiddenKind::DoubleThree)
        } else {
            None
        }
    }

    fn overline(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        self.get_rows(&next, true, RowKind::Overline)
            .iter()
            .find(|&r| between(&p, r))
            .is_some()
    }

    fn double_four(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        let fours = self.get_rows(&next, true, RowKind::Four);
        let new_fours: Vec<_> = fours.iter().filter(|&r| between(&p, r)).collect();
        if new_fours.len() < 2 {
            return false;
        }
        distinctive(new_fours)
    }

    fn double_three(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        let threes = self.get_rows(&next, true, RowKind::Three);
        let new_threes: Vec<_> = threes.iter().filter(|&r| between(&p, r)).collect();
        if new_threes.len() < 2 {
            return false;
        }
        let truthy_threes: Vec<_> = new_threes
            .iter()
            .filter(|&r| self.forbidden(&next, r.eyes[0]).is_none())
            .map(|&r| r)
            .collect();
        distinctive(truthy_threes)
    }
}

fn board_row(direction: Direction, i: u8, row: &Row, kind: RowKind) -> BoardRow {
    BoardRow {
        kind: kind,
        direction: direction,
        start: Index { i: i, j: row.start }.to_point(direction),
        end: Index {
            i: i,
            j: row.start + row.size - 1,
        }
        .to_point(direction),
        eyes: row
            .eyes
            .iter()
            .map(|&j| Index { i: i, j: j }.to_point(direction))
            .collect(),
    }
}

fn line_rows(line: &Line, black: bool, kind: RowKind) -> Vec<Row> {
    let blacks_: Stones;
    let whites_: Stones;
    if black {
        blacks_ = line.blacks << 1;
        whites_ = append_dummies(line.whites, line.size);
    } else {
        blacks_ = append_dummies(line.blacks, line.size);
        whites_ = line.whites << 1;
    }
    let size_ = line.size + 2;

    search_pattern(blacks_, whites_, size_, black, kind)
        .iter()
        .map(|row| Row {
            start: row.start - 1,
            size: row.size,
            eyes: row.eyes.iter().map(|x| x - 1).collect(),
        })
        .collect()
}

fn append_dummies(stones: Stones, size: u8) -> Stones {
    (stones << 1) | 0b1 | (0b1 << (size + 1))
}

fn between(p: &Point, r: &BoardRow) -> bool {
    let (s, e) = (r.start, r.end);
    match r.direction {
        Direction::Vertical => p.x == s.x && s.y <= p.y && p.y <= e.y,
        Direction::Horizontal => p.y == s.y && s.x <= p.x && p.x <= e.x,
        Direction::Ascending => {
            s.x <= p.x && p.x <= e.x && s.y <= p.y && p.y <= e.y && p.x - s.x == p.y - s.y
        }
        Direction::Descending => {
            s.x <= p.x && p.x <= e.x && e.y <= p.y && p.y <= s.y && p.x - s.x == s.y - p.y
        }
    }
}

fn distinctive(srows: Vec<&BoardRow>) -> bool {
    let mut prev: Option<&BoardRow> = None;
    for s in srows {
        match prev {
            None => (),
            Some(p) => {
                if !adjacent(p, s) {
                    return true;
                }
            }
        }
        prev = Some(s);
    }
    false
}

fn adjacent(a: &BoardRow, b: &BoardRow) -> bool {
    if a.direction != b.direction {
        return false;
    }
    let (xd, yd) = (
        a.start.x as i32 - b.start.x as i32,
        a.start.y as i32 - b.start.y as i32,
    );
    match a.direction {
        Direction::Vertical => xd == 0 && yd.abs() == 1,
        Direction::Horizontal => xd.abs() == 1 && yd == 0,
        Direction::Ascending => xd.abs() == 1 && xd == yd,
        Direction::Descending => xd.abs() == 1 && xd == -yd,
    }
}
