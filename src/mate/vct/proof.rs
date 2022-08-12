use super::state::State;
use crate::board::Point;
use std::collections::HashMap;
use std::fmt;

pub trait ProofTree {
    fn attacker_table(&mut self) -> &mut Table;
    fn defender_table(&mut self) -> &mut Table;
}

pub struct Table {
    table: HashMap<u64, Node>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, state: &State, node: Node) {
        let key = state.zobrist_hash();
        self.table.insert(key, node.clone());
    }

    pub fn lookup_next(&self, state: &mut State, next_move: Option<Point>) -> Option<Node> {
        let key = state.next_zobrist_hash(next_move);
        self.table.get(&key).map(|&c| c)
    }
}

pub const INF: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Node {
    pub pn: u32,
    pub dn: u32,
    pub limit: u8,
}

impl Node {
    pub fn new(pn: u32, dn: u32, limit: u8) -> Self {
        Self {
            pn: pn,
            dn: dn,
            limit: limit,
        }
    }

    pub fn inf() -> Self {
        Self::new(INF, INF, 0)
    }

    pub fn zero_pn(limit: u8) -> Self {
        Self::new(0, INF, limit)
    }

    pub fn zero_dn(limit: u8) -> Self {
        Self::new(INF, 0, limit)
    }

    pub fn init_pn(approx_dn: u32, limit: u8) -> Self {
        Self::new(1, approx_dn, limit)
    }

    pub fn init_dn(approx_pn: u32, limit: u8) -> Self {
        Self::new(approx_pn, 1, limit)
    }

    pub fn proven(&self) -> bool {
        self.pn == 0
    }

    pub fn min_pn_sum_dn(&self, another: Self) -> Self {
        Self::new(
            self.pn.min(another.pn),
            self.dn.saturating_add(another.dn),
            self.limit.min(another.limit),
        )
    }

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
