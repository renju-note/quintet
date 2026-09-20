use super::selector::Selection;
use crate::mate::vct::proof::*;

/// How far a child may be searched before control returns to its parent.
///
/// This is the only thing that differs between the DFS, PNS and df-pn VCT
/// solvers. `expand_attacks` / `expand_defences` in `searcher.rs` call these
/// with the parent's current threshold and the result of selecting the
/// most-proving child.
pub trait ThresholdPolicy {
    fn next_threshold_attack(selection: &Selection, threshold: Node) -> Node;
    fn next_threshold_defence(selection: &Selection, threshold: Node) -> Node;
}

/// No threshold: the chosen child is searched to completion before the parent
/// looks at the next one. Proof numbers are used only for move ordering.
pub struct DFSThreshold;

impl ThresholdPolicy for DFSThreshold {
    fn next_threshold_attack(_selection: &Selection, _threshold: Node) -> Node {
        Node::no_threshold()
    }

    fn next_threshold_defence(_selection: &Selection, _threshold: Node) -> Node {
        Node::no_threshold()
    }
}

/// The child returns as soon as its numbers change, so the most-proving child
/// is re-selected at every level. Emulates best-first PNS inside a recursive
/// search.
pub struct PNSThreshold;

impl ThresholdPolicy for PNSThreshold {
    fn next_threshold_attack(selection: &Selection, _threshold: Node) -> Node {
        let best = selection.best_child;
        Node::new(
            best.pn.saturating_add(1),
            best.dn.saturating_add(1),
            best.limit,
        )
    }

    fn next_threshold_defence(selection: &Selection, _threshold: Node) -> Node {
        let best = selection.best_child;
        Node::new(
            best.pn.saturating_add(1),
            best.dn.saturating_add(1),
            best.limit,
        )
    }
}

/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/

/// Df-pn thresholds: stay in the best child as long as it remains the best
/// (bounded by the second-best child), and never exceed the parent's budget.
pub struct DFPNSThreshold;

impl ThresholdPolicy for DFPNSThreshold {
    fn next_threshold_attack(selection: &Selection, threshold: Node) -> Node {
        let pn = threshold
            .pn
            .min(selection.second_child.pn.saturating_add(1));
        let dn = (threshold.dn - selection.node.dn).saturating_add(selection.best_child.dn);
        Node::new(pn, dn, selection.best_child.limit)
    }

    fn next_threshold_defence(selection: &Selection, threshold: Node) -> Node {
        let pn = (threshold.pn - selection.node.pn).saturating_add(selection.best_child.pn);
        let dn = threshold
            .dn
            .min(selection.second_child.dn.saturating_add(1));
        Node::new(pn, dn, selection.best_child.limit)
    }
}
