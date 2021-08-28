use super::bits::*;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub i: u8,
    pub j: u8,
}

const N: u8 = BOARD_SIZE - 1;

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.x, self.y);
        match direction {
            Direction::Vertical => Index { i: x, j: y },
            Direction::Horizontal => Index { i: y, j: x },
            Direction::Ascending => {
                let i = x + N - y;
                let j = if i < N { x } else { y };
                Index { i: i, j: j }
            }
            Direction::Descending => {
                let i = x + y;
                let j = if i < N { x } else { N - y };
                Index { i: i, j: j }
            }
        }
    }
}

impl Index {
    pub fn to_point(&self, direction: Direction) -> Point {
        let (i, j) = (self.i, self.j);
        match direction {
            Direction::Vertical => Point { x: i, y: j },
            Direction::Horizontal => Point { x: j, y: i },
            Direction::Ascending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { N - i + j } else { j };
                Point { x: x, y: y }
            }
            Direction::Descending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { i - j } else { N - j };
                Point { x: x, y: y }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError;

const N_RANGE: std::ops::Range<u8> = 0..BOARD_SIZE;

impl FromStr for Point {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = s
            .chars()
            .nth(0)
            .map(|c| match c {
                'A'..='O' => Some(c as u8 - 'A' as u8),
                'a'..='o' => Some(c as u8 - 'a' as u8),
                _ => None,
            })
            .flatten()
            .ok_or(ParseError)?;
        let y = s
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<u8>()
            .map(|y| y - 1)
            .map_err(|_| ParseError)?;
        let is_valid = N_RANGE.contains(&x) && N_RANGE.contains(&y);
        is_valid.then(|| Point { x: x, y: y }).ok_or(ParseError)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xs = std::char::from_u32(('A' as u8 + self.x) as u32)
            .unwrap()
            .to_string();
        let ys = (self.y + 1).to_string();
        write!(f, "{}{}", xs, ys)
    }
}
