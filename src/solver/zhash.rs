use super::super::bitboard::*;

pub struct ZobristTable {
    pub table: [u64; TABLE_SIZE],
}

impl ZobristTable {
    pub fn new() -> ZobristTable {
        let mut lcg_state: u128 = 1;
        let mut table: [u64; TABLE_SIZE] = [0; TABLE_SIZE];
        for i in 0..TABLE_SIZE {
            lcg_state = lcg(lcg_state);
            table[i] = lcg_state as u64;
        }
        ZobristTable { table: table }
    }

    pub fn apply(&self, current: u64, black: bool, p: Point) -> u64 {
        let i = 2 * ((p.x * BOARD_SIZE + p.y) as usize) + if black { 0 } else { 1 };
        let zhash = self.table[i];
        current ^ zhash
    }
}

const TABLE_SIZE: usize = 2 * (BOARD_SIZE as usize) * (BOARD_SIZE as usize); // 450
const LCG_MULTIPLIER: u128 = 0x2d99787926d46932a4c1f32680f70c55;
const LCG_INCREMENT: u128 = 0x1;

// https://www.pcg-random.org/posts/does-it-beat-the-minimal-standard.html
fn lcg(state: u128) -> u128 {
    (state * LCG_MULTIPLIER + LCG_INCREMENT) >> 64
}
