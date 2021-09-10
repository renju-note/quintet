use super::bitboard::*;
use super::solver;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, depth_limit: u8) -> Option<Box<[u8]>> {
    let blacks = Points::try_from(blacks);
    let whites = Points::try_from(whites);
    if !blacks.is_ok() || !whites.is_ok() {
        return None;
    }
    let board = Board::from_points(&blacks.unwrap().into_vec(), &whites.unwrap().into_vec());
    let solution = solver::solve(depth_limit, &board, Player::from(black));
    solution.map(|ps| <Vec<u8>>::from(Points(ps)).into_boxed_slice())
}
