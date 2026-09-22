use super::state::VCFState;
use crate::board::*;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::memo::Memo;
use crate::mate::solver::Solver;
use crate::mate::state::State;

/// How many deadends a solver carries into a new search before it starts
/// dropping what older searches left behind. See [`Memo`].
pub const DEFAULT_CARRY_CAPACITY: usize = 1 << 16;

/// Depth-first VCF search.
///
/// The attacker plays four-making moves only, so every defender reply is
/// forced: the tree is a chain of `(attack, defence)` pairs and the search is
/// a plain DFS over them. `search` is the recursive routine; [`Solver::solve`]
/// wraps it in a generation.
pub struct DFSSolver {
    /// For each position already shown to have no VCF, the largest limit it
    /// was shown for — and so, since no VCF within `limit` is no VCF within
    /// less, every limit up to it.
    ///
    /// Keyed by [`Key::position`](crate::mate::Key::position) alone. Nothing this solver generates
    /// depends on `limit` (`move_pairs` and `neighbor_move_pairs` read only
    /// the board), so the tree at one limit is the tree at a larger one cut
    /// short, and the bound holds exactly. `search` is only ever called with
    /// the attacker to move, so the attacker in the position pins the turn.
    deadends: Memo<u8>,
}

impl DFSSolver {
    pub fn init() -> Self {
        Self::with_carry_capacity(DEFAULT_CARRY_CAPACITY)
    }

    pub fn with_carry_capacity(carry_capacity: usize) -> Self {
        Self {
            deadends: Memo::new(carry_capacity),
        }
    }

    /// The search itself, with the attacker to move. Unlike
    /// [`Solver::solve`] it opens no generation, so a caller that asks it
    /// many times within one question — [`IDDFSSolver`] deepening, the VCT
    /// solver evaluating threats — does not churn the memo.
    ///
    /// [`IDDFSSolver`]: super::IDDFSSolver
    pub fn search(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        if state.limit == 0 {
            return None;
        }
        if !budget.consume() {
            return None;
        }

        let key = state.key();
        if self
            .deadends
            .get(key.position)
            .is_some_and(|&l| key.limit <= l)
        {
            return None;
        }
        let result = self.search_move_pairs(state, budget);
        // A search that gave up proves nothing, so it must not be memoized.
        if result.is_none() && !budget.is_exhausted() {
            let known = self.deadends.get(key.position).copied().unwrap_or(0);
            self.deadends.insert(key.position, known.max(key.limit));
        }
        result
    }

    fn search_move_pairs(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => None,
                Forced(p) => state
                    .forced_move_pair(p)
                    .and_then(|(a, d)| self.search_attack(state, a, d, budget)),
            };
        }

        let neighbor_pairs = state.neighbor_move_pairs();
        for &(attack, defence) in &neighbor_pairs {
            let result = self.search_attack(state, attack, defence, budget);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        let pairs = state.move_pairs();
        for &(attack, defence) in &pairs {
            if neighbor_pairs.iter().any(|(a, _)| *a == attack) {
                continue;
            }
            let result = self.search_attack(state, attack, defence, budget);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        None
    }

    fn search_attack(
        &mut self,
        state: &mut VCFState,
        attack: Point,
        defence: Point,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if state.is_forbidden_move(attack) {
            return None;
        }

        state.into_play(Some(attack), |s| {
            self.search_defence(s, defence, budget)
                .map(|m| m.unshift(attack))
        })
    }

    fn search_defence(
        &mut self,
        state: &mut VCFState,
        defence: Point,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if let Some(Defeated(end)) = state.check_event() {
            return Some(Mate::new(end, vec![]));
        }

        state.into_play(Some(defence), |s| {
            self.search(s, budget).map(|m| m.unshift(defence))
        })
    }
}

impl Solver for DFSSolver {
    type State = VCFState;

    fn solve(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        self.advance_generation();
        self.search(state, budget)
    }

    fn clear(&mut self) {
        self.deadends.clear();
    }

    fn advance_generation(&mut self) {
        self.deadends.advance_generation();
    }

    fn memo_len(&self) -> usize {
        self.deadends.len()
    }
}
