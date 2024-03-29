use crate::board::Point;
use crate::mate::state::State;
use crate::mate::vct_lazy::proof::*;
use crate::mate::vct_lazy::state::LazyVCTState;

pub struct Selection {
    pub best: Option<Point>,
    pub current: Node,
    pub next1: Node,
    pub next2: Node,
}

pub trait Traverser: ProofTree {
    fn traverse_attacks<F>(
        &mut self,
        state: &mut LazyVCTState,
        attacks: &[(Point, Node)],
        threshold: Node,
        search_defences: F,
    ) -> Selection
    where
        F: Fn(&mut Self, &mut LazyVCTState, Node) -> Node,
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
        state: &mut LazyVCTState,
        defences: &[(Point, Node)],
        threshold: Node,
        search_defences: F,
    ) -> Selection
    where
        F: Fn(&mut Self, &mut LazyVCTState, Node) -> Node,
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

    fn select_attack(&mut self, state: &mut LazyVCTState, attacks: &[(Point, Node)]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(attacks[0].0);
        let mut current = Node::zero_dn(limit);
        let mut next1 = Node::zero_dn(limit);
        let mut next2 = Node::zero_dn(limit);
        for &(attack, init) in attacks {
            let child = self
                .attacker_table()
                .lookup_next(state, Some(attack))
                .unwrap_or(init);
            current = current.min_pn_sum_dn(child);
            if child.pn < next1.pn {
                best.replace(attack);
                next2 = next1;
                next1 = child;
            } else if child.pn < next2.pn {
                next2 = child;
            }
            if current.pn == 0 {
                current.dn = INF;
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
        }
    }

    fn select_defence(
        &mut self,
        state: &mut LazyVCTState,
        defences: &[(Point, Node)],
    ) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(defences[0].0);
        let mut current = Node::zero_pn(limit - 1);
        let mut next1 = Node::zero_pn(limit - 1);
        let mut next2 = Node::zero_pn(limit - 1);
        for &(defence, init) in defences {
            let child = self
                .defender_table()
                .lookup_next(state, Some(defence))
                .unwrap_or(init);
            current = current.min_dn_sum_pn(child);
            if child.dn < next1.dn {
                best.replace(defence);
                next2 = next1;
                next1 = child;
            } else if child.dn < next2.dn {
                next2 = child;
            }
            if current.dn == 0 {
                current.pn = INF;
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
        }
    }

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
