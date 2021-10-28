use super::fundamentals::*;
use super::point::*;
use super::row::*;
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
    next.put(Player::Black, p);
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

fn distinctive(rows: &Vec<Row>) -> bool {
    let first = &rows[0];
    for row in rows.iter().skip(1) {
        if !first.adjacent(row) {
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
            ---------------
            --o------------
            -o-o-----------
            --o------------
            ---------------
            ---------------
            -----------o---
            ----------o----
            ---------o-----
            ----ooo--------
            ---------------
            ---------------
            ---------------
            ---------------
            --------oo-ooo-
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
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -------o-------
            ------o-o------
            -------o-------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -----o-o-------
            ----o--o-------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleThree));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -------o-------
            ----x-o-o-x----
            -------o-------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            -------x-------
            ---------------
            -------ooox----
            ------x--------
            ------oo-------
            ------o-oox----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
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
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ------x--------
            --o--o-o-------
            -----o--oo-----
            ----o------oo-x
            --------oxoo---
            ----------o----
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(6, 7));
        assert_eq!(result, None);

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ------x--------
            --o--o-o-------
            -----o--oo-----
            ----o------oo--
            --------oxoo---
            ----------o----
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(6, 7));
        assert_eq!(result, Some(DoubleThree));

        Ok(())
    }

    #[test]
    fn test_double_four() -> Result<(), String> {
        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -----o---------
            ------o--------
            -----oo-o------
            --------o------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ----o-o-o-o----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ----oo--o-oo---
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---ooo---ooo---
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(DoubleFour));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -----oo-o------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        let square = "
            ---------------
            ---------------
            ---------------
            ---o-----------
            ---------------
            -----o---------
            ------o--------
            -----oo-o------
            --------o------
            ---------x-----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_overline() -> Result<(), String> {
        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ----ooo-oo-----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, Some(Overline));

        let square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ----ooo--oo----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let result = forbidden(&square, Point(7, 7));
        assert_eq!(result, None);

        Ok(())
    }
}
