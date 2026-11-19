use super::budget::NodeBudget;
use super::mate::Mate;
use crate::board::Point;

/// What a search concluded: a mate (and what is reported about it), none,
/// or nothing known.
///
/// `Disproven` and `Unknown` are both "no mate to report", but only
/// `Disproven` says there is none: `Unknown` means the search ran out of
/// budget, or was never run, and the position is still open. A caller that
/// reads `Unknown` as `Disproven` walks into mates.
///
/// The solvers answer with an `Outcome<Proof>` ([`Solver::prove`]) when
/// only the answer is wanted, and with an `Outcome<Mate>`
/// ([`Solver::solve`], [`SolveResult`]) when the winning line is wanted too.
///
/// [`Solver::prove`]: crate::mate::Solver::prove
/// [`Solver::solve`]: crate::mate::Solver::solve
/// [`SolveResult`]: crate::mate::SolveResult
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Outcome<T> {
    /// The attacker wins; `T` tells how.
    Proven(T),
    /// The attacker has no mate within the limits.
    Disproven,
    /// The budget ran out first, or the search was never run; nothing is
    /// known.
    Unknown,
}

impl<T> Outcome<T> {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven(_))
    }

    pub fn is_disproven(&self) -> bool {
        matches!(self, Self::Disproven)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// What was reported with the proof, if the search found one.
    pub fn proven(&self) -> Option<&T> {
        match self {
            Self::Proven(t) => Some(t),
            _ => None,
        }
    }

    /// [`Self::proven`], by value.
    pub fn into_proven(self) -> Option<T> {
        match self {
            Self::Proven(t) => Some(t),
            _ => None,
        }
    }

    /// What a search that ended with `budget` concluded from what it
    /// found: a search that found nothing has shown there is nothing only if
    /// it did not run out of budget.
    pub(crate) fn found(found: Option<T>, budget: &NodeBudget) -> Self {
        match found {
            Some(t) => Self::Proven(t),
            None if budget.is_exhausted() => Self::Unknown,
            None => Self::Disproven,
        }
    }

    /// The same outcome, with `f` applied to what a proof reports.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Outcome<U> {
        match self {
            Self::Proven(t) => Outcome::Proven(f(t)),
            Self::Disproven => Outcome::Disproven,
            Self::Unknown => Outcome::Unknown,
        }
    }
}

impl<T> From<Outcome<T>> for Option<T> {
    /// What a proof reports, `None` for no mate and for nothing known alike.
    fn from(outcome: Outcome<T>) -> Self {
        outcome.into_proven()
    }
}

impl Outcome<Mate> {
    /// The winning line, if the search found one.
    pub fn mate(&self) -> Option<&Mate> {
        self.proven()
    }

    /// [`Self::mate`], by value.
    pub fn into_mate(self) -> Option<Mate> {
        self.into_proven()
    }

    /// What [`Solver::prove`](crate::mate::Solver::prove) would have
    /// reported about the same proof.
    pub fn to_proof(&self) -> Outcome<Proof> {
        match self {
            Self::Proven(mate) => Outcome::Proven(Proof::of(mate)),
            Self::Disproven => Outcome::Disproven,
            Self::Unknown => Outcome::Unknown,
        }
    }
}

/// What [`Solver::prove`](crate::mate::Solver::prove) reports about a
/// proof: what can be read off without recovering the winning line.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Proof {
    /// The attacker's first move of the winning line: the same move that
    /// [`Solver::solve`](crate::mate::Solver::solve) would put first, asked
    /// the same question of a solver in the same state.
    pub first_move: Point,
}

impl Proof {
    /// The proof `mate` stands for.
    ///
    /// # Panics
    ///
    /// If `mate` has no moves, which no solver reports.
    pub fn of(mate: &Mate) -> Self {
        Self {
            first_move: mate.path[0],
        }
    }
}
