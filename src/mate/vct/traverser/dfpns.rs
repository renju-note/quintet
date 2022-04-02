use super::Selection;
use crate::mate::vct::proof::*;

/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/

pub trait DFPNSTraverser {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = threshold
            .pn
            .min(selection.next2.pn.checked_add(1).unwrap_or(INF));
        let dn = (threshold.dn - selection.current.dn)
            .checked_add(selection.next1.dn)
            .unwrap_or(INF);
        Node::new(pn, dn, selection.next1.limit)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = (threshold.pn - selection.current.pn)
            .checked_add(selection.next1.pn)
            .unwrap_or(INF);
        let dn = threshold
            .dn
            .min(selection.next2.dn.checked_add(1).unwrap_or(INF));
        Node::new(pn, dn, selection.next1.limit)
    }
}
