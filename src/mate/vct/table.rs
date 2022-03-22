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
        let key = state.zobrist_hash();
        self.table.get(&key).map_or(Node::init(state.limit), |c| *c)
    }

    pub fn lookup_child(&self, state: &mut State, m: Point) -> Node {
        let last2_move = state.game().last2_move();
        state.play(m);
        let result = self.lookup(state);
        state.undo(last2_move);
        result
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
