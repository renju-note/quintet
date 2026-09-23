pub const VICTORY: u8 = 5;

const TARGET_MASK: u8 = 0b00111110;
const MARGIN_MASK: u8 = 0b01000001;
const FIRST_MASK: u8 = 0b00000010;

pub struct Potentials {
    my: u16,
    op: u16,
    min: u8,
    exact: bool,
    limit: u8,
    i: u8,
    acc: [u8; 5],
}

impl Potentials {
    pub fn new(size: u8, my: u16, op: u16, min: u8, exact: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            min,
            exact,
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
        let is_valid = op_ & TARGET_MASK == 0b0 && (!self.exact || my_ & MARGIN_MASK == 0b0);
        let blank = my_ & FIRST_MASK == 0b0 && op_ & FIRST_MASK == 0b0;

        self.acc[0] = self.acc[1];
        self.acc[1] = self.acc[2];
        self.acc[2] = self.acc[3];
        self.acc[3] = self.acc[4];
        self.acc[4] = if is_fullsize && is_valid {
            let p = (my_ & TARGET_MASK).count_ones() as u8 + 1;
            if p >= self.min { p } else { 0 }
        } else {
            0
        };

        let mut max = 0;
        let mut count = 0;
        for p in self.acc {
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

    /// Potentials of the empty cells of a line written like `"-o-ox"`, cell
    /// 0 first: `o` is an own stone, `x` an opponent's.
    fn potentials(line: &str, min: u8, exact: bool) -> Vec<(u8, u8)> {
        let (mut my, mut op) = (0, 0);
        for (i, c) in line.chars().enumerate() {
            match c {
                'o' => my |= 1 << i,
                'x' => op |= 1 << i,
                _ => {}
            }
        }
        Potentials::new(line.len() as u8, my, op, min, exact).collect()
    }

    #[test]
    fn test_potentials() {
        // A window free of opponent stones is worth its own stones + 1. An
        // empty cell scores the best window through it, times how many
        // windows reach that best; below `min` it is not reported.
        //
        // Cell 7 is in three windows holding both stones (3 each): 9. Cell
        // 5 is in one of them: 3. The same holds mirrored on the right.
        let line = "--------oo-----";
        let expected = [(5, 3), (6, 6), (7, 9), (10, 9), (11, 6), (12, 3)];
        assert_eq!(potentials(line, 3, false), expected);
        // Nothing here can become an overline, so `exact` changes nothing.
        assert_eq!(potentials(line, 3, true), expected);

        // Windows through the `x` at cell 6 are worth nothing, so cell 5
        // only sees window 1 (two stones).
        let line = "--o-o-xo---ooo-";
        assert_eq!(
            potentials(line, 3, false),
            [
                (0, 3),
                (1, 6),
                (3, 6),
                (5, 3),
                (8, 6),
                (9, 4),
                (10, 8),
                (14, 4)
            ]
        );
        // With `exact`, windows 7 (cells 7-11) and 8 (cells 8-12) have an
        // own stone just outside them, at 12 and at 7, and would make an
        // overline; that leaves cell 8 with nothing.
        assert_eq!(
            potentials(line, 3, true),
            [(0, 3), (1, 6), (3, 6), (5, 3), (9, 4), (10, 8), (14, 4)]
        );
    }
}
