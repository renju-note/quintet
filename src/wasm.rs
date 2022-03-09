use super::board::*;
use super::mate;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if !blacks.is_ok() || !whites.is_ok() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let solution = mate::solve_vcf(&board, Player::from(black), max_depth);
    solution.map(|ps| <Vec<u8>>::from(Points(ps)).into_boxed_slice())
}

#[wasm_bindgen]
pub fn solve_vct(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if !blacks.is_ok() || !whites.is_ok() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let solution = mate::solve_vct(&board, Player::from(black), max_depth);
    solution.map(|ps| <Vec<u8>>::from(Points(ps)).into_boxed_slice())
}

#[wasm_bindgen]
pub fn solve_vct_pn(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if !blacks.is_ok() || !whites.is_ok() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let solution = mate::solve_vct_pn(&board, Player::from(black), max_depth);
    solution.map(|ps| <Vec<u8>>::from(Points(ps)).into_boxed_slice())
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
