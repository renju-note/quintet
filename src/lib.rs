pub mod board;
pub mod feature;
pub mod mate;
pub mod wasm;

pub use mate::{solve, solve_with_stats};
