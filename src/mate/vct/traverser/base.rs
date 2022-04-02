use crate::board::*;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::State;

pub struct Selection {
    pub best: Option<Point>,
    pub current: Node,
    pub next1: Node,
    pub next2: Node,
}

pub trait Traverser {
    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection;

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection;

    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node;

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node;

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node);

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node);

    fn loop_attacks(&mut self, state: &mut State, attacks: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_attack(state, &attacks);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.next_threshold_attack(&selection, threshold);
            self.expand_attack(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn loop_defences(&mut self, state: &mut State, defences: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_defence(state, &defences);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.next_threshold_defence(&selection, threshold);
            self.expand_defence(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn backoff(&self, current: Node, threshold: Node) -> bool {
        current.pn >= threshold.pn || current.dn >= threshold.dn
    }
}
