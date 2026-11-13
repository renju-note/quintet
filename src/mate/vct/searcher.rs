use super::generator::Candidate;
use super::generator::Candidates::*;
use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::state::State;
use crate::mate::vct::proof::*;

/// AND/OR search over proof numbers.
///
/// `search_attacks` / `search_defences` evaluate one node (OR / AND
/// respectively); `expand_attacks` / `expand_defences` are the loop that keeps
/// descending into the most-proving child until the node's numbers reach the
/// threshold handed down by the parent; `select_attack` / `select_defence`
/// pick that child, expanding nothing.
///
/// An expansion reads its children's numbers from the tables once and
/// then keeps them itself (on `children`, one stack for all the expansions
/// under way, rather than a `Vec` of its own), writing back only the child it has just
/// searched, rather than looking every child up again at each step.
/// Nothing else can change them in the meantime: every position under
/// that child holds its stone, which none of its siblings does (the search
/// never passes).
impl<P: ThresholdPolicy> VCTSolver<P> {
    /// Runs the search and reports whether the attacker wins. `false` also
    /// covers "gave up": the caller tells the two apart by
    /// `budget.is_exhausted()`.
    ///
    /// A proof is never spurious, so a `true` that was reached just before the
    /// budget ran out still holds.
    pub fn search(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> bool {
        if state.limit() == 0 {
            return false;
        }
        self.search_attacks(state, PnDn::no_threshold(), budget)
            .is_proven()
    }

    pub fn search_attacks(
        &mut self,
        state: &mut VCTState,
        threshold: PnDn,
        budget: &mut NodeBudget,
    ) -> PnDn {
        if !budget.consume() {
            return PnDn::unknown();
        }

        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => PnDn::disproven(state.limit()),
                Forced(next_move) => {
                    let attacks = &[Candidate::new(next_move, 1)];
                    self.expand_attacks(state, attacks, threshold, budget).node
                }
            };
        }

        let attacks = match self.generate_attacks(state, budget) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_attacks(state, &attacks, threshold, budget).node
    }

    pub fn search_defences(
        &mut self,
        state: &mut VCTState,
        threshold: PnDn,
        budget: &mut NodeBudget,
    ) -> PnDn {
        if !budget.consume() {
            return PnDn::unknown();
        }

        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => PnDn::proven(state.limit()),
                Forced(next_move) => {
                    if state.limit() <= 1 {
                        PnDn::disproven(state.limit())
                    } else {
                        let defences = &[Candidate::new(next_move, 1)];
                        self.expand_defences(state, defences, threshold, budget)
                            .node
                    }
                }
            };
        }

        if state.limit() <= 1 {
            return PnDn::disproven(state.limit());
        }

        let defences = match self.generate_defences(state, budget) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_defences(state, &defences, threshold, budget)
            .node
    }

    fn expand_attacks(
        &mut self,
        state: &mut VCTState,
        attacks: &[Candidate],
        threshold: PnDn,
        budget: &mut NodeBudget,
    ) -> Selection {
        let base = self.children.len();
        self.attacker_table
            .lookup_children(state, attacks, &mut self.children);
        for (attack, child) in attacks.iter().zip(&mut self.children[base..]) {
            let forbidden = match child {
                // Found forbidden before, at the check below: disproven
                // again before the numbers are taken, so that they are the
                // numbers this node had when it was last searched.
                None => self
                    .forbidden_attacks
                    .get(state.move_key(attack.point))
                    .is_some(),
                // A child the table has may have been reached by another
                // order of the moves, one that played the point while it was
                // not forbidden (in this search or one before it on the same
                // solver). Its numbers hold for that position, but not for a
                // move Black cannot play now: a proven one would prove this
                // node through a forbidden move, before the check below for
                // unvisited children is ever reached.
                Some(c) => c.dn != 0 && state.is_forbidden_move(attack.point),
            };
            if forbidden {
                *child = Some(PnDn::disproven(state.limit()));
            }
        }
        let selection = loop {
            let children = &self.children[base..];
            let selection = Self::select_attack(state.limit(), attacks, children);
            if Self::exceeds_threshold(selection.node, threshold) {
                break selection;
            }
            if budget.is_exhausted() {
                break selection;
            }
            let best = attacks[selection.best].point;
            // Few attacks are forbidden, so each is only asked about when
            // it is first searched rather than when it is generated
            // (`compute_attacks`). A forbidden one is disproven: the
            // attacker cannot play it. The disproof is kept by the move
            // (`forbidden_attacks`), not in the table: the table is keyed by
            // the position, and another order of the moves may reach the
            // same position with the point played while it was not
            // forbidden. A disproof stored there would deny that order its
            // line.
            if selection.unvisited && state.is_forbidden_move(best) {
                self.forbidden_attacks.insert(state.move_key(best), ());
                self.children[base + selection.best] = Some(PnDn::disproven(state.limit()));
                continue;
            }
            let next_threshold = P::next_threshold_attack(&selection, threshold);
            let result = state.with_move(Some(best), |child| {
                let result = self.search_defences(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.attacker_table.insert(child, result);
                }
                result
            });
            // The loop ends at the next check if the budget ran out, so the
            // result is only read back when it is also in the table.
            self.children[base + selection.best] = Some(result);
        };
        self.children.truncate(base);
        selection
    }

    fn expand_defences(
        &mut self,
        state: &mut VCTState,
        defences: &[Candidate],
        threshold: PnDn,
        budget: &mut NodeBudget,
    ) -> Selection {
        let base = self.children.len();
        self.defender_table
            .lookup_children(state, defences, &mut self.children);
        let selection = loop {
            let children = &self.children[base..];
            let selection = Self::select_defence(state.limit(), defences, children);
            if Self::exceeds_threshold(selection.node, threshold) {
                break selection;
            }
            if budget.is_exhausted() {
                break selection;
            }
            let best = defences[selection.best].point;
            let next_threshold = P::next_threshold_defence(&selection, threshold);
            let result = state.with_move(Some(best), |child| {
                let result = self.search_attacks(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.defender_table.insert(child, result);
                }
                result
            });
            self.children[base + selection.best] = Some(result);
        };
        self.children.truncate(base);
        selection
    }

    /// Chooses among `attacks` by their numbers in `children`, as the
    /// attacker table had them (`None` for a child not in it).
    fn select_attack(limit: u8, attacks: &[Candidate], children: &[Option<PnDn>]) -> Selection {
        let mut best = 0;
        let mut unvisited = false;
        let mut node = PnDn::disproven(limit);
        let mut best_child = PnDn::disproven(limit);
        let mut second_child = PnDn::disproven(limit);
        for (i, (attack, &maybe_child)) in attacks.iter().zip(children).enumerate() {
            let child = maybe_child.unwrap_or(PnDn::unexpanded_defence(attack.estimate, limit));
            node = node.min_pn_sum_dn(child);
            if child.pn < best_child.pn {
                best = i;
                unvisited = maybe_child.is_none();
                second_child = best_child;
                best_child = child;
            } else if child.pn < second_child.pn {
                second_child = child;
            }
            if node.pn == 0 {
                node.dn = INF;
                break;
            }
        }
        Selection {
            best,
            unvisited,
            node,
            best_child,
            second_child,
        }
    }

    /// [`Self::select_attack`] for the defender, with `children` as the
    /// defender table had them.
    fn select_defence(limit: u8, defences: &[Candidate], children: &[Option<PnDn>]) -> Selection {
        let mut best = 0;
        let mut unvisited = false;
        let mut node = PnDn::proven(limit - 1);
        let mut best_child = PnDn::proven(limit - 1);
        let mut second_child = PnDn::proven(limit - 1);
        for (i, (defence, &maybe_child)) in defences.iter().zip(children).enumerate() {
            let child = maybe_child.unwrap_or(PnDn::unexpanded_attack(defence.estimate, limit - 1));
            node = node.min_dn_sum_pn(child);
            if child.dn < best_child.dn {
                best = i;
                unvisited = maybe_child.is_none();
                second_child = best_child;
                best_child = child;
            } else if child.dn < second_child.dn {
                second_child = child;
            }
            if node.dn == 0 {
                node.pn = INF;
                break;
            }
        }
        Selection {
            best,
            unvisited,
            node,
            best_child,
            second_child,
        }
    }

    fn exceeds_threshold(node: PnDn, threshold: PnDn) -> bool {
        node.pn >= threshold.pn || node.dn >= threshold.dn
    }
}

/// Result of evaluating a node from its children's table entries.
pub struct Selection {
    /// The most-proving child (the move to search next), as an index into
    /// the candidates.
    pub best: usize,
    /// Whether the tables knew nothing of `best`: it has not been
    /// searched.
    pub unvisited: bool,
    /// The parent's own numbers, aggregated from the children.
    pub node: PnDn,
    /// The numbers of `best`.
    pub best_child: PnDn,
    /// The numbers of the second-most-proving child.
    pub second_child: PnDn,
}

#[cfg(test)]
mod tests {
    use crate::board::Player::Black;
    use crate::board::{Board, Point};
    use crate::mate::budget::NodeBudget;
    use crate::mate::solver::Solver;
    use crate::mate::vct::{DFPNSVCTSolver, VCTState};

    /// A forbidden attack is kept by the move, not in the table. The
    /// table is keyed by the position, and the same position may be reached
    /// by another order of the moves, one that plays the point before it is
    /// forbidden: a disproof stored for it would be wrong there.
    #[test]
    fn test_a_forbidden_attack_leaves_nothing_in_the_table() -> Result<(), String> {
        // H8 is a double three (F8-G8-H8 and H6-H7-H8).
        let board = "F8,G8,H6,H7/A1,A3,A5,A7".parse::<Board>()?;
        let h8 = "H8".parse::<Point>()?;
        assert!(board.forbidden(h8).is_some());
        let mut solver = DFPNSVCTSolver::new(2);
        let state = &mut VCTState::from_board(&board, Black, 1, 1);
        assert!(solver.solve(state, &mut NodeBudget::unlimited()).is_none());
        // Every child was disproven, a legal one in the table.
        let i8 = "I8".parse::<Point>()?;
        let legal = solver.attacker_table.lookup_next(state, Some(i8));
        assert!(legal.is_some_and(|node| node.dn == 0));
        assert_eq!(solver.attacker_table.lookup_next(state, Some(h8)), None);
        // It is kept by the move instead.
        assert!(solver.forbidden_attacks.get(state.move_key(h8)).is_some());
        Ok(())
    }
}
