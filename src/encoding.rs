use super::board::square::*;
use super::foundation::*;
use regex::Regex;

const N_RANGE: std::ops::RangeInclusive<u32> = 1..=N;

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

fn x_to_str(x: u32) -> String {
    std::char::from_u32('A' as u32 + x - 1).unwrap().to_string()
}

fn y_to_str(y: u32) -> String {
    y.to_string()
}

fn x_from_str(s: &str) -> u32 {
    match s.chars().nth(0) {
        Some(c) => match c {
            'A'..='O' => c as u32 - 'A' as u32 + 1,
            'a'..='o' => c as u32 - 'a' as u32 + 1,
            _ => 0,
        },
        None => 0,
    }
}

fn y_from_str(s: &str) -> u32 {
    s.parse::<u32>().unwrap_or(0)
}
