use super::grid::*;
use super::player::*;
use super::point::*;
use super::sequence::*;

pub fn forbiddens(g: &Grid) -> Vec<(ForbiddenKind, Point)> {
    g.empties()
        .filter_map(|p| forbidden_strict(g, p).map(|k| (k, p)))
        .collect()
}

pub fn forbidden_strict(g: &Grid, p: Point) -> Option<ForbiddenKind> {
    if g.stone(p).is_some() {
        return None;
    }
    let mut fours = g.sequences_on(p, Black, Four);
    if fours.next().is_some() {
        return None;
    }
    forbidden(g, p)
}

pub fn forbidden(g: &Grid, p: Point) -> Option<ForbiddenKind> {
    if overline(g, p) {
        Some(Overline)
    } else if double_four(g, p) {
        Some(DoubleFour)
    } else if double_three(g, p) {
        Some(DoubleThree)
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub use ForbiddenKind::*;

fn overline(g: &Grid, p: Point) -> bool {
    let mut overlinings = g.sequences_on(p, Black, Overlining);
    overlinings.next().is_some()
}

fn double_four(g: &Grid, p: Point) -> bool {
    let swords = g.sequences_on(p, Black, Sword);
    distinctive(&mut swords.map(|s| s.start_index()))
}

fn double_three(g: &Grid, p: Point) -> bool {
    let twos = g.sequences_on(p, Black, Two);
    if !distinctive(&mut twos.map(|s| s.start_index())) {
        return false;
    }
    let mut next = g.clone();
    next.put_mut(Black, p);
    truthy_double_three(&next, p)
}

fn truthy_double_three(next: &Grid, p: Point) -> bool {
    let truthy_threes = next.sequences_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();
        forbidden_strict(next, eye).is_none()
    });
    distinctive(&mut truthy_threes.map(|s| s.start_index()))
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
    use super::*;

    #[test]
    fn test_forbiddens() -> Result<(), String> {
        let grid = "
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
        .parse::<Grid>()?;
        // M13 would be a double-three, but it also completes the five
        // K11-O15, which wins, so it is not reported.
        let result = forbiddens(&grid);
        let expected = [
            (DoubleThree, Point(2, 12)), // C13
            (Overline, Point(3, 0)),     // D1
            (DoubleFour, Point(4, 4)),   // E5
        ];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_double_three_eye_makes_five() -> Result<(), String> {
        // H8 makes two threes. The eye of the horizontal one (G8) is a
        // double-four, but it also completes a five (G4-G8), so it is a legal
        // move and the three counts as a real one (rule 9.2 / 9.3).
        let grid = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . o . . . . . . . . . .
         . . . . . o . o . . . . . . .
         . . . . . o . . o . . . . . .
         . . . . . . o o . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Grid>()?;
        let mut next = grid.clone();
        next.put_mut(Black, Point(7, 7));
        assert_eq!(forbidden(&next, Point(6, 7)), Some(DoubleFour));
        assert_eq!(forbidden_strict(&next, Point(6, 7)), None);

        assert_eq!(forbidden(&grid, Point(7, 7)), Some(DoubleThree));

        Ok(())
    }

    #[test]
    fn test_double_three_nested_eye_makes_five() -> Result<(), String> {
        // H8 makes a four (E8-H8, eye I8) and two threes (H6-H9 via H7,
        // F6-I9 via I9). Whether H8 is a double-three depends on the nested
        // check: H7 would make two threes (H7-K10 via I8, G8-J5 via I6), and
        // I8 is a double-four but also completes the five E8-I8, so I8 is a
        // legal move, H7 is a real double-three (forbidden), the vertical
        // three through H8 is fake, and H8 is a legal four-three.
        // Checking I8 with `forbidden` instead of `forbidden_strict` flips
        // every step and wrongly reports H8 as a double-three.
        let grid = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . x . . . . . . . . . .
         . . . . . o . x . . . . . . .
         . . . . . . o . . . o . . . .
         . . . . x . x o . o . . . . .
         . . . x o o o . . . . . . . .
         . . . . . x o . . . . . . . .
         . . . . x o . o . . . . . . .
         . . . . . . . . x o . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Grid>()?;
        let h8 = Point(7, 7);
        let h7 = Point(7, 6);
        let i8 = Point(8, 7);

        let mut after_h8 = grid.clone();
        after_h8.put_mut(Black, h8);
        let mut after_h7 = after_h8.clone();
        after_h7.put_mut(Black, h7);
        assert_eq!(forbidden(&after_h7, i8), Some(DoubleFour));
        assert_eq!(forbidden_strict(&after_h7, i8), None);
        assert_eq!(forbidden(&after_h8, h7), Some(DoubleThree));

        assert_eq!(forbidden(&grid, h8), None);

        Ok(())
    }

    #[test]
    fn test_double_three() -> Result<(), String> {
        // H8 makes two open threes, H7-H9 and G8-I8.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        // Split threes count too: H5-H6-_-H8 and E5-F6-_-H8.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        // White's E8 and K8 leave G8-I8 no way to an open four, so H8 only
        // makes one three.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // H8 completes the five F8-J8: nothing else is looked at.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // E8-F8-_-H8-_-J8-K8: every four H8 could lead to is an overline, so
        // there is no three.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // A three only counts if its open-four point is legal. H8 and I7 each
        // make two threes, but one of them (H7-H10, I6-I10) can only be
        // completed at a double-four (H6, I8), so both moves are legal. J7
        // makes two real threes.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);
        let result = forbidden(&grid, Point(8, 6));
        assert_eq!(result, None);
        let result = forbidden(&grid, Point(9, 6));
        assert_eq!(result, Some(DoubleThree));

        // following examples are from https://twitter.com/tanaseY/status/944521796585373696
        //
        // The two positions differ only in White's O6, which decides three
        // levels down whether G8 is a double-three. With O6, K6 only makes
        // one real three (K6-N6 needs J6, a double-four), so K6 is legal, I8
        // is a double-three, and G8's horizontal three (via I8) is fake.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(6, 7));
        assert_eq!(result, None);

        // Without O6, K6-O6 is a second real three: K6 is a double-three, so
        // I8 is legal and G8 makes two real threes.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(6, 7));
        assert_eq!(result, Some(DoubleThree));

        Ok(())
    }

    #[test]
    fn test_double_four() -> Result<(), String> {
        // H8 makes fours in two directions, F8-I8 and F10-I7.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        // Two fours on one line: E8-_-G8-H8-I8-_-K8.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        // Two fours on one line: E8-F8-_-H8-I8-_-K8-L8.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        // Two fours on one line: D8-F8 and J8-L8, eyes G8 and I8.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        // G8 or I8 would each make six, so H8 makes no four at all.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // The same in two directions: both fours would complete as overlines.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // H8 is too far from either three to make a four.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // H8 is next to neither four.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // A single four is fine.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        // The first position with D12 and White's J6: the diagonal four could
        // only be completed as an overline (D12-I7), leaving one four.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_overline() -> Result<(), String> {
        // H8 fills E8-J8, six in a row.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, Some(Overline));

        // Only E8-H8 would be connected: no six.
        let grid = "
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
        .parse::<Grid>()?;
        let result = forbidden(&grid, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }
}
