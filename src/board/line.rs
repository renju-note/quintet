pub type Stones = u32;

pub const INT_SIZE: u32 = 32;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Line {
    pub size: u32,
    pub blacks: Stones,
    pub whites: Stones,
}

impl Line {
    pub fn new(size: u32, blacks: Stones, whites: Stones) -> Result<Line, String> {
        if size < 1 || INT_SIZE < size {
            return Err(String::from("Wrong size"));
        }
        if blacks & whites != 0b0 {
            return Err(String::from("Blacks and whites are overlapping"));
        }
        Ok(Line::new_raw(size, blacks, whites))
    }

    pub fn put(&self, black: bool, i: u32) -> Line {
        let stones = 0b1 << i;
        let blacks: Stones;
        let whites: Stones;
        if black {
            blacks = self.blacks | stones;
            whites = self.whites & !stones;
        } else {
            blacks = self.blacks & !stones;
            whites = self.whites | stones;
        }
        Line::new_raw(self.size, blacks, whites)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..self.size {
            let pat = 0b1 << i;
            let c = if self.blacks & pat != 0b0 {
                'o'
            } else if self.whites & pat != 0b0 {
                'x'
            } else {
                '-'
            };
            result.push(c)
        }
        result
    }

    fn new_raw(size: u32, blacks: Stones, whites: Stones) -> Line {
        Line {
            size: size,
            blacks: blacks,
            whites: whites,
        }
    }
}

fn append_dummies(stones: Stones, size: u32) -> Stones {
    (stones << 1) | 0b1 | (0b1 << (size + 1))
}
