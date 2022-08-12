use super::Selection;
use crate::mate::vct_lazy::proof::*;

/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/

pub trait DFPNSTraverser {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = threshold.pn.min(selection.next2.pn.saturating_add(1));
        let dn = (threshold.dn - selection.current.dn).saturating_add(selection.next1.dn);
        Node::new(pn, dn, selection.next1.limit)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = (threshold.pn - selection.current.pn).saturating_add(selection.next1.pn);
        let dn = threshold.dn.min(selection.next2.dn.saturating_add(1));
        Node::new(pn, dn, selection.next1.limit)
    }
}
