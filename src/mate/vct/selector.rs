use super::solver::VCTSolver;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;

/// Result of evaluating a node from its children's table entries.
pub struct Selection {
    /// The most-proving child (the move to search next).
    pub best: Option<Point>,
    /// The parent's own numbers, aggregated from the children.
    pub node: Node,
    /// The numbers of `best`.
    pub best_child: Node,
    /// The numbers of the second-most-proving child.
    pub second_child: Node,
}

/// Choosing the most-proving child. Expands nothing; only reads the tables.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn select_attack(&self, state: &mut VCTState, attacks: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(attacks[0]);
        let mut node = Node::disproven(limit);
        let mut best_child = Node::disproven(limit);
        let mut second_child = Node::disproven(limit);
        let init = Node::unexpanded_defence(attacks.len() as u32, limit); // trick
        for &attack in attacks {
            let maybe_child = self.attacker_table.lookup_next(state, Some(attack));
            let child = maybe_child.unwrap_or(init);
            node = node.min_pn_sum_dn(child);
            if child.pn < best_child.pn {
                best.replace(attack);
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
            node,
            best_child,
            second_child,
        }
    }

    pub fn select_defence(&self, state: &mut VCTState, defences: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(defences[0]);
        let mut node = Node::proven(limit - 1);
        let mut best_child = Node::proven(limit - 1);
        let mut second_child = Node::proven(limit - 1);
        let init = Node::unexpanded_attack(defences.len() as u32, limit - 1); // trick
        for &defence in defences {
            let maybe_child = self.defender_table.lookup_next(state, Some(defence));
            let child = maybe_child.unwrap_or(init);
            node = node.min_dn_sum_pn(child);
            if child.dn < best_child.dn {
                best.replace(defence);
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
            node,
            best_child,
            second_child,
        }
    }
}
