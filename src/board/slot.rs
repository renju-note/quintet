use super::fundamentals::Player::*;
use super::fundamentals::*;
use std::fmt;

pub const WINDOW_SIZE: u8 = 7;
const WINDOW_MASK: u16 = (0b1 << 7) - 1;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Slot(pub u8);

impl Slot {
    pub fn new(blacks_: u8, whites_: u8) -> Self {
        let blacks = (blacks_ & 0b0111110) >> 1;
        let whites = (whites_ & 0b0111110) >> 1;
        let margin = blacks_ & 0b1000001;

        let black_flag = if blacks != 0b0 { BLACK_FLAG } else { 0b0 };
        let white_flag = if whites != 0b0 { WHITE_FLAG } else { 0b0 };
        let overline_flag = if margin != 0b0 { OVERLINE_FLAG } else { 0b0 };
        let stones = if blacks != 0b0 && whites != 0b0 {
            0b0
        } else if blacks != 0b0 {
            blacks
        } else if whites != 0b0 {
            whites
        } else {
            0b0
        };

        Self(black_flag | white_flag | overline_flag | stones)
    }

    pub fn potential(&self, player: Player) -> u8 {
        if self.is_free() {
            return 1;
        }
        if self.contains(player.opponent()) {
            return 0;
        }
        if player.is_black() && self.will_overline() {
            return 0;
        }
        self.nstones() + 1
    }

    pub fn occupied_by(&self, player: Player) -> bool {
        if player.is_black() {
            self.contains(Black) && !self.contains(White)
        } else {
            self.contains(White) && !self.contains(Black)
        }
    }

    pub fn contains(&self, player: Player) -> bool {
        if player.is_black() {
            self.0 & BLACK_FLAG != 0b0
        } else {
            self.0 & WHITE_FLAG != 0b0
        }
    }

    pub fn nstones(&self) -> u8 {
        COUNT_ONES[(self.0 & 0b00011111) as usize]
    }

    pub fn nstones_head(&self) -> u8 {
        COUNT_ONES[(self.0 & 0b00001111) as usize]
    }

    pub fn eyes(&self) -> &'static [u8] {
        EYES[(self.0 & 0b00011111) as usize]
    }

    pub fn eyes_head(&self) -> &'static [u8] {
        EYES[(self.0 & 0b00001111) as usize]
    }

    pub fn will_overline(&self) -> bool {
        self.0 & OVERLINE_FLAG != 0b0
    }

    pub fn is_free(&self) -> bool {
        self.0 == EMPTY_SIGNATURE
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Slot({:#010b})", self.0)
    }
}

pub struct Slots {
    blacks: Bits,
    whites: Bits,
    limit: u8,
    i: u8,
}

impl Slots {
    pub fn new(blacks: Bits, whites: Bits, start: u8, end: u8) -> Self {
        Self {
            blacks: blacks,
            whites: whites,
            limit: end - (WINDOW_SIZE - 1),
            i: start,
        }
    }
}

impl Iterator for Slots {
    type Item = (u8, Slot);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i > self.limit {
            return None;
        }
        self.i += 1;
        let blacks_ = (self.blacks >> i & WINDOW_MASK) as u8;
        let whites_ = (self.whites >> i & WINDOW_MASK) as u8;
        Some((i, Slot::new(blacks_, whites_)))
    }
}

const BLACK_FLAG: u8 = 0b01000000;
const WHITE_FLAG: u8 = 0b10000000;
const OVERLINE_FLAG: u8 = 0b00100000;
const EMPTY_SIGNATURE: u8 = 0b00000000;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(Slot::new(0b0000000, 0b0000000), Slot(0b00000000));
        assert_eq!(Slot::new(0b0000100, 0b0000010), Slot(0b11000000));
        assert_eq!(Slot::new(0b0010100, 0b0000000), Slot(0b01001010));
        assert_eq!(Slot::new(0b0000000, 0b0110100), Slot(0b10011010));
        assert_eq!(Slot::new(0b0000011, 0b0000000), Slot(0b01100001));
        assert_eq!(Slot::new(0b1000000, 0b0000000), Slot(0b00100000));
        assert_eq!(Slot::new(0b0000001, 0b0001100), Slot(0b10100110));
    }

    #[test]
    fn test_potential() {
        let slot = Slot::new(0b0000000, 0b0000000);
        assert_eq!(slot.potential(Black), 1);
        assert_eq!(slot.potential(White), 1);

        let slot = Slot::new(0b0000100, 0b0000010);
        assert_eq!(slot.potential(Black), 0);
        assert_eq!(slot.potential(White), 0);

        let slot = Slot::new(0b0010100, 0b0000000);
        assert_eq!(slot.potential(Black), 3);
        assert_eq!(slot.potential(White), 0);

        let slot = Slot::new(0b0000000, 0b0110100);
        assert_eq!(slot.potential(Black), 0);
        assert_eq!(slot.potential(White), 4);

        let slot = Slot::new(0b0000011, 0b0000000);
        assert_eq!(slot.potential(Black), 0);
        assert_eq!(slot.potential(White), 0);

        let slot = Slot::new(0b0000001, 0b0000000);
        assert_eq!(slot.potential(Black), 0);
        assert_eq!(slot.potential(White), 1);
    }
}
