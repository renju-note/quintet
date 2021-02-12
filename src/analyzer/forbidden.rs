use super::super::board::*;
use super::row;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub fn forbiddens(board: &Board) -> Vec<(ForbiddenKind, Point)> {
    (1..=BOARD_SIZE)
        .flat_map(|x| (1..=BOARD_SIZE).map(move |y| Point { x: x, y: y }))
        .map(|p| (forbidden(board, &p), p))
        .filter(|(k, _)| k.is_some())
        .map(|(k, p)| (k.unwrap(), p))
        .collect()
}

pub fn forbidden(board: &Board, p: &Point) -> Option<ForbiddenKind> {
    let mut next = board.clone();
    next.put(true, p);
    if overline(&next, p) {
        Some(ForbiddenKind::Overline)
    } else if double_four(&next, p) {
        Some(ForbiddenKind::DoubleFour)
    } else if double_three(&next, p) {
        Some(ForbiddenKind::DoubleThree)
    } else {
        None
    }
}

fn overline(next: &Board, p: &Point) -> bool {
    let new_overlines = row::rows_on(&next, true, row::RowKind::Overline, p);
    new_overlines.len() >= 1
}

fn double_four(next: &Board, p: &Point) -> bool {
    let new_fours = row::rows_on(&next, true, row::RowKind::Four, p);
    if new_fours.len() < 2 {
        return false;
    }
    distinctive(new_fours.iter().collect())
}

fn double_three(next: &Board, p: &Point) -> bool {
    let new_threes = row::rows_on(&next, true, row::RowKind::Three, p);
    if new_threes.len() < 2 {
        return false;
    }
    let truthy_threes = new_threes
        .iter()
        .filter(|r| forbidden(&next, &r.eyes[0]).is_none())
        .collect::<Vec<_>>();
    if truthy_threes.len() < 2 {
        return false;
    }
    distinctive(truthy_threes)
}

fn distinctive(rows: Vec<&row::Row>) -> bool {
    let first = rows[0];
    for row in rows.iter().skip(1) {
        if !adjacent(first, row) {
            return true;
        }
    }
    false
}

fn adjacent(a: &row::Row, b: &row::Row) -> bool {
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
