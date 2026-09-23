//! The mate solvers: VCF (victory by continuous fours) and VCT (victory by
//! continuous threats).
//!
//! [`solve`] answers "does `attacker` have a mate on this board" in one
//! call. The pieces it is built from are public too, so that
//! a caller that asks many related questions can keep one solver — and the
//! tables it has filled — alive across them. They fit together like this:
//!
//! - [`Game`] is a board plus the moves played on it; a move may be `None`,
//!   a pass, which is how "would the opponent have a mate if I did nothing"
//!   is asked.
//! - A [`State`] is a `Game` plus what a search needs on top of it: whose
//!   side the search is for, the remaining limit, and for VCT the potential
//!   field. [`VCFState`] and [`VCTState`] are the two. A state also knows its
//!   [`Key`], what every memo is keyed by.
//! - A [`Solver`] searches a state for a [`Mate`]. [`DFSSolver`] and
//!   [`IDDFSSolver`] search for VCFs, [`VCTSolver`] for VCTs, and the VCT
//!   solver has the VCF ones inside it, for telling threats apart from
//!   ordinary moves. Every solver memoizes across calls, keyed by the
//!   position, the turn, the remaining limit *and* the attacker, so one
//!   solver can be asked about both sides without its answers being
//!   confused. Each [`Solver::solve`] opens a new generation, which settles
//!   the memos at about two searches' worth however many searches are asked;
//!   [`Solver::clear`] throws the lot away, which nothing requires.
//! - [`NodeBudget`] bounds how much work a search may do, in nodes. There is
//!   no clock: everything here compiles for `wasm32-unknown-unknown`, so a
//!   caller with a time control converts it into a number of nodes itself.
//!
//! [`VCTState::threat_defences`] lists the moves worth trying against a
//! threat that was found, for a caller that wants to defend.

mod budget;
mod game;
#[allow(clippy::module_inception)]
mod mate;
mod memo;
mod solve;
mod solver;
mod state;
mod vcf;
mod vct;

pub use budget::NodeBudget;
pub use game::{End, Event, Game};
pub use mate::Mate;
pub use solve::{DEFAULT_DEFENDER_VCF_DEPTH, SolveLimits, SolveMode, SolveResult, solve};
pub use solver::Solver;
pub use state::{Key, State};
pub use vcf::{DFSSolver, IDDFSSolver, VCFState};
pub use vct::{
    DFPNSThreshold, DFPNSVCTSolver, DFSThreshold, DFSVCTSolver, Node, PNSThreshold, PNSVCTSolver,
    ThresholdPolicy, VCTSolver, VCTState,
};
