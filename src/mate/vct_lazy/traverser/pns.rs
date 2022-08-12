use super::Selection;
use crate::mate::vct_lazy::proof::*;

pub trait PNSTraverser {
    fn next_threshold_attack(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.saturating_add(1),
            next.dn.saturating_add(1),
            next.limit,
        )
    }

    fn next_threshold_defence(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.saturating_add(1),
            next.dn.saturating_add(1),
            next.limit,
        )
    }
}
