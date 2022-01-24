use super::player::*;
use super::point::*;
use super::sequence::*;
use super::square::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub fn forbiddens(q: &Square) -> Vec<(ForbiddenKind, Point)> {
    (0..SQUARE_SIZE)
        .flat_map(|x| (0..SQUARE_SIZE).map(move |y| Point(x, y)))
        .map(|p| (forbidden_strict(q, p), p))
        .filter(|(k, _)| k.is_some())
        .map(|(k, p)| (k.unwrap(), p))
        .collect()
}

pub fn forbidden_strict(q: &Square, p: Point) -> Option<ForbiddenKind> {
    if q.stone(p).is_some() {
        return None;
    }
    let mut next_fives = q.sequences_on(p, Black, Single, 4, true);
    if next_fives.next().is_some() {
        return None;
    }
    forbidden(q, p)
}

pub fn forbidden(q: &Square, p: Point) -> Option<ForbiddenKind> {
    if overline(&q, p) {
        Some(ForbiddenKind::Overline)
    } else if double_four(&q, p) {
        Some(ForbiddenKind::DoubleFour)
    } else if double_three(&q, p) {
        Some(ForbiddenKind::DoubleThree)
    } else {
        None
    }
}

fn overline(q: &Square, p: Point) -> bool {
    let mut next_overlines = q.sequences_on(p, Black, Double, 4, false);
    next_overlines.next().is_some()
}

fn double_four(q: &Square, p: Point) -> bool {
    let next_fours = q.sequences_on(p, Black, Single, 3, true);
    distinctive(&mut next_fours.map(|p| p.0))
}

fn double_three(q: &Square, p: Point) -> bool {
    let next_threes = q.sequences_on(p, Black, Compact, 2, true);
    if !distinctive(&mut next_threes.map(|p| p.0)) {
        return false;
    }
    let mut next = q.clone();
    next.put_mut(Black, p);
    truthy_double_three(&next, p)
}

fn truthy_double_three(next: &Square, p: Point) -> bool {
    let truthy_threes = next
        .sequences_on(p, Black, Compact, 3, true)
        .filter(|(i, s)| {
            let eye = i.subsequence(s.eyes()).next().unwrap().to_point();
            forbidden(&next, eye).is_none()
        });
    distinctive(&mut truthy_threes.map(|p| p.0))
}

fn distinctive(indices: &mut impl Iterator<Item = Index>) -> bool {
    let first = indices.next();
    if first.is_none() {
        return false;
    }
    let next_to_first = first.unwrap().walk(1);
    for index in indices {
        if index != next_to_first {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::ForbiddenKind::*;
    use super::*;

    #[test]
    fn test_forbiddens() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . o
         . . o . . . . . . . . x o o .
         . o . o . . . . . . . o . o .
         . . o . . . . . . . . o o x .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . o . . . . . . . . . . . . .
         . . o . . . . . . . . . . . .
         . . . o . . . . . . . . . . .
         . o o o . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . o o . o o o . . . o o o o .
        "
        .parse::<Square>()?;
        let result = forbiddens(&square);
        let expected = [
            (DoubleThree, Point(2, 12)),
            (Overline, Point(3, 0)),
            (DoubleFour, Point(4, 4)),
        ];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_double_three() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . o . o . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        let square = "
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . o . o . . . . . . .
        . . . . o . . o . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . x . o . o . x . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o o . o o . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o o . . . o o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o o o x . . . .
         . . . . . . x . . . . . . . .
         . . . . . . o o . . . . . . .
         . . . . . . o . o o x . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);
        let result = forbidden(&square, Point(8, 6));
        assert_eq!(result, None);
        let result = forbidden(&square, Point(9, 6));
        assert_eq!(result, Some(DoubleThree));

        // following examples are from https://twitter.com/tanaseY/status/944521796585373696
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . x . . . . . . . .
         . . o . . o . o . . . . . . .
         . . . . . o . . o o . . . . .
         . . . . o . . . . . . o o . x
         . . . . . . . . o x o o . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(6, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . x . . . . . . . .
         . . o . . o . o . . . . . . .
         . . . . . o . . o o . . . . .
         . . . . o . . . . . . o o . .
         . . . . . . . . o x o o . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(6, 7));
        assert_eq!(result, Some(DoubleThree));

        Ok(())
    }

    #[test]
    fn test_double_four() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o . . . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . o o . o . . . . . .
         . . . . . . . . o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o . o . o . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o o . . o . o o . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . o o o . . . o o o . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . o o o o . . . o o o o . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . o o o o . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . o o o . . . . . o o o . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . o o o o . . . . . o o o o .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o o . o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . o . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o . . . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . o o . o . . . . . .
         . . . . . . . . o . . . . . .
         . . . . . . . . . x . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_overline() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o o o . o o . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(Overline));

        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o o o . . o o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }
}
