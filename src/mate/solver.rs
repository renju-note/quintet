use super::budget::NodeBudget;
use super::mate::Mate;
use super::outcome::{Outcome, Proof};
use super::solve::trivial_result;
use super::state::State;

/// What every mate solver looks like from outside: [`DFSSolver`],
/// [`IDDFSSolver`] and [`VCTSolver`] all implement it.
///
/// A solver owns memos that outlive one question, so it is asked as
/// `solve(&mut self, state, budget)` rather than as a function: the tables it
/// fills are most of its value and are meant to be kept. Each solver has its
/// own [`State`] — the game plus whatever the search needs on top of it —
/// which is `Self::State`.
///
/// [`solve`](Self::solve) is the whole of one question: it opens a new
/// generation in every memo and then runs the solver's own search. The two
/// halves are separate because a solver may sit inside another one — the
/// VCT solver asks its nested VCF solvers many times per search — and those
/// callers search directly, leaving the generation to the outermost
/// question. [`prove`](Self::prove) is the same question when the winning
/// line is not wanted, which a caller that only branches on the answer can
/// ask for less.
///
/// [`DFSSolver`]: crate::mate::DFSSolver
/// [`IDDFSSolver`]: crate::mate::IDDFSSolver
/// [`VCTSolver`]: crate::mate::VCTSolver
pub trait Solver {
    /// The state this solver searches over.
    type State: State;

    /// Answers one question: does the attacker in `state` have a mate within
    /// `state.limit()`? And if so, by which line.
    ///
    /// `state` is the attacker to move, as `VCFState::from_board` and
    /// `VCTState::from_board` make it. A board that needs no search is
    /// answered without one ([`trivial_result`]): a game already over is
    /// `Disproven`, an attacker with a four wins by making five.
    ///
    /// Opens a new generation first (see [`Self::advance_generation`]), so
    /// asking many questions of one solver keeps its memos bounded.
    ///
    /// [`trivial_result`]: crate::mate::trivial_result
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Outcome<Mate> {
        if let Some(outcome) = trivial_result(state.game().board(), state.attacker()) {
            return outcome;
        }
        self.advance_generation();
        let found = self.search_path(state, budget);
        Outcome::found(found, budget)
    }

    /// [`Self::solve`] without the winning line: whether there is a mate, and
    /// its first move.
    ///
    /// A solver that only proves a mate and has to walk its tables again to
    /// recover the line (the VCT solvers) skips that walk; one that builds
    /// the line as it searches (the VCF ones) answers as `solve` does. On a
    /// fresh solver the walk is a small part of the search; asked a question
    /// it has just answered, a VCT solver finds the proof in its tables at
    /// once, and the walk is most of the call (`examples/prove_cost.rs`).
    fn prove(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Outcome<Proof> {
        self.solve(state, budget).to_proof()
    }

    /// The search behind [`Self::solve`], on a board that needs a search,
    /// in the generation the caller has opened. `None` means either "no
    /// mate" or "gave up", told apart by `budget.is_exhausted()`.
    fn search_path(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;

    /// Forgets everything remembered from earlier searches.
    ///
    /// Nothing requires this: every memo is keyed by the position, the turn,
    /// the remaining limit *and* the attacker, so what a search leaves behind
    /// stays true whatever is asked next, and [`Self::solve`] keeps the
    /// memory bounded on its own. Use it to hand a solver on with a clean
    /// slate, or to give the memory back.
    fn clear(&mut self);

    /// Opens a new generation in every memo the solver keeps, which is how a
    /// reused solver's memory stays bounded (see `Memo::advance_generation`
    /// in `memo.rs`).
    /// [`Self::solve`] does it; a caller driving a solver's `search` by hand
    /// does it itself, once per question.
    fn advance_generation(&mut self);

    /// How many entries the solver's memos hold between them, for a caller
    /// sizing a carry capacity.
    fn memo_len(&self) -> usize;
}
