pub const VICTORY: u8 = 5;

const TARGET_MASK: u8 = 0b00111110;
const MARGIN_MASK: u8 = 0b01000001;
const FIRST_MASK: u8 = 0b00000010;

pub struct Potentials {
    my: u16,
    op: u16,
    min: u8,
    strict: bool,
    limit: u8,
    i: u8,
    acc: [u8; 5],
}

impl Potentials {
    pub fn new(size: u8, my: u16, op: u16, min: u8, strict: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            min: min,
            strict: strict,
            limit: size,
            i: 0,
            acc: <[u8; 5]>::default(),
        }
    }
}

impl Iterator for Potentials {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.limit {
            return None;
        }
        let i = self.i;
        self.i += 1;

        let my_ = (self.my >> i) as u8;
        let op_ = (self.op >> i) as u8;
        let is_fullsize = i <= self.limit - VICTORY;
        let is_valid = op_ & TARGET_MASK == 0b0 && (!self.strict || my_ & MARGIN_MASK == 0b0);
        let blank = my_ & FIRST_MASK == 0b0 && op_ & FIRST_MASK == 0b0;

        self.acc[0] = self.acc[1];
        self.acc[1] = self.acc[2];
        self.acc[2] = self.acc[3];
        self.acc[3] = self.acc[4];
        self.acc[4] = if is_fullsize && is_valid {
            let p = (my_ & TARGET_MASK).count_ones() as u8 + 1;
            if p >= self.min {
                p
            } else {
                0
            }
        } else {
            0
        };

        let mut max = 0;
        let mut count = 0;
        for i in 0..5 {
            let p = self.acc[i];
            if p > max {
                max = p;
                count = 1;
            } else if p == max {
                count += 1
            }
        }

        let ret = max * count;
        if blank && ret >= self.min {
            Some((i, ret))
        } else {
            self.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potentials() {
        let my = 0b011100010010100;
        let op = 0b000000001000000;

        let result = Potentials::new(15, my, op, 3, false).collect::<Vec<_>>();
        let expected = [
            (0, 3),
            (1, 6),
            (3, 6),
            (5, 3),
            (8, 6),
            (9, 4),
            (10, 8),
            (14, 4),
        ];
        assert_eq!(result, expected);

        let result = Potentials::new(15, my, op, 3, true).collect::<Vec<_>>();
        let expected = [(0, 3), (1, 6), (3, 6), (5, 3), (9, 4), (10, 8), (14, 4)];
        assert_eq!(result, expected);

        let my = 0b000001100000000;
        let op = 0b000000000000000;

        let result = Potentials::new(15, my, op, 3, false).collect::<Vec<_>>();
        let expected = [(5, 3), (6, 6), (7, 9), (10, 9), (11, 6), (12, 3)];
        assert_eq!(result, expected);

        let result = Potentials::new(15, my, op, 3, true).collect::<Vec<_>>();
        let expected = [(5, 3), (6, 6), (7, 9), (10, 9), (11, 6), (12, 3)];
        assert_eq!(result, expected);
    }
}
