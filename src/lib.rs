extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

pub(crate) mod analyzer;
pub(crate) mod board;
pub mod encoding;
pub(crate) mod solver;

pub use analyzer::{Analyzer, ForbiddenKind, Row, RowKind};
pub use board::{Board, Point};
pub use solver::VCFSolver;

#[wasm_bindgen]
pub fn solve_vcf(blacks: &[u8], whites: &[u8], black: bool, depth_limit: u8) -> Option<Box<[u8]>> {
    let mut board = Board::new();
    for &code in blacks {
        let x = decode_x(code);
        let y = decode_y(code);
        board = board.put(true, &Point { x: x, y: y });
    }
    for &code in whites {
        let x = decode_x(code);
        let y = decode_y(code);
        board = board.put(false, &Point { x: x, y: y });
    }

    let mut solver = VCFSolver::new();
    match solver.solve(&board, black, depth_limit) {
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
    (x - 1) * 15 + (y - 1)
}

fn decode_x(code: u8) -> u8 {
    code / 15 + 1
}

fn decode_y(code: u8) -> u8 {
    code % 15 + 1
}
