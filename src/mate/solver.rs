use super::budget::NodeBudget;
use super::mate::Mate;
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
/// generation in every memo and then runs the solver's own `search`. The two
/// halves are separate because a solver may sit inside another one — the
/// VCT solver asks its nested VCF solvers many times per search — and those
/// callers use `search`, leaving the generation to the outermost question.
///
/// [`DFSSolver`]: crate::mate::DFSSolver
/// [`IDDFSSolver`]: crate::mate::IDDFSSolver
/// [`VCTSolver`]: crate::mate::VCTSolver
pub trait Solver {
    /// The state this solver searches over.
    type State: State;

    /// Answers one question: does the attacker in `state` have a mate within
    /// `state.limit()`? `None` means either "no mate" or "gave up"; the two
    /// are told apart by `budget.is_exhausted()`.
    ///
    /// Opens a new generation first (see [`Self::advance_generation`]), so
    /// asking many questions of one solver keeps its memos bounded.
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;

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
