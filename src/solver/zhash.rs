use super::super::board::*;
use rand::prelude::*;

const Z_HASH_TABLE_SIZE: usize = 2 * (BOARD_SIZE as usize) * (BOARD_SIZE as usize);

pub struct ZobristHashTable {
    pub table: [u64; Z_HASH_TABLE_SIZE],
}

impl ZobristHashTable {
    pub fn new() -> ZobristHashTable {
        let mut rng = rand::thread_rng();
        let mut table: [u64; Z_HASH_TABLE_SIZE] = [0; Z_HASH_TABLE_SIZE];
        for i in 0..Z_HASH_TABLE_SIZE {
            table[i] = rng.gen();
        }
        ZobristHashTable { table: table }
    }

    pub fn get(&self, black: bool, p: Point) -> u64 {
        let i = 2 * ((p.x * BOARD_SIZE + p.y) as usize) + if black { 0 } else { 1 };
        self.table[i]
    }
}
