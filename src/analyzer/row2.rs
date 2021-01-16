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
    cache: HashMap<(Line, bool, RowKind), Vec<Segment>>,
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
                .map(|s| Row::from(s, direction, i))
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
                .map(|s| Row::from(s, direction, i))
                .filter(|r| r.overlap(p))
                .collect();
            result.append(&mut rows);
        }
        result
    }

    fn search_line(&mut self, line: &Line, black: bool, kind: RowKind) -> Vec<Segment> {
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

    fn scan(&self, line: &Line, black: bool, kind: RowKind) -> Vec<Segment> {
        match (black, kind) {
            (true, RowKind::Three) => self.scan_patterns(line, black, &BLACK_THREES),
            (true, RowKind::Four) => self.scan_patterns(line, black, &BLACK_FOURS),
            (true, RowKind::Five) => self.scan_patterns(line, black, &BLACK_FIVES),
            (true, RowKind::Overline) => self.scan_patterns(line, black, &BLACK_OVERLINES),
            (false, RowKind::Three) => self.scan_patterns(line, black, &WHITE_THREES),
            (false, RowKind::Four) => self.scan_patterns(line, black, &WHITE_FOURS),
            (false, RowKind::Five) => self.scan_patterns(line, black, &WHITE_FIVES),
            _ => vec![],
        }
    }

    fn scan_patterns(&self, line: &Line, black: bool, patterns: &[Pattern]) -> Vec<Segment> {
        patterns
            .iter()
            .flat_map(|p| self.scan_pattern(line, p, black))
            .collect()
    }

    fn scan_pattern(&self, line: &Line, pattern: &Pattern, black: bool) -> Vec<Segment> {
        let pattern_size = pattern.size();
        let scanned_size = line.size + 2;
        if scanned_size < pattern_size {
            return vec![];
        }

        let mut stones: Bits = if black { line.blacks } else { line.whites };
        let mut blanks: Bits = line.blanks();

        let mut result = vec![];
        stones <<= 1;
        blanks <<= 1;
        for i in 0..=(scanned_size - pattern_size) {
            let segment = pattern.matches(stones, blanks, i as i8 - 1);
            if segment.is_some() {
                result.push(segment.unwrap())
            }
            stones >>= 1;
            blanks >>= 1;
        }
        return result;
    }
}

#[derive(Clone)]
pub struct Row {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eyes: Vec<Point>,
}

impl Row {
    fn from(segment: &Segment, direction: Direction, i: u8) -> Row {
        Row {
            direction: direction,
            start: Index {
                i: i,
                j: segment.start,
            }
            .to_point(direction),
            end: Index {
                i: i,
                j: segment.end,
            }
            .to_point(direction),
            eyes: segment
                .eyes
                .iter()
                .map(|&j| Index { i: i, j: j }.to_point(direction))
                .collect(),
        }
    }

    pub fn overlap(&self, p: &Point) -> bool {
        let (s, e) = (self.start, self.end);
        match self.direction {
            Direction::Vertical => p.x == s.x && Row::between(s.y, p.y, e.y),
            Direction::Horizontal => p.y == s.y && Row::between(s.x, p.x, e.x),
            Direction::Ascending => {
                Row::between(s.x, p.x, e.x) && Row::between(s.y, p.y, e.y) && p.x - s.x == p.y - s.y
            }
            Direction::Descending => {
                Row::between(s.x, p.x, e.x) && Row::between(e.y, p.y, s.y) && p.x - s.x == s.y - p.y
            }
        }
    }

    fn between(a: u8, x: u8, b: u8) -> bool {
        a <= x && x <= b
    }
}
