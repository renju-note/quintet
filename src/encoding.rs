use super::board::*;
use regex::Regex;

const N_RANGE: std::ops::RangeInclusive<u8> = 1..=BOARD_SIZE;

pub struct Coder {
    re: Regex,
}

impl Coder {
    pub fn new() -> Coder {
        Coder {
            re: Regex::new("[a-oA-O][0-9]+").unwrap(),
        }
    }

    pub fn encode(&self, ps: &[Point]) -> Result<String, String> {
        ps.iter().map(|p| self.encode_one(p)).collect()
    }

    pub fn encode_one(&self, p: &Point) -> Result<String, String> {
        if !N_RANGE.contains(&p.x) || !N_RANGE.contains(&p.y) {
            Err("Invalid point".to_string())
        } else {
            let mut result = String::new();
            result.push_str(&x_to_str(p.x));
            result.push_str(&y_to_str(p.y));
            Ok(result)
        }
    }

    pub fn decode(&self, s: &str) -> Result<Vec<Point>, String> {
        self.re
            .find_iter(s)
            .map(|m| self.decode_one(m.as_str()))
            .collect()
    }

    pub fn decode_one(&self, s: &str) -> Result<Point, String> {
        let x = x_from_str(&s.chars().take(1).collect::<String>());
        let y = y_from_str(&s.chars().skip(1).collect::<String>());
        if !N_RANGE.contains(&x) || !N_RANGE.contains(&y) {
            Err("Invalid point".to_string())
        } else {
            Ok(Point { x: x, y: y })
        }
    }
}

fn x_to_str(x: u8) -> String {
    let code = ('A' as u8 + x - 1) as u32;
    std::char::from_u32(code).unwrap().to_string()
}

fn y_to_str(y: u8) -> String {
    y.to_string()
}

fn x_from_str(s: &str) -> u8 {
    match s.chars().nth(0) {
        Some(c) => match c {
            'A'..='O' => c as u8 - 'A' as u8 + 1,
            'a'..='o' => c as u8 - 'a' as u8 + 1,
            _ => 0,
        },
        None => 0,
    }
}

fn y_from_str(s: &str) -> u8 {
    s.parse::<u8>().unwrap_or(0)
}
