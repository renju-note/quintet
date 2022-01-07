use super::point::*;
use super::sequence::*;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Row {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
}

impl Row {
    pub fn new(
        direction: Direction,
        start: Point,
        end: Point,
        eye1: Option<Point>,
        eye2: Option<Point>,
    ) -> Row {
        Row {
            direction: direction,
            start: start,
            end: end,
            eye1: eye1,
            eye2: eye2,
        }
    }

    pub fn from_sequence(s: &Sequence, d: Direction, i: u8) -> Row {
        Row {
            direction: d,
            start: Index::new(d, i, s.start).to_point(),
            end: Index::new(d, i, s.end).to_point(),
            eye1: s.eye1.map(|e| Index::new(d, i, e).to_point()),
            eye2: s.eye2.map(|e| Index::new(d, i, e).to_point()),
        }
    }

    pub fn overlap(&self, p: Point) -> bool {
        let (px, py) = (p.0, p.1);
        let (sx, sy) = (self.start.0, self.start.1);
        let (ex, ey) = (self.end.0, self.end.1);
        match self.direction {
            Direction::Vertical => px == sx && bw(sy, py, ey),
            Direction::Horizontal => py == sy && bw(sx, px, ex),
            Direction::Ascending => bw(sx, px, ex) && bw(sy, py, ey) && px - sx == py - sy,
            Direction::Descending => bw(sx, px, ex) && bw(ey, py, sy) && px - sx == sy - py,
        }
    }

    pub fn adjacent(&self, other: &Row) -> bool {
        if self.direction != other.direction {
            return false;
        }
        let (sx, sy) = (self.start.0, self.start.1);
        let (ox, oy) = (other.start.0, other.start.1);
        let (xd, yd) = (sx as i8 - ox as i8, sy as i8 - oy as i8);
        match self.direction {
            Direction::Vertical => xd == 0 && yd.abs() == 1,
            Direction::Horizontal => xd.abs() == 1 && yd == 0,
            Direction::Ascending => xd.abs() == 1 && xd == yd,
            Direction::Descending => xd.abs() == 1 && xd == -yd,
        }
    }

    pub fn into_iter_eyes(&self) -> impl IntoIterator<Item = Point> {
        self.eye1.into_iter().chain(self.eye2.into_iter())
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} start: {} end: {}",
            self.direction, self.start, self.end
        )?;
        for eye1 in self.eye1 {
            write!(f, " eye1: {}", eye1)?;
        }
        for eye2 in self.eye2 {
            write!(f, " eye2: {}", eye2)?;
        }
        Ok(())
    }
}

fn bw(a: u8, x: u8, b: u8) -> bool {
    a <= x && x <= b
}

#[cfg(test)]
mod tests {
    use super::Direction::*;
    use super::*;

    #[test]
    fn test_adjacent() {
        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Horizontal, Point(3, 3), Point(9, 3), None, None);
        assert!(!a.adjacent(&b));

        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Vertical, Point(3, 4), Point(3, 10), None, None);
        assert!(a.adjacent(&b));

        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Vertical, Point(3, 5), Point(3, 11), None, None);
        assert!(!a.adjacent(&b));

        let a = Row::new(Descending, Point(3, 9), Point(9, 3), None, None);
        let b = Row::new(Descending, Point(4, 8), Point(10, 2), None, None);
        assert!(a.adjacent(&b));
    }
}
