use super::row::*;
use std::collections::HashMap;

const INT_SIZE: u32 = 32;

pub struct Line {
    pub size: u32,
    pub blacks: Stones,
    pub whites: Stones,
    rows_cache: HashMap<(bool, RowKind), Vec<Row>>,
}

impl Line {
    pub fn new(size: u32, blacks: Stones, whites: Stones) -> Result<Line, String> {
        if size < 1 || INT_SIZE < size {
            return Err("Wrong size".to_owned());
        }
        if blacks & whites != 0b0 {
            return Err("Blacks and whites are overlapping".to_owned());
        }
        Ok(Line::new_raw(size, blacks, whites))
    }

    pub fn put(&self, black: bool, i: u32) -> Line {
        self.overlay(black, 0b1 << i)
    }

    pub fn put_multi(&self, black: bool, is: &[u32]) -> Line {
        let mut stones: Stones = 0b0;
        for i in is {
            stones += 0b1 << i;
        }
        self.overlay(black, stones)
    }

    pub fn remove(&self, black: bool, i: u32) -> Line {
        self.overlay(black, 0b1 << i)
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> std::slice::Iter<'_, Row> {
        let size = self.size;
        let blacks = self.blacks;
        let whites = self.whites;
        self.rows_cache
            .entry((black, kind))
            .or_insert_with(|| compute_rows(size, blacks, whites, black, kind))
            .iter()
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..self.size {
            let pat = 0b1 << i;
            let c = if self.blacks & pat != 0b0 {
                'o'
            } else if self.whites != 0b0 {
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
            rows_cache: HashMap::new(),
        }
    }

    fn overlay(&self, black: bool, stones: Stones) -> Line {
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
}

fn compute_rows(size: u32, blacks: Stones, whites: Stones, black: bool, kind: RowKind) -> Vec<Row> {
    let blacks_: Stones;
    let whites_: Stones;
    if black {
        blacks_ = blacks << 1;
        whites_ = append_dummies(whites, size);
    } else {
        blacks_ = append_dummies(blacks, size);
        whites_ = whites << 1;
    }
    let size_ = size + 2;

    search_pattern(blacks_, whites_, size_, black, kind)
        .into_iter()
        .map(|row| Row {
            start: row.start - 1,
            size: row.size,
            eyes: row.eyes.into_iter().map(|x| x - 1).collect(),
        })
        .collect()
}

fn append_dummies(stones: Stones, size: u32) -> Stones {
    (stones << 1) | 0b1 | (0b1 << (size + 1))
}
