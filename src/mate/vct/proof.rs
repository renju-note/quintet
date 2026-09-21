use super::state::VCTState;
use crate::board::Point;
use crate::mate::memo::Memo;
use crate::mate::state::State;
use std::fmt;

/// How many entries a table carries into a new search before it starts
/// dropping what older searches left behind. See [`Memo`].
pub const DEFAULT_CARRY_CAPACITY: usize = 1 << 16;

/// Transposition table of proof numbers, keyed by
/// [`State::zobrist_hash`] — the position, the turn, the remaining `limit`
/// and the attacker.
pub struct ProofTable {
    table: Memo<Node>,
}

impl ProofTable {
    pub fn with_carry_capacity(carry_capacity: usize) -> Self {
        Self {
            table: Memo::new(carry_capacity),
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// See [`Memo::advance_generation`]: once per search, not per node.
    pub fn advance_generation(&mut self) {
        self.table.advance_generation();
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn insert(&mut self, state: &VCTState, node: Node) {
        let key = state.zobrist_hash();
        self.table.insert(key, node);
    }

    pub fn lookup_next(&self, state: &mut VCTState, next_move: Option<Point>) -> Option<Node> {
        let key = state.next_zobrist_hash(next_move);
        self.table.get(key).copied()
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
