use crate::board::Point;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;

pub struct Selection {
    pub best: Option<Point>,
    pub current: Node,
    pub next1: Node,
    pub next2: Node,
}

pub trait Selector: ProofTree {
    fn select_attack(&mut self, state: &mut VCTState, attacks: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(attacks[0]);
        let mut current = Node::zero_dn(limit);
        let mut next1 = Node::zero_dn(limit);
        let mut next2 = Node::zero_dn(limit);
        let init = Node::unit_dn(attacks.len() as u32, limit); // trick
        for &attack in attacks {
            let maybe_child = self.attacker_table().lookup_next(state, Some(attack));
            let child = maybe_child.unwrap_or(init);
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

    fn select_defence(&mut self, state: &mut VCTState, defences: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(defences[0]);
        let mut current = Node::zero_pn(limit - 1);
        let mut next1 = Node::zero_pn(limit - 1);
        let mut next2 = Node::zero_pn(limit - 1);
        let init = Node::unit_pn(defences.len() as u32, limit - 1); // trick
        for &defence in defences {
            let maybe_child = self.defender_table().lookup_next(state, Some(defence));
            let child = maybe_child.unwrap_or(init);
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
}
