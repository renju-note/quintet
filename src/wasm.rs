use super::board::*;
use super::mate;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve(
    mode: u8,
    limit: u8,
    blacks: &[u8],
    whites: &[u8],
    black: bool,
    threat_limit: u8,
) -> Option<Box<[u8]>> {
    let mode = mate::SolveMode::try_from(mode);
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if mode.is_err() || blacks.is_err() || whites.is_err() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let player = Player::from(black);
    let limits = mate::SolveLimits::new(limit).with_threat_limit(threat_limit);
    let solution = mate::solve(mode.unwrap(), &board, player, limits).into_mate();
    solution.map(|s| <Vec<u8>>::from(Points(s.path)).into_boxed_slice())
}

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, limit: u8) -> Option<Box<[u8]>> {
    solve(0, limit, blacks, whites, black, limit)
}

#[wasm_bindgen]
pub fn solve_vct(blacks: &[u8], whites: &[u8], black: bool, limit: u8) -> Option<Box<[u8]>> {
    solve(10, limit, blacks, whites, black, limit)
}

#[wasm_bindgen]
pub fn solve_vct_dfpn(blacks: &[u8], whites: &[u8], black: bool, limit: u8) -> Option<Box<[u8]>> {
    solve(16, limit, blacks, whites, black, limit)
}

#[wasm_bindgen]
pub fn encode_xy(x: u8, y: u8) -> u8 {
    Point(x, y).into()
}

#[wasm_bindgen]
pub fn decode_x(code: u8) -> u8 {
    Point::try_from(code).unwrap().0
}

#[wasm_bindgen]
pub fn decode_y(code: u8) -> u8 {
    Point::try_from(code).unwrap().1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        // E6: `x * 15 + y`, the same code as `u8::from(Point)`.
        assert_eq!(encode_xy(4, 5), 65);
        assert_eq!((decode_x(65), decode_y(65)), (4, 5));
    }

    #[test]
    fn test_solve() {
        // Black H8-I8-J8 with White far away: Black has a VCF.
        let blacks = [encode_xy(7, 7), encode_xy(8, 7), encode_xy(9, 7)];
        let whites = [encode_xy(0, 0), encode_xy(0, 1)];

        // The path comes back as point codes, the same as `mate::solve`'s.
        let board = Board::from_stones(
            &Points::try_from(&blacks[..]).unwrap(),
            &Points::try_from(&whites[..]).unwrap(),
        );
        let limits = mate::SolveLimits::new(3).with_threat_limit(3);
        let expected = mate::solve(mate::SolveMode::VCFDFS, &board, Player::Black, limits)
            .into_mate()
            .unwrap();
        let expected = <Vec<u8>>::from(Points(expected.path));
        let result = solve_vcf(&blacks, &whites, true, 3).unwrap();
        assert_eq!(result.to_vec(), expected);
        assert_eq!(solve(0, 3, &blacks, &whites, true, 3), Some(result));

        // No mate for White.
        assert_eq!(solve_vcf(&blacks, &whites, false, 3), None);

        // Bad input is `None` rather than a panic.
        assert_eq!(solve(2, 3, &blacks, &whites, true, 3), None);
        assert_eq!(solve(0, 3, &[225], &whites, true, 3), None);
        assert_eq!(solve(0, 3, &blacks, &[255], true, 3), None);
    }
}
