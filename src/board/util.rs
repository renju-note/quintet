pub fn count_ones(bits: u8) -> u8 {
    COUNT_ONES[bits as usize]
}

pub fn eyes(bits: u8) -> &'static [u8] {
    EYES[bits as usize]
}

const COUNT_ONES: [u8; 32] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
];

const EYES: [&[u8]; 32] = [
    &[0, 1, 2, 3, 4],
    &[1, 2, 3, 4],
    &[0, 2, 3, 4],
    &[2, 3, 4],
    &[0, 1, 3, 4],
    &[1, 3, 4],
    &[0, 3, 4],
    &[3, 4],
    &[0, 1, 2, 4],
    &[1, 2, 4],
    &[0, 2, 4],
    &[2, 4],
    &[0, 1, 4],
    &[1, 4],
    &[0, 4],
    &[4],
    &[0, 1, 2, 3],
    &[1, 2, 3],
    &[0, 2, 3],
    &[2, 3],
    &[0, 1, 3],
    &[1, 3],
    &[0, 3],
    &[3],
    &[0, 1, 2],
    &[1, 2],
    &[0, 2],
    &[2],
    &[0, 1],
    &[1],
    &[0],
    &[],
];
