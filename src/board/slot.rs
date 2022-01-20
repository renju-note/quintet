use super::fundamentals::*;
use super::point::*;
use std::fmt;

const BLACK_FLAG: u8 = 0b01000000;
const WHITE_FLAG: u8 = 0b10000000;
const OVERLINE_FLAG: u8 = 0b00100000;
const EMPTY_SIGNATURE: u8 = 0b00000000;

pub type Signature = u8;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Slot {
    pub start: Index,
    pub signature: Signature,
}

impl Slot {
    pub fn new(start: Index, blacks_: u8, whites_: u8) -> Self {
        let blacks = (blacks_ & 0b0111110) >> 1;
        let whites = (whites_ & 0b0111110) >> 1;
        let margin = blacks_ & 0b1000001;
        let black = if blacks != 0b0 { BLACK_FLAG } else { 0b0 };
        let white = if whites != 0b0 { WHITE_FLAG } else { 0b0 };
        let overline = if margin != 0b0 { OVERLINE_FLAG } else { 0b0 };
        let stones = if blacks != 0b0 && whites != 0b0 {
            0b0
        } else if blacks != 0b0 {
            blacks
        } else if whites != 0b0 {
            whites
        } else {
            0b0
        };
        Slot {
            signature: black | white | overline | stones,
            start: start,
        }
    }

    pub fn potential(&self, player: Player) -> i8 {
        if self.signature == EMPTY_SIGNATURE {
            return 0;
        }
        let player_black = player.is_black();
        let black = self.contains_black();
        let white = self.contains_white();
        if black && white {
            return -1;
        }
        if black && !player_black || white && player_black {
            return -2;
        }
        if player_black && self.will_overline() {
            return -3;
        }
        self.nstones() as i8
    }

    pub fn eyes(&self) -> impl Iterator<Item = Index> + '_ {
        let stones = self.signature & 0b11111;
        (0..5)
            .filter(move |i| !stones & (0b1 << i) != 0b0)
            .map(move |i| self.start.walk(i))
            .flatten()
    }

    pub fn occupied_by(&self, player: Player) -> bool {
        if player.is_black() {
            self.contains_black() && !self.contains_white()
        } else {
            !self.contains_black() && self.contains_white()
        }
    }

    pub fn nstones(&self) -> u8 {
        (self.signature & 0b00011111).count_ones() as u8
    }

    pub fn will_overline(&self) -> bool {
        self.signature & OVERLINE_FLAG != 0b0
    }

    fn contains_black(&self) -> bool {
        self.signature & BLACK_FLAG != 0b0
    }

    fn contains_white(&self) -> bool {
        self.signature & WHITE_FLAG != 0b0
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Slot({:#010b})", self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::super::fundamentals::Player::*;
    use super::*;

    #[test]
    fn test_potential() {
        let start = Index::new(Direction::Vertical, 0, 0);
        let slot = Slot::new(start, 0b0000000, 0b0000000);
        assert_eq!(slot.potential(Black), 0);
        assert_eq!(slot.potential(White), 0);

        let slot = Slot::new(start, 0b0000100, 0b0000010);
        assert_eq!(slot.potential(Black), -1);
        assert_eq!(slot.potential(White), -1);

        let slot = Slot::new(start, 0b0010100, 0b0000000);
        assert_eq!(slot.potential(Black), 2);
        assert_eq!(slot.potential(White), -2);

        let slot = Slot::new(start, 0b0000000, 0b0110100);
        assert_eq!(slot.potential(Black), -2);
        assert_eq!(slot.potential(White), 3);

        let slot = Slot::new(start, 0b0000011, 0b0000000);
        assert_eq!(slot.potential(Black), -3);
        assert_eq!(slot.potential(White), -2);

        let slot = Slot::new(start, 0b0000001, 0b0000000);
        assert_eq!(slot.potential(Black), -3);
        assert_eq!(slot.potential(White), 0);
    }
}
