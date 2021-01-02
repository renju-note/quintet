use super::row::*;

pub const INT_SIZE: u32 = 32;

#[derive(Clone)]
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

    pub fn rows(&self, black: bool, kind: RowKind) -> Vec<Row> {
        let blacks_: Stones;
        let whites_: Stones;
        if black {
            blacks_ = self.blacks << 1;
            whites_ = append_dummies(self.whites, self.size);
        } else {
            blacks_ = append_dummies(self.blacks, self.size);
            whites_ = self.whites << 1;
        }
        let size_ = self.size + 2;

        search_pattern(blacks_, whites_, size_, black, kind)
            .iter()
            .map(|row| Row {
                start: row.start - 1,
                size: row.size,
                eyes: row.eyes.iter().map(|x| x - 1).collect(),
            })
            .collect()
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
