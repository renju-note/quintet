mod dfpns;
mod dfs;
mod pns;

pub use dfpns::DFPNSTraverser;
pub use dfs::DFSTraverser;
pub use pns::PNSTraverser;

use super::selector::*;
use crate::board::Point;
use crate::mate::state::State;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;

pub trait Traverser: Selector {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node;
    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node;

    fn traverse_attacks<F>(
        &mut self,
        state: &mut VCTState,
        attacks: &[Point],
        threshold: Node,
        search_defences: F,
    ) -> Selection
    where
        F: Fn(&mut Self, &mut VCTState, Node) -> Node,
    {
        loop {
            let selection = self.select_attack(state, &attacks);
            if self.backoff(selection.current, threshold) {
                return selection;
            }
            let next_threshold = self.next_threshold_attack(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = search_defences(self, child, next_threshold);
                self.attacker_table().insert(child, result);
            });
        }
    }

    fn traverse_defences<F>(
        &mut self,
        state: &mut VCTState,
        defences: &[Point],
        threshold: Node,
        search_defences: F,
    ) -> Selection
    where
        F: Fn(&mut Self, &mut VCTState, Node) -> Node,
    {
        loop {
            let selection = self.select_defence(state, &defences);
            if self.backoff(selection.current, threshold) {
                return selection;
            }
            let next_threshold = self.next_threshold_defence(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = search_defences(self, child, next_threshold);
                self.defender_table().insert(child, result);
            });
        }
    }

    fn backoff(&self, current: Node, threshold: Node) -> bool {
        current.pn >= threshold.pn || current.dn >= threshold.dn
    }
}
