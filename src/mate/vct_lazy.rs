/*
Lazy VCT solver is earlier experimental implementation and not maintained well.
The idea was inspired by following paper.

長井歩. "難解な必至問題を解くアルゴリズムとその実装." ゲームプログラミングワークショップ 2011 論文集 2011.6 (2011): 1-8.
*/

mod generator;
mod helper;
mod proof;
mod resolver;
mod searcher;
mod solver;
mod state;
mod traverser;

pub use solver::LazyVCTSolver;
pub use state::LazyVCTState;
