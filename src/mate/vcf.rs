//! VCF — victory by continuous fours.
//!
//! The attacker makes a four with every move, so each defender reply is
//! forced and the tree is a chain of `(attack, defence)` pairs. [`VCFState`]
//! generates those pairs from the `Sword` rows on the board (three
//! stones with two empty eyes: play one eye, the other is the block);
//! [`DFSSolver`] searches them depth-first, memoizing the positions it has
//! shown to be deadends; [`IDDFSSolver`] runs it at increasing limits.
//!
//! Besides answering `SolveMode::VCFDFS`, this is the subroutine the VCT
//! solver asks whether a move is a threat: would the other side have a VCF
//! if I passed?

mod dfs;
mod iddfs;
mod state;

pub use dfs::DFSSolver;
pub use iddfs::IDDFSSolver;
pub use state::VCFState;
