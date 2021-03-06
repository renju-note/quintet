use super::board::*;

const N_RANGE: std::ops::Range<u8> = 0..BOARD_SIZE;

pub fn decode_board(code: &str) -> Result<Board, String> {
    let codes = code.trim().split('/').collect::<Vec<_>>();
    if codes.len() != 2 {
        return Err("Invalid".to_owned());
    }
    let blacks = match decode(codes[0]) {
        Ok(points) => points,
        Err(s) => return Err(s),
    };
    let whites = match decode(codes[1]) {
        Ok(points) => points,
        Err(s) => return Err(s),
    };
    let mut board = Board::new();
    for p in blacks {
        board.put(true, p);
    }
    for p in whites {
        board.put(false, p);
    }
    Ok(board)
}

pub fn encode(ps: &[Point]) -> Result<String, String> {
    let result = ps
        .iter()
        .map(|p| encode_one(p))
        .collect::<Result<Vec<_>, String>>();
    result.map(|ss| ss.join(","))
}

pub fn encode_one(p: &Point) -> Result<String, String> {
    if !N_RANGE.contains(&p.x) || !N_RANGE.contains(&p.y) {
        Err("Invalid point".to_string())
    } else {
        let mut result = String::new();
        result.push_str(&x_to_str(p.x));
        result.push_str(&y_to_str(p.y));
        Ok(result)
    }
}

pub fn decode(s: &str) -> Result<Vec<Point>, String> {
    s.split(',').map(|m| decode_one(m)).collect()
}

pub fn decode_one(s: &str) -> Result<Point, String> {
    let x = x_from_str(&s.chars().take(1).collect::<String>());
    let y = y_from_str(&s.chars().skip(1).collect::<String>());
    if !N_RANGE.contains(&x) || !N_RANGE.contains(&y) {
        Err("Invalid point".to_string())
    } else {
        Ok(Point { x: x, y: y })
    }
}

fn x_to_str(x: u8) -> String {
    let code = ('A' as u8 + x) as u32;
    std::char::from_u32(code).unwrap().to_string()
}

fn y_to_str(y: u8) -> String {
    (y + 1).to_string()
}

fn x_from_str(s: &str) -> u8 {
    match s.chars().nth(0) {
        Some(c) => match c {
            'A'..='O' => c as u8 - 'A' as u8,
            'a'..='o' => c as u8 - 'a' as u8,
            _ => 0,
        },
        None => 0,
    }
}

fn y_from_str(s: &str) -> u8 {
    s.parse::<u8>().unwrap_or(0) - 1
}
