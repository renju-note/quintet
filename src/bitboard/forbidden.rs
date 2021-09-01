use super::point::*;
use super::row::*;
use super::square::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub fn forbidden(square: &Square, p: Point) -> Option<ForbiddenKind> {
    let mut next = square.clone();
    next.put(Player::Black, p);
    if overline(&mut next, p) {
        Some(ForbiddenKind::Overline)
    } else if double_four(&mut next, p) {
        Some(ForbiddenKind::DoubleFour)
    } else if double_three(&mut next, p) {
        Some(ForbiddenKind::DoubleThree)
    } else {
        None
    }
}

fn overline(next: &Square, p: Point) -> bool {
    let new_overlines = next.rows_on(Player::Black, RowKind::Overline, p);
    new_overlines.len() >= 1
}

fn double_four(next: &Square, p: Point) -> bool {
    let new_fours = next.rows_on(Player::Black, RowKind::Four, p);
    if new_fours.len() < 2 {
        return false;
    }
    distinctive(&new_fours)
}

fn double_three(next: &Square, p: Point) -> bool {
    let new_threes = next.rows_on(Player::Black, RowKind::Three, p);
    if new_threes.len() < 2 || !distinctive(&new_threes) {
        return false;
    }
    let truthy_threes = new_threes
        .into_iter()
        .filter(|r| forbidden(&next, r.eye1.unwrap()).is_none())
        .collect::<Vec<_>>();
    if truthy_threes.len() < 2 {
        return false;
    }
    distinctive(&truthy_threes)
}

fn distinctive(rows: &Vec<RowSegment>) -> bool {
    let first = &rows[0];
    for row in rows.iter().skip(1) {
        if !adjacent(first, row) {
            return true;
        }
    }
    false
}

fn adjacent(a: &RowSegment, b: &RowSegment) -> bool {
    if a.direction != b.direction {
        return false;
    }
    let (ax, ay) = (a.start.0, a.start.1);
    let (bx, by) = (b.start.0, b.start.1);
    let (xd, yd) = (ax as i8 - bx as i8, ay as i8 - by as i8);
    match a.direction {
        Direction::Vertical => xd == 0 && yd.abs() == 1,
        Direction::Horizontal => xd.abs() == 1 && yd == 0,
        Direction::Ascending => xd.abs() == 1 && xd == yd,
        Direction::Descending => xd.abs() == 1 && xd == -yd,
    }
}
