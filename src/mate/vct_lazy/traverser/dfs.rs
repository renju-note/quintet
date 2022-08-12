use super::Selection;
use crate::mate::vct_lazy::proof::*;

pub trait DFSTraverser {
    fn next_threshold_attack(&self, _selection: &Selection, _threshold: Node) -> Node {
        Node::inf()
    }

    fn next_threshold_defence(&self, _selection: &Selection, _threshold: Node) -> Node {
        Node::inf()
    }
}
