use super::state::*;
use crate::board::*;
use std::collections::HashMap;

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

    pub fn lookup(&self, state: &State) -> Node {
        self.lookup_hash_limit(state.zobrist_hash(), state.limit)
    }

    pub fn lookup_next(&self, state: &mut State, next_move: Point) -> Node {
        // Extract game in order not to cause updating state.field (which costs high)
        let (next_zobrist_hash, next_limit) = state.next_zobrist_hash_limit(next_move);
        self.lookup_hash_limit(next_zobrist_hash, next_limit)
    }

    fn lookup_hash_limit(&self, hash: u64, limit: u8) -> Node {
        self.table.get(&hash).map_or(Node::init(limit), |c| *c)
    }
}

use std::fmt;

pub const INF: usize = usize::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Node {
    pub pn: usize,
    pub dn: usize,
    pub limit: u8,
}

impl Node {
    pub fn new(pn: usize, dn: usize, limit: u8) -> Self {
        Self {
            pn: pn,
            dn: dn,
            limit: limit,
        }
    }

    pub fn init(limit: u8) -> Self {
        Self::new(1, 1, limit)
    }

    pub fn root(limit: u8) -> Self {
        Self::new(INF - 1, INF - 1, limit)
    }

    pub fn inf_pn(limit: u8) -> Self {
        Self::new(INF, 0, limit)
    }

    pub fn inf_dn(limit: u8) -> Self {
        Self::new(0, INF, limit)
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
