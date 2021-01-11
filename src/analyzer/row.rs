use super::super::board::{Board, Direction, Index, Line, Point, Stones};
use super::pattern::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

pub struct RowSearcher {
    cache: HashMap<(Line, bool, RowKind), Vec<LineRow>>,
}

impl RowSearcher {
    pub fn new() -> RowSearcher {
        RowSearcher {
            cache: HashMap::new(),
        }
    }

    pub fn search(&mut self, board: &Board, black: bool, kind: RowKind) -> Vec<Row> {
        let mut result = Vec::new();
        for (direction, i, line) in board.iter_lines() {
            let mut rows = self
                .search_line(&line, black, kind)
                .iter()
                .map(|lr| Row::from(lr, direction, i))
                .collect();
            result.append(&mut rows);
        }
        result
    }

    pub fn search_containing(
        &mut self,
        board: &Board,
        black: bool,
        kind: RowKind,
        p: Point,
    ) -> Vec<Row> {
        let mut result = Vec::new();
        for (direction, i, line) in board.lines_of(p) {
            let mut rows = self
                .search_line(&line, black, kind)
                .iter()
                .map(|lr| Row::from(lr, direction, i))
                .filter(|r| r.overlap(p))
                .collect();
            result.append(&mut rows);
        }
        result
    }

    fn search_line(&mut self, line: &Line, black: bool, kind: RowKind) -> Vec<LineRow> {
        let key = (*line, black, kind);
        match self.cache.get(&key) {
            Some(result) => result.to_vec(),
            None => {
                let result = self.scan(line, black, kind);
                self.cache.insert(key, result.to_vec());
                result
            }
        }
    }

    fn scan(&self, line: &Line, black: bool, kind: RowKind) -> Vec<LineRow> {
        match (black, kind) {
            (true, RowKind::Two) => self.scan_patterns(line, black, BLACK_TWO_PATTERNS),
            (true, RowKind::Sword) => self.scan_patterns(line, black, BLACK_SWORD_PATTERNS),
            (true, RowKind::Three) => self.scan_patterns(line, black, BLACK_THREE_PATTERNS),
            (true, RowKind::Four) => self.scan_patterns(line, black, BLACK_FOUR_PATTERNS),
            (true, RowKind::Five) => self.scan_patterns(line, black, BLACK_FIVE_PATTERNS),
            (true, RowKind::Overline) => self.scan_patterns(line, black, BLACK_OVERLINE_PATTERNS),
            (false, RowKind::Two) => self.scan_patterns(line, black, WHITE_TWO_PATTERNS),
            (false, RowKind::Sword) => self.scan_patterns(line, black, WHITE_SWORD_PATTERNS),
            (false, RowKind::Three) => self.scan_patterns(line, black, WHITE_THREE_PATTERNS),
            (false, RowKind::Four) => self.scan_patterns(line, black, WHITE_FOUR_PATTERNS),
            (false, RowKind::Five) => self.scan_patterns(line, black, WHITE_FIVE_PATTERNS),
            _ => vec![],
        }
    }

    fn scan_patterns(&self, line: &Line, black: bool, patterns: &[&RowPattern]) -> Vec<LineRow> {
        patterns
            .iter()
            .flat_map(|p| self.scan_pattern(line, &p, black))
            .collect()
    }

    fn scan_pattern(&self, line: &Line, pattern: &RowPattern, black: bool) -> Vec<LineRow> {
        if line.size < pattern.row.size {
            return vec![];
        }

        let mut blacks_: Stones;
        let mut whites_: Stones;
        if black {
            blacks_ = line.blacks << 1;
            whites_ = append_dummies(line.whites, line.size);
        } else {
            blacks_ = append_dummies(line.blacks, line.size);
            whites_ = line.whites << 1;
        }
        let within = line.size + 2;
        if within < pattern.size {
            return vec![];
        }

        let filter: Stones = (1 << pattern.size) - 1;
        let mut result = vec![];
        for i in 0..=(within - pattern.size) {
            if (blacks_ & filter & !pattern.blmask) == pattern.blacks
                && (whites_ & filter & !pattern.whmask) == pattern.whites
            {
                let start = i + pattern.offset - 1;
                let row = LineRow {
                    start: start,
                    size: pattern.row.size,
                    eyes: pattern.row.eyes.iter().map(|eye| eye + start).collect(),
                };
                result.push(row);
            }
            blacks_ = blacks_ >> 1;
            whites_ = whites_ >> 1;
        }
        return result;
    }
}

fn append_dummies(stones: Stones, size: u8) -> Stones {
    (stones << 1) | 0b1 | (0b1 << (size + 1))
}

fn between(a: u8, x: u8, b: u8) -> bool {
    a <= x && x <= b
}

#[derive(Clone)]
pub struct Row {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eyes: Vec<Point>,
}

impl Row {
    fn from(lr: &LineRow, direction: Direction, i: u8) -> Row {
        Row {
            direction: direction,
            start: Index { i: i, j: lr.start }.to_point(direction),
            end: Index {
                i: i,
                j: lr.start + lr.size - 1,
            }
            .to_point(direction),
            eyes: lr
                .eyes
                .iter()
                .map(|&j| Index { i: i, j: j }.to_point(direction))
                .collect(),
        }
    }

    pub fn overlap(&self, p: Point) -> bool {
        let (s, e) = (self.start, self.end);
        match self.direction {
            Direction::Vertical => p.x == s.x && between(s.y, p.y, e.y),
            Direction::Horizontal => p.y == s.y && between(s.x, p.x, e.x),
            Direction::Ascending => {
                between(s.x, p.x, e.x) && between(s.y, p.y, e.y) && p.x - s.x == p.y - s.y
            }
            Direction::Descending => {
                between(s.x, p.x, e.x) && between(e.y, p.y, s.y) && p.x - s.x == s.y - p.y
            }
        }
    }
}

#[derive(Clone)]
struct LineRow {
    start: u8,
    size: u8,
    eyes: Vec<u8>,
}
