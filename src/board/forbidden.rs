use super::row::*;
use super::square::*;

#[derive(Debug)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub fn forbiddens(square: &Square) -> Vec<(Point, ForbiddenKind)> {
    (1..=square.size)
        .flat_map(|x| (1..=square.size).map(move |y| Point { x: x, y: y }))
        .map(|p| (p, forbidden(square, p)))
        .filter(|(p, okind)| okind.is_some())
        .map(|(p, okind)| (p, okind.unwrap()))
        .collect()
}

pub fn forbidden(square: &Square, p: Point) -> Option<ForbiddenKind> {
    if (overline(square, p)) {
        Some(ForbiddenKind::Overline)
    } else if (double_four(square, p)) {
        Some(ForbiddenKind::DoubleFour)
    } else if (double_three(square, p)) {
        Some(ForbiddenKind::DoubleThree)
    } else {
        None
    }
}

fn overline(square: &Square, p: Point) -> bool {
    let next = square.put(true, p);
    next.rows(true, RowKind::Overline)
        .iter()
        .find(|&r| between(&p, r))
        .is_some()
}

fn double_four(square: &Square, p: Point) -> bool {
    let next = square.put(true, p);
    let fours = next.rows(true, RowKind::Four);
    let new_fours: Vec<_> = fours.iter().filter(|&r| between(&p, r)).collect();
    if new_fours.len() < 2 {
        return false;
    }
    distinctive(new_fours)
}

fn double_three(square: &Square, p: Point) -> bool {
    let next = square.put(true, p);
    let threes = next.rows(true, RowKind::Three);
    let new_threes: Vec<_> = threes.iter().filter(|&r| between(&p, r)).collect();
    if new_threes.len() < 2 {
        return false;
    }
    let truthy_threes = new_threes
        .iter()
        .filter(|&r| forbidden(&next, r.eyes[0]).is_none());
    distinctive(new_threes)
}

fn between(p: &Point, r: &SquareRow) -> bool {
    let (s, e) = (r.start, r.end);
    match r.direction {
        Direction::Vertical => p.x == s.x && s.y <= p.y && p.y <= e.y,
        Direction::Horizontal => p.y == s.y && s.x <= p.x && p.x <= e.x,
        Direction::Ascending => s.x <= p.x && p.x <= e.x && p.x - s.x == p.y - s.y,
        Direction::Descending => s.x <= p.x && p.x <= e.x && p.x - s.x == s.y - p.y,
    }
}

fn distinctive(srows: Vec<&SquareRow>) -> bool {
    let mut prev: Option<&SquareRow> = None;
    for s in srows {
        match prev {
            None => (),
            Some(p) => {
                if !adjacent(p, s) {
                    return true;
                }
            }
        }
        prev = Some(s);
    }
    false
}

fn adjacent(a: &SquareRow, b: &SquareRow) -> bool {
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
