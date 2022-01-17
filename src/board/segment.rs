use super::fundamentals::*;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Segment(u8);

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Segment({:#010b})", self.0)
    }
}

pub fn scan(blacks: Bits, whites: Bits, limit: u8) -> Vec<Segment> {
    let mut result = vec![];
    let blanks_ = (!(blacks | whites) & ((0b1 << limit as u16) - 1)) << 1;
    let blacks_ = blacks << 1;
    let whites_ = whites << 1;
    for i in 0..=(limit + 2 - 7) {
        let blanks = blanks_ >> i & 0b0111110;
        let bpadds = blacks_ >> i & 0b1000001;
        let blacks = blacks_ >> i & 0b0111110;
        let whites = whites_ >> i & 0b0111110;
        if blanks == 0b0111110 {
            if bpadds == 0b0000000 {
                result.push(Segment(0b00000000));
            } else {
                result.push(Segment(0b10000000))
            }
        } else if blacks != 0b0 && whites == 0b0 && bpadds == 0b0 {
            result.push(Segment(0b00000000 | encode(blacks >> 1)));
        } else if whites != 0b0 && blacks == 0b0 {
            result.push(Segment(0b10000000 | encode(whites >> 1)));
        } else {
            result.push(Segment(0b00001111))
        }
    }
    result
}

fn encode(stones: Bits) -> u8 {
    let count = stones.count_ones();
    let shape = match count {
        1 => stones.trailing_zeros(),
        2 => encode_shape(stones),
        3 => encode_shape(!stones & 0b11111),
        4 => (!stones & 0b11111).trailing_zeros(),
        _ => 0b0000,
    };
    ((count << 4) | shape) as u8
}

fn encode_shape(stones: Bits) -> u32 {
    let right = stones.trailing_zeros();
    let stones = stones & !(0b1 << right);
    let left = (stones >> 1).trailing_zeros();
    (left << 2) | right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        let result = scan(0b000000001100000, 0b000000000000000, 15);
        let expected = [
            Segment(0b10000000),
            Segment(0b00001111),
            Segment(0b00101111),
            Segment(0b00101010),
            Segment(0b00100101),
            Segment(0b00100000),
            Segment(0b00001111),
            Segment(0b10000000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
        ];
        assert_eq!(result, expected);

        let result = scan(0b000000000000000, 0b000000001100000, 15);
        let expected = [
            Segment(0b00000000),
            Segment(0b10010100),
            Segment(0b10101111),
            Segment(0b10101010),
            Segment(0b10100101),
            Segment(0b10100000),
            Segment(0b10010000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
        ];
        assert_eq!(result, expected);
    }
}
