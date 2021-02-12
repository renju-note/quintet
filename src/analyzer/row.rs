use super::super::board::{Bits, Board, Direction, Index, Line, Point};
use super::pattern::*;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

pub fn rows(board: &Board, black: bool, kind: RowKind) -> Vec<Row> {
    let mut result = Vec::new();
    for (direction, i, line) in board.lines(black, !black) {
        let mut rows = scan(&line, black, kind)
            .iter()
            .map(|s| Row::from(s, direction, i))
            .collect();
        result.append(&mut rows);
    }
    result
}

pub fn rows_on(board: &Board, black: bool, kind: RowKind, p: &Point) -> Vec<Row> {
    let mut result = Vec::new();
    for (direction, i, line) in board.lines_on(p) {
        let mut rows = scan(&line, black, kind)
            .iter()
            .map(|s| Row::from(s, direction, i))
            .filter(|r| r.overlap(p))
            .collect();
        result.append(&mut rows);
    }
    result
}

pub fn row_eyes(board: &Board, black: bool, kind: RowKind) -> Vec<Point> {
    rows(board, black, kind)
        .iter()
        .flat_map(|r| r.eyes.to_vec())
        .collect::<HashSet<_>>() // unique
        .into_iter()
        .collect()
}

pub fn row_eyes_on(board: &Board, black: bool, kind: RowKind, p: &Point) -> Vec<Point> {
    rows_on(board, black, kind, p)
        .iter()
        .flat_map(|r| r.eyes.to_vec())
        .collect::<HashSet<_>>() // unique
        .into_iter()
        .collect()
}

fn scan(line: &Line, black: bool, kind: RowKind) -> Vec<Segment> {
    match (black, kind) {
        (true, RowKind::Two) => scan_patterns(line, black, &BLACK_TWOS),
        (true, RowKind::Sword) => scan_patterns(line, black, &BLACK_SWORDS),
        (true, RowKind::Three) => scan_patterns(line, black, &BLACK_THREES),
        (true, RowKind::Four) => scan_patterns(line, black, &BLACK_FOURS),
        (true, RowKind::Five) => scan_patterns(line, black, &BLACK_FIVES),
        (true, RowKind::Overline) => scan_patterns(line, black, &BLACK_OVERLINES),
        (false, RowKind::Two) => scan_patterns(line, black, &WHITE_TWOS),
        (false, RowKind::Sword) => scan_patterns(line, black, &WHITE_SWORDS),
        (false, RowKind::Three) => scan_patterns(line, black, &WHITE_THREES),
        (false, RowKind::Four) => scan_patterns(line, black, &WHITE_FOURS),
        (false, RowKind::Five) => scan_patterns(line, black, &WHITE_FIVES),
        _ => vec![],
    }
}

fn scan_patterns(line: &Line, black: bool, patterns: &[Pattern]) -> Vec<Segment> {
    patterns
        .iter()
        .flat_map(|p| scan_pattern(line, p, black))
        .collect()
}

fn scan_pattern(line: &Line, pattern: &Pattern, black: bool) -> Vec<Segment> {
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

#[derive(Clone, Debug)]
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
