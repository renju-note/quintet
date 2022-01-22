use super::fundamentals::*;
use super::util;
use std::cmp;
use std::fmt;

pub const SLOT_SIZE: u8 = 5;
pub const WINDOW_SIZE: u8 = SLOT_SIZE + 2;
const WINDOW_MASK: u16 = (0b1 << WINDOW_SIZE) - 1;
const BLACK_FLAG: u8 = 0b01000000;
const WHITE_FLAG: u8 = 0b10000000;
const OVERLINE_FLAG: u8 = 0b00100000;
const EMPTY_SIGNATURE: u8 = 0b00000000;

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
        if player.is_black() && self.will_overline() {
            return 0;
        }
        if self.contains(player.opponent()) {
            return 0;
        }
        if self.is_free() {
            return 1;
        }
        self.nstones() + 1
    }

    pub fn occupied_by(&self, player: Player) -> bool {
        self.contains(player) && !self.contains(player.opponent())
    }

    pub fn nstones(&self) -> u8 {
        util::count_ones(self.0 & 0b00011111)
    }

    pub fn nstones_head(&self) -> u8 {
        util::count_ones(self.0 & 0b00001111)
    }

    pub fn eyes(&self) -> &'static [u8] {
        util::eyes(self.0 & 0b00011111)
    }

    pub fn eyes_head(&self) -> &'static [u8] {
        util::eyes(self.0 & 0b00001111)
    }

    pub fn contains(&self, player: Player) -> bool {
        if player.is_black() {
            self.0 & BLACK_FLAG != 0b0
        } else {
            self.0 & WHITE_FLAG != 0b0
        }
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
    blacks: u16,
    whites: u16,
    limit: u8,
    i: u8,
}

impl Slots {
    pub fn new(size: u8, blacks: u16, whites: u16) -> Self {
        Self {
            blacks: blacks,
            whites: whites,
            limit: size - (WINDOW_SIZE - 1),
            i: 0,
        }
    }

    pub fn new_on(i: u8, size: u8, blacks: u16, whites: u16) -> Self {
        Self {
            blacks: blacks,
            whites: whites,
            limit: cmp::min(size - (WINDOW_SIZE - 1), i),
            i: cmp::max(0, i as i8 - (WINDOW_SIZE as i8 - 1)) as u8,
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
