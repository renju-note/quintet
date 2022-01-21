use super::fundamentals::*;
use super::point::*;
use super::slot::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

use RowKind::*;

impl RowKind {
    pub fn nstones(&self) -> u8 {
        match self {
            Two => 2,
            Sword => 3,
            Three => 3,
            Four => 4,
            Five => 5,
            Overline => 5,
        }
    }
}

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

    pub fn from_slot(
        start: Index,
        target: Slot,
        may_prev: Option<Slot>,
        player: Player,
        kind: RowKind,
    ) -> Option<Row> {
        match kind {
            Five | Four | Sword => Self::from_single_slot(start, target, player, kind),
            _ if may_prev.is_some() => {
                Self::from_double_slots(start, target, may_prev.unwrap(), player, kind)
            }
            _ => None,
        }
    }

    fn from_single_slot(start: Index, target: Slot, player: Player, kind: RowKind) -> Option<Row> {
        if player.is_black() && target.will_overline() {
            return None;
        }

        if !target.occupied_by(player) {
            return None;
        }

        if target.nstones() != kind.nstones() {
            return None;
        }

        let direction = start.direction;
        let row_start = start.to_point();
        let row_end = start.walk(4).unwrap().to_point();

        if kind == Five {
            return Some(Row::new(direction, row_start, row_end, None, None));
        }

        let mut eyes = target.eyes();
        let row_eye1 = eyes.next().map(|e| start.walk(e as i8).unwrap().to_point());
        let row_eye2 = eyes.next().map(|e| start.walk(e as i8).unwrap().to_point());

        Some(Row::new(direction, row_start, row_end, row_eye1, row_eye2))
    }

    fn from_double_slots(
        start: Index,
        target: Slot,
        prev: Slot,
        player: Player,
        kind: RowKind,
    ) -> Option<Row> {
        if !target.occupied_by(player) || !prev.occupied_by(player) {
            return None;
        }

        let n = kind.nstones();
        if target.nstones() != n || prev.nstones() != n {
            return None;
        }

        if player.is_black() && (target.will_overline() || prev.will_overline()) {
            if kind == Overline && target.will_overline() && prev.will_overline() {
                let direction = start.direction;
                let row_start = start.walk(-1).unwrap().to_point();
                let row_end = start.walk(4).unwrap().to_point();
                return Some(Row::new(direction, row_start, row_end, None, None));
            } else {
                return None;
            };
        }

        if (target.signature & 0b00001111).count_ones() as u8 != n {
            return None;
        }

        let direction = start.direction;
        let row_start = start.to_point();
        let row_end = start.walk(3).unwrap().to_point();

        let mut eyes = target.eyes();
        let row_eye1 = eyes.next().map(|e| start.walk(e as i8).unwrap().to_point());
        let row_eye2 = if kind == Three {
            None
        } else {
            eyes.next().map(|e| start.walk(e as i8).unwrap().to_point())
        };

        Some(Row::new(direction, row_start, row_end, row_eye1, row_eye2))
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
    use super::super::line::*;
    use super::Direction::*;
    use super::Player::*;
    use super::*;

    #[test]
    fn test_from_slot() {
        let index0 = Index::new(Vertical, 0, 0);
        let result = Row::from_slot(
            index0,
            Slot::new(index0, 0b0111110, 0b0000000),
            None,
            Black,
            Five,
        );
        let expected = Some(Row::new(Vertical, Point(0, 0), Point(0, 4), None, None));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_line() -> Result<(), String> {
        let line = "-x-xx-----".parse::<Line>()?;
        let slots = line.segments().map(move |(j, blacks_, whites_)| {
            let index = Index::new(Vertical, 0, j as u8);
            (index, Slot::new(index, blacks_, whites_))
        });
        let result: Vec<Row> = slots
            .scan(None, |may_prev, (start, target)| {
                let result = Row::from_slot(start, target, *may_prev, White, Two);
                *may_prev = Some(target);
                Some(result)
            })
            .flatten()
            .collect();
        let expected = [Row {
            direction: Vertical,
            start: Point(0, 3),
            end: Point(0, 6),
            eye1: Some(Point(0, 5)),
            eye2: Some(Point(0, 6)),
        }];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_scan_line2() -> Result<(), String> {
        let line = "--o--ooo-------".parse::<Line>()?;
        let slots = line.segments().map(move |(j, blacks_, whites_)| {
            let index = Index::new(Vertical, 0, j as u8);
            (index, Slot::new(index, blacks_, whites_))
        });
        let result: Vec<Row> = slots
            .scan(None, |may_prev, (start, target)| {
                let result = Row::from_slot(start, target, *may_prev, Black, Three);
                *may_prev = Some(target);
                Some(result)
            })
            .flatten()
            .collect();
        let expected = [Row {
            direction: Vertical,
            start: Point(0, 5),
            end: Point(0, 8),
            eye1: Some(Point(0, 8)),
            eye2: None,
        }];
        assert_eq!(result, expected);

        Ok(())
    }

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
