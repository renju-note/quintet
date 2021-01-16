use super::super::board::{Bits, Board, Direction, Index, Line, Point};
use super::pattern2::*;
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
        for (direction, i, line) in board.iter_lines(black, !black) {
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
        p: &Point,
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
            (true, RowKind::Four) => self.scan_patterns(line, black, &BLACK_FOURS),
            (true, RowKind::Five) => self.scan_patterns(line, black, &BLACK_FIVES),
            (true, RowKind::Overline) => self.scan_patterns(line, black, &BLACK_OVERLINES),
            (false, RowKind::Four) => self.scan_patterns(line, black, &WHITE_FOURS),
            (false, RowKind::Five) => self.scan_patterns(line, black, &WHITE_FIVES),
            _ => vec![],
        }
    }

    fn scan_patterns(&self, line: &Line, black: bool, patterns: &[Pattern]) -> Vec<LineRow> {
        patterns
            .iter()
            .flat_map(|p| self.scan_pattern(line, &p, black))
            .collect()
    }

    fn scan_pattern(&self, line: &Line, pattern: &Pattern, black: bool) -> Vec<LineRow> {
        let pattern_size = pattern.size();
        let scanned_size = line.size + 2;
        if scanned_size < pattern_size {
            return vec![];
        }

        let mut stones: Bits = if black { line.blacks } else { line.whites };
        let mut blanks: Bits = !(line.blacks | line.whites) & ((0b1 << line.size) - 1);

        let mut result = vec![];
        stones <<= 1;
        blanks <<= 1;
        for i in 0..=(scanned_size - pattern_size) {
            if (stones & pattern.filter == pattern.stones)
                && (blanks & pattern.filter & pattern.blanks == pattern.blanks)
            {
                let start = i + pattern.start() - 1;
                let end = i + pattern.end() - 1;
                let eyes = pattern.eyes().iter().map(|e| e + i - 1).collect();
                let row = LineRow {
                    start: start,
                    end: end,
                    eyes: eyes,
                };
                result.push(row);
            }
            stones >>= 1;
            blanks >>= 1;
        }
        return result;
    }
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
            end: Index { i: i, j: lr.end }.to_point(direction),
            eyes: lr
                .eyes
                .iter()
                .map(|&j| Index { i: i, j: j }.to_point(direction))
                .collect(),
        }
    }

    pub fn overlap(&self, p: &Point) -> bool {
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
    end: u8,
    eyes: Vec<u8>,
}
