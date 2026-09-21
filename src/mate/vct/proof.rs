use super::state::VCTState;
use crate::board::Point;
use crate::mate::memo::Memo;
use crate::mate::state::{Key, State};
use std::fmt;

/// How many entries a table carries into a new search before it starts
/// dropping what older searches left behind. See [`Memo`].
pub const DEFAULT_CARRY_CAPACITY: usize = 1 << 16;

/// What a search has settled about a position, whatever limit it was asked
/// at. A mate within `limit` is a mate within more, and no mate within
/// `limit` is no mate within less, so one decision bounds a whole range.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Decided {
    /// Proven at this limit, hence at every limit at least this large.
    min_proven: Option<u8>,
    /// Disproven at this limit, hence at every limit at most this large.
    max_disproven: Option<u8>,
}

/// Transposition table of proof numbers, in two halves.
///
/// `estimates` holds proof numbers short of a decision, one entry per
/// [`Key`] — position *and* limit — because an unfinished number belongs to
/// the limit it was computed at. `decided` holds what is settled, keyed by
/// [`Key::position`] alone, so that a proof found at one limit answers at
/// another and a search can use what a search from a different root
/// established.
///
/// `transfer_from` is where that is allowed to start. `VCTSolver`'s
/// candidate generation asks its nested VCF solvers with
/// `min(state.limit, depth)` (`VCTState::vcf_state` / `threat_state`), so
/// below that depth the moves generated still change with the limit, and two
/// limits are then answering about different trees. From `transfer_from` up,
/// generation no longer moves and the bound holds.
pub struct ProofTable {
    estimates: Memo<Node>,
    decided: Memo<Decided>,
    transfer_from: u8,
}

impl ProofTable {
    pub fn with_carry_capacity(carry_capacity: usize, transfer_from: u8) -> Self {
        Self {
            estimates: Memo::new(carry_capacity),
            decided: Memo::new(carry_capacity),
            transfer_from,
        }
    }

    pub fn clear(&mut self) {
        self.estimates.clear();
        self.decided.clear();
    }

    /// See [`Memo::advance_generation`]: once per search, not per node.
    pub fn advance_generation(&mut self) {
        self.estimates.advance_generation();
        self.decided.advance_generation();
    }

    pub fn len(&self) -> usize {
        self.estimates.len() + self.decided.len()
    }

    pub fn insert(&mut self, state: &VCTState, node: Node) {
        self.record(state.key(), node);
    }

    fn record(&mut self, key: Key, node: Node) {
        self.estimates.insert(key.hash(), node);
        if key.limit < self.transfer_from {
            return;
        }
        // The bound is the limit the node was *asked* at. `node.limit` is an
        // aggregate over children and can be smaller than the depth the
        // decision actually needed, so it would claim too much.
        let mut decided = self.decided.get(key.position).copied().unwrap_or_default();
        if node.is_proven() {
            decided.min_proven = Some(decided.min_proven.unwrap_or(key.limit).min(key.limit));
        } else if node.dn == 0 {
            decided.max_disproven = Some(decided.max_disproven.unwrap_or(key.limit).max(key.limit));
        } else {
            return;
        }
        self.decided.insert(key.position, decided);
    }

    pub fn lookup_next(&self, state: &mut VCTState, next_move: Option<Point>) -> Option<Node> {
        let key = state.next_key(next_move);
        if let Some(&node) = self.estimates.get(key.hash()) {
            return Some(node);
        }
        self.decided_at(key)
    }

    fn decided_at(&self, key: Key) -> Option<Node> {
        if key.limit < self.transfer_from {
            return None;
        }
        let decided = self.decided.get(key.position)?;
        if let Some(proven) = decided.min_proven
            && proven <= key.limit
        {
            return Some(Node::proven(proven));
        }
        if let Some(disproven) = decided.max_disproven
            && key.limit <= disproven
        {
            return Some(Node::disproven(key.limit));
        }
        None
    }
}

pub const INF: u32 = u32::MAX;

/// Proof and disproof numbers of a position, plus the smallest `limit` at
/// which the numbers were established.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Node {
    pub pn: u32,
    pub dn: u32,
    pub limit: u8,
}

impl Node {
    pub fn new(pn: u32, dn: u32, limit: u8) -> Self {
        Self { pn, dn, limit }
    }

    /// A position the tables know nothing about.
    pub fn unknown() -> Self {
        Self::new(INF, INF, 0)
    }

    /// A threshold that is never exceeded.
    pub fn no_threshold() -> Self {
        Self::new(INF, INF, 0)
    }

    /// The attacker wins (pn = 0).
    pub fn proven(limit: u8) -> Self {
        Self::new(0, INF, limit)
    }

    /// The attacker cannot win within `limit` (dn = 0).
    pub fn disproven(limit: u8) -> Self {
        Self::new(INF, 0, limit)
    }

    /// Initial value of an unexpanded child of an AND node (a position where
    /// the attacker is to move). `approx_dn` is a heuristic disproof number.
    pub fn unexpanded_attack(approx_dn: u32, limit: u8) -> Self {
        Self::new(1, approx_dn, limit)
    }

    /// Initial value of an unexpanded child of an OR node (a position where
    /// the defender is to move). `approx_pn` is a heuristic proof number.
    pub fn unexpanded_defence(approx_pn: u32, limit: u8) -> Self {
        Self::new(approx_pn, 1, limit)
    }

    pub fn is_proven(&self) -> bool {
        self.pn == 0
    }

    /// Aggregation for an OR node: pn is the minimum, dn the sum over children.
    pub fn min_pn_sum_dn(&self, another: Self) -> Self {
        Self::new(
            self.pn.min(another.pn),
            self.dn.saturating_add(another.dn),
            self.limit.min(another.limit),
        )
    }

    /// Aggregation for an AND node: dn is the minimum, pn the sum over children.
    pub fn min_dn_sum_pn(&self, another: Self) -> Self {
        Self::new(
            self.pn.saturating_add(another.pn),
            self.dn.min(another.dn),
            self.limit.min(another.limit),
        )
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pn = if self.pn == INF {
            "INF".to_string()
        } else {
            self.pn.to_string()
        };
        let dn = if self.dn == INF {
            "INF".to_string()
        } else {
            self.dn.to_string()
        };
        write!(f, "(pn: {}, dn: {})", pn, dn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(transfer_from: u8) -> ProofTable {
        ProofTable::with_carry_capacity(64, transfer_from)
    }

    #[test]
    fn test_a_proof_answers_at_every_larger_limit() {
        let mut t = table(3);
        t.record(Key::new(0xabc, 5), Node::proven(5));
        assert_eq!(t.decided_at(Key::new(0xabc, 5)), Some(Node::proven(5)));
        assert_eq!(t.decided_at(Key::new(0xabc, 9)), Some(Node::proven(5)));
        // Nothing is claimed below the limit it was proven at.
        assert_eq!(t.decided_at(Key::new(0xabc, 4)), None);
        // A shallower proof tightens the bound.
        t.record(Key::new(0xabc, 4), Node::proven(4));
        assert_eq!(t.decided_at(Key::new(0xabc, 4)), Some(Node::proven(4)));
        assert_eq!(t.decided_at(Key::new(0xabc, 9)), Some(Node::proven(4)));
    }

    #[test]
    fn test_a_disproof_answers_at_every_smaller_limit() {
        let mut t = table(3);
        t.record(Key::new(0xabc, 4), Node::disproven(4));
        assert_eq!(t.decided_at(Key::new(0xabc, 4)), Some(Node::disproven(4)));
        assert_eq!(t.decided_at(Key::new(0xabc, 3)), Some(Node::disproven(3)));
        assert_eq!(t.decided_at(Key::new(0xabc, 5)), None);
        // A deeper disproof widens the bound.
        t.record(Key::new(0xabc, 6), Node::disproven(6));
        assert_eq!(t.decided_at(Key::new(0xabc, 5)), Some(Node::disproven(5)));
    }

    #[test]
    fn test_nothing_carries_below_transfer_from() {
        let mut t = table(3);
        // Recorded below the threshold, so not carried at all.
        t.record(Key::new(0xabc, 2), Node::proven(2));
        assert_eq!(t.decided_at(Key::new(0xabc, 2)), None);
        assert_eq!(t.decided_at(Key::new(0xabc, 8)), None);
        // Recorded above it, but not read below it.
        t.record(Key::new(0xdef, 4), Node::disproven(4));
        assert_eq!(t.decided_at(Key::new(0xdef, 2)), None);
        assert_eq!(t.decided_at(Key::new(0xdef, 4)), Some(Node::disproven(4)));
    }

    #[test]
    fn test_undecided_numbers_are_not_carried() {
        let mut t = table(3);
        t.record(Key::new(0xabc, 5), Node::new(3, 7, 5));
        assert_eq!(t.decided_at(Key::new(0xabc, 5)), None);
        assert_eq!(t.decided_at(Key::new(0xabc, 9)), None);
    }
}
