use super::fundamentals::*;
use super::point::*;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Slot {
    start: Index,
    signature: u8,
}

impl Slot {
    pub fn new(start: Index, blacks_: u8, whites_: u8) -> Self {
        let blacks = (blacks_ & 0b0111110) >> 1;
        let whites = (whites_ & 0b0111110) >> 1;
        let margin = blacks_ & 0b1000001;
        let black = if blacks != 0b0 { 0b01000000 } else { 0b0 };
        let white = if whites != 0b0 { 0b10000000 } else { 0b0 };
        let overline = if margin != 0b0 { 0b00100000 } else { 0b0 };
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
        if self.signature == 0b00000000 {
            return 0;
        }
        let player_black = player.is_black();
        let black = self.black();
        let white = self.white();
        if black && white {
            return -1;
        }
        if black && !player_black || white && player_black {
            return -2;
        }
        if player_black && self.overline() {
            return -3;
        }
        self.count_stones()
    }

    pub fn eyes(&self) -> impl Iterator<Item = Index> + '_ {
        let stones = self.signature & 0b11111;
        (0..5)
            .filter(move |i| !stones & (0b1 << i) != 0b0)
            .map(move |i| self.start.walk(i))
            .flatten()
    }

    fn black(&self) -> bool {
        self.signature & 0b01000000 != 0b0
    }

    fn white(&self) -> bool {
        self.signature & 0b10000000 != 0b0
    }

    fn overline(&self) -> bool {
        self.signature & 0b00100000 != 0b0
    }

    fn count_stones(&self) -> i8 {
        (self.signature & 0b00011111).count_ones() as i8
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
