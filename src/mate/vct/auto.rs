use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::board::{Board, Player};
use crate::mate::budget::NodeBudget;
use crate::mate::mate::Mate;
use crate::mate::outcome::{Outcome, Proof};
use crate::mate::solve::trivial_result;
use crate::mate::solver::Solver;

/// The limit [`VCTSolver::solve_auto`] searches at: as many attacks as a
/// `u8` holds, i.e. no limit. The proof numbers decide how deep to go.
pub const AUTO_LIMIT: u8 = u8::MAX;

/// The `threat_limit`s [`VCTSolver::solve_auto`] searches under, in the order
/// each round tries them.
///
/// On the benchmark (07) at [`AUTO_LIMIT`], 2 is the cheapest on most
/// positions; 3 is far cheaper on some (`vct_game_black_l07_3`: 1.6k nodes
/// against 47k) and the only one that finds `vct_fukumi`; 1 is the only
/// one that finds `vct_game_black_l08_2` within 5M nodes.
pub const AUTO_THREAT_LIMITS: [u8; 3] = [2, 3, 1];

/// The nodes each `threat_limit` gets in the first round. Every round
/// doubles it.
pub const AUTO_FIRST_SLICE: u64 = 1_000;

/// Searching with only a node budget.
impl<P: ThresholdPolicy> VCTSolver<P> {
    /// Looks for a VCT by `attacker` on `board` with no `limit` or
    /// `threat_limit` to choose: only `budget` bounds the work.
    ///
    /// Searches at [`AUTO_LIMIT`] under each of [`AUTO_THREAT_LIMITS`] in
    /// turn, a slice of the budget each, the slice doubling every round from
    /// [`AUTO_FIRST_SLICE`]. The first proof wins. A search that ran out of
    /// its slice goes on in the next round from what the tables kept, so the
    /// rounds lose little to starting over. One that finished without a
    /// proof is not asked again.
    ///
    /// `Disproven` means no VCT under any of [`AUTO_THREAT_LIMITS`], of any
    /// length. The mate found is not the shortest: a longer one is often
    /// cheaper to prove. A board that needs no search is answered as
    /// [`Solver::solve`] answers it.
    ///
    /// Like [`Solver::solve`] it opens one new generation, for the whole
    /// call: one per slice would let the memos drop what the previous rounds
    /// under the other `threat_limit`s found.
    pub fn solve_auto(
        &mut self,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Outcome<Mate> {
        if let Some(outcome) = trivial_result(board, attacker) {
            return outcome;
        }
        // As in `solve`, a proof is not abandoned for want of budget.
        let found = self
            .search_auto(board, attacker, budget)
            .and_then(|mut state| self.extract(&mut state, &mut NodeBudget::unlimited()));
        Outcome::found(found, budget)
    }

    /// [`Self::solve_auto`] without recovering the line, as
    /// [`Solver::prove`].
    pub fn prove_auto(
        &mut self,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Outcome<Proof> {
        if let Some(outcome) = trivial_result(board, attacker) {
            return outcome.to_proof();
        }
        let found = self
            .search_auto(board, attacker, budget)
            .and_then(|mut state| self.first_move(&mut state, &mut NodeBudget::unlimited()))
            .map(|first_move| Proof { first_move });
        Outcome::found(found, budget)
    }

    /// The rounds of [`Self::solve_auto`]: the root state of the first
    /// search that proved a VCT, `None` if none did.
    fn search_auto(
        &mut self,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Option<VCTState> {
        self.advance_generation();
        let mut disproven = [false; AUTO_THREAT_LIMITS.len()];
        let mut slice = AUTO_FIRST_SLICE;
        while !disproven.iter().all(|&d| d) {
            for (i, &threat_limit) in AUTO_THREAT_LIMITS.iter().enumerate() {
                if disproven[i] {
                    continue;
                }
                if budget.is_exhausted() {
                    return None;
                }
                let mut state = VCTState::from_board(board, attacker, AUTO_LIMIT, threat_limit);
                let mut part = budget.slice(slice);
                let proven = self.search(&mut state, &mut part);
                budget.absorb(&part);
                if proven {
                    return Some(state);
                }
                disproven[i] = !part.is_exhausted();
            }
            slice = slice.saturating_mul(2);
        }
        None
    }
}
