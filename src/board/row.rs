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

    pub fn potential(&self) -> u8 {
        match self {
            Two => 3,
            Sword => 4,
            Three => 4,
            Four => 5,
            Five => 6,
            Overline => 6,
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

    pub fn from_slots(
        slots: Slots,
        d: Direction,
        i: u8,
        player: Player,
        kind: RowKind,
    ) -> impl Iterator<Item = Row> {
        slots
            .map(move |(j, s)| (Index::new(d, i, j), s))
            .scan(None, move |may_prev, (start, target)| {
                let result = Row::from_slot(start, target, *may_prev, player, kind);
                *may_prev = Some(target);
                Some(result)
            })
            .flatten()
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

        let eyes = target.eyes();
        let row_eye1 = start.walk(eyes[0] as i8).map(|i| i.to_point());

        if kind == Four {
            return Some(Row::new(direction, row_start, row_end, row_eye1, None));
        }

        let row_eye2 = start.walk(eyes[1] as i8).map(|i| i.to_point());
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

        if kind == Overline {
            return if player.is_black() && target.will_overline() && prev.will_overline() {
                let direction = start.direction;
                let row_start = start.walk(-1).unwrap().to_point();
                let row_end = start.walk(4).unwrap().to_point();
                Some(Row::new(direction, row_start, row_end, None, None))
            } else {
                None
            };
        }

        if player.is_black() && (target.will_overline() || prev.will_overline()) {
            return None;
        }

        if target.nstones_head() != n {
            return None;
        }

        let direction = start.direction;
        let row_start = start.to_point();
        let row_end = start.walk(3).unwrap().to_point();

        let eyes = target.eyes_head();
        let row_eye1 = start.walk(eyes[0] as i8).map(|i| i.to_point());

        if kind == Three {
            return Some(Row::new(direction, row_start, row_end, row_eye1, None));
        }

        let row_eye2 = start.walk(eyes[1] as i8).map(|i| i.to_point());
        Some(Row::new(direction, row_start, row_end, row_eye1, row_eye2))
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

#[cfg(test)]
mod tests {
    use super::super::line::*;
    use super::Direction::*;
    use super::Player::*;
    use super::*;

    #[test]
    fn test_from_slot() {
        let index0 = Index::new(Vertical, 0, 0);
        let result = Row::from_slot(index0, Slot::new(0b0111110, 0b0000000), None, Black, Five);
        let expected = Some(Row::new(Vertical, Point(0, 0), Point(0, 4), None, None));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_slots() -> Result<(), String> {
        let line = "-x-xx-----".parse::<Line>()?;
        let result = Row::from_slots(line.slots(), Vertical, 0, White, Two).collect::<Vec<_>>();
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
    fn test_from_slots_2() -> Result<(), String> {
        let line = "--o--ooo-------".parse::<Line>()?;
        let result = Row::from_slots(line.slots(), Vertical, 0, Black, Three).collect::<Vec<_>>();
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
