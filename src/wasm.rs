use super::bitboard::*;
use super::solver;
use std::convert::{From, TryFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, depth_limit: u8) -> Option<Box<[u8]>> {
    let mut board = Board::new();
    let blacks = blacks.iter().map(|c| Point::try_from(c)).flatten();
    for p in blacks {
        board.put(Player::Black, p);
    }
    let whites = whites.iter().map(|c| Point::try_from(c)).flatten();
    for p in whites {
        board.put(Player::White, p);
    }
    solver::solve(depth_limit, &mut board, Player::new(black)).map(|ps| {
        ps.iter()
            .map(|p| u8::from(p))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    })
}
