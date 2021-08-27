extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

pub mod bitboard;
pub mod encoding;
pub mod solver;

use bitboard::{Board, Point, BOARD_SIZE};

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, depth_limit: u8) -> Option<Box<[u8]>> {
    let mut board = Board::new();
    for &code in blacks {
        let x = decode_x(code);
        let y = decode_y(code);
        board.put(true, Point { x: x, y: y });
    }
    for &code in whites {
        let x = decode_x(code);
        let y = decode_y(code);
        board.put(false, Point { x: x, y: y });
    }

    match solver::solve(depth_limit, &mut board, black) {
        Some(ps) => Some(
            ps.iter()
                .map(|p| encode_xy(p.x, p.y))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        None => None,
    }
}

fn encode_xy(x: u8, y: u8) -> u8 {
    x * BOARD_SIZE + y
}

fn decode_x(code: u8) -> u8 {
    code / BOARD_SIZE
}

fn decode_y(code: u8) -> u8 {
    code % BOARD_SIZE
}
