use super::board::*;
use super::mate;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve(
    kind_code: u8,
    max_depth: u8,
    blacks: &[u8],
    whites: &[u8],
    black: bool,
) -> Option<Box<[u8]>> {
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if !blacks.is_ok() || !whites.is_ok() {
        return None;
    }
    let board = Board::from_stones(&blacks.unwrap(), &whites.unwrap());
    let kind = solver_kind(kind_code);
    let solution = mate::solve(kind, max_depth, &board, Player::from(black));
    solution.map(|ps| <Vec<u8>>::from(Points(ps)).into_boxed_slice())
}

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    solve(0, max_depth, blacks, whites, black)
}

#[wasm_bindgen]
pub fn solve_vct(blacks: &[u8], whites: &[u8], black: bool, max_depth: u8) -> Option<Box<[u8]>> {
    solve(1, max_depth, blacks, whites, black)
}

#[wasm_bindgen]
pub fn solve_vct_dfpn(
    blacks: &[u8],
    whites: &[u8],
    black: bool,
    max_depth: u8,
) -> Option<Box<[u8]>> {
    solve(2, max_depth, blacks, whites, black)
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

fn solver_kind(code: u8) -> mate::SolverKind {
    match code {
        0 => mate::SolverKind::VCF,
        1 => mate::SolverKind::VCTDFS,
        2 => mate::SolverKind::VCTDFPN,
        _ => mate::SolverKind::VCF,
    }
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
