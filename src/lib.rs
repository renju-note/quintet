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
pub fn solve_vcf(code: &str, depth_limit: u8, shortest: bool) -> String {
    let points = match encoding::decode(&code.trim()) {
        Ok(points) => points,
        Err(_) => return "ERROR: decode failed".to_string(),
    };

    let mut board = Board::new();
    let mut black = true;
    for p in &points {
        board = board.put(black, p);
        black = !black
    }

    let mut solver = VCFSolver::new();
    let result = solver.solve(&board, black, depth_limit, shortest);
    match result {
        Some(ps) => match encoding::encode(&ps) {
            Ok(s) => "SOLVED: ".to_string() + &s,
            Err(s) => "ERROR: ".to_string() + &s,
        },
        None => "UNSOLVED: ".to_string(),
    }
}
