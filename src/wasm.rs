use super::board::*;
use super::mate;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve(
    mode: u8,
    max_depth: u8,
    blacks: &[u8],
    whites: &[u8],
    black: bool,
    threat_limit: u8,
) -> Option<Box<[u8]>> {
    let mode = mate::SolveMode::try_from(mode);
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if mode.is_err() || blacks.is_err() || !whites.is_err() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let player = Player::from(black);
    let solution = mate::solve(mode.unwrap(), max_depth, &board, player, threat_limit);
    solution.map(|s| <Vec<u8>>::from(Points(s.path)).into_boxed_slice())
}

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    solve(0, max_depth, blacks, whites, black, max_depth)
}

#[wasm_bindgen]
pub fn solve_vct(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    solve(10, max_depth, blacks, whites, black, max_depth)
}

#[wasm_bindgen]
pub fn solve_vct_dfpn(
    blacks: &[u8],
    whites: &[u8],
    black: bool,
    max_depth: u8,
) -> Option<Box<[u8]>> {
    solve(16, max_depth, blacks, whites, black, max_depth)
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
    fn test_encode_xy() {
        assert_eq!(encode_xy(4, 5), 65);
    }

    #[test]
    fn test_decode_x() {
        assert_eq!(decode_x(65), 4);
    }

    #[test]
    fn test_decode_y() {
        assert_eq!(decode_y(65), 5);
    }
}
