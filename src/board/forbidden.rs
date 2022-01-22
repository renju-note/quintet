use super::fundamentals::*;
use super::point::*;
use super::sequence::*;
use super::square::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub fn forbiddens(square: &Square) -> Vec<(ForbiddenKind, Point)> {
    (0..BOARD_SIZE)
        .flat_map(|x| (0..BOARD_SIZE).map(move |y| Point(x, y)))
        .map(|p| (forbidden(square, p), p))
        .filter(|(k, _)| k.is_some())
        .map(|(k, p)| (k.unwrap(), p))
        .collect()
}

pub fn forbidden(square: &Square, p: Point) -> Option<ForbiddenKind> {
    let mut next = square.clone();
    next.put_mut(Black, p);
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

fn overline(next: &Square, p: Point) -> bool {
    let mut new_overlines = next.sequences_on(p, Black, Double, 5, false);
    new_overlines.next().is_some()
}

fn double_four(next: &Square, p: Point) -> bool {
    let new_fours = next.sequences_on(p, Black, Single, 4, true);
    distinctive(&mut new_fours.map(|p| p.0))
}

fn double_three(next: &Square, p: Point) -> bool {
    let new_threes = next.sequences_on(p, Black, Double, 3, true);
    if !distinctive(&mut new_threes.map(|p| p.0)) {
        return false;
    }
    let truthy_threes = next
        .sequences_on(p, Black, Double, 3, true)
        .filter(|(i, s)| {
            let eye = i.subsequent(s.eyes()).next().unwrap().to_point();
            forbidden(&next, eye).is_none()
        });
    distinctive(&mut truthy_threes.map(|p| p.0))
}

fn distinctive(indices: &mut impl Iterator<Item = Index>) -> bool {
    let first = indices.next();
    if first.is_none() {
        return false;
    }
    let next_to_first = first.unwrap().walk(1).unwrap();
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
         . . . . . . . . . . . . . . .
         . . o . . . . . . . . . . . .
         . o . o . . . . . . . . . . .
         . . o . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . o . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . o . . . . .
         . . . . o o o . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . o o . o o o .
        "
        .parse::<Square>()?;
        let result = forbiddens(&square);
        let expected = [
            (DoubleThree, Point(2, 12)),
            (DoubleFour, Point(8, 5)),
            (Overline, Point(10, 0)),
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
