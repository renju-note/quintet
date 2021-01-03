use super::super::board::{Line, Stones};
use super::pattern::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

#[derive(Clone)]
pub struct LineRow {
    pub kind: RowKind,
    pub start: u8,
    pub size: u8,
    pub eyes: Vec<u8>,
}

pub fn search_pattern(
    blacks: Stones,
    whites: Stones,
    within: u8,
    black: bool,
    kind: RowKind,
) -> Vec<LineRow> {
    match (black, kind) {
        (true, RowKind::Two) => search_multi(blacks, whites, within, BLACK_TWO_PATTERNS, kind),
        (true, RowKind::Sword) => search_multi(blacks, whites, within, BLACK_SWORD_PATTERNS, kind),
        (true, RowKind::Three) => search_multi(blacks, whites, within, BLACK_THREE_PATTERNS, kind),
        (true, RowKind::Four) => search_multi(blacks, whites, within, BLACK_FOUR_PATTERNS, kind),
        (true, RowKind::Five) => search_multi(blacks, whites, within, BLACK_FIVE_PATTERNS, kind),
        (true, RowKind::Overline) => {
            search_multi(blacks, whites, within, BLACK_OVERLINE_PATTERNS, kind)
        }
        (false, RowKind::Two) => search_multi(blacks, whites, within, WHITE_TWO_PATTERNS, kind),
        (false, RowKind::Sword) => search_multi(blacks, whites, within, WHITE_SWORD_PATTERNS, kind),
        (false, RowKind::Three) => search_multi(blacks, whites, within, WHITE_THREE_PATTERNS, kind),
        (false, RowKind::Four) => search_multi(blacks, whites, within, WHITE_FOUR_PATTERNS, kind),
        (false, RowKind::Five) => search_multi(blacks, whites, within, WHITE_FIVE_PATTERNS, kind),
        _ => vec![],
    }
}

fn search_multi(
    blacks: Stones,
    whites: Stones,
    within: u8,
    patterns: &[&RowPattern],
    kind: RowKind,
) -> Vec<LineRow> {
    patterns
        .iter()
        .flat_map(|p| search(blacks, whites, within, &p, kind))
        .collect()
}

fn search(
    blacks: Stones,
    whites: Stones,
    within: u8,
    pattern: &RowPattern,
    kind: RowKind,
) -> Vec<LineRow> {
    let mut result = Vec::new();
    if within < pattern.size {
        return result;
    }
    let filter: Stones = (1 << pattern.size) - 1;
    let mut blacks = blacks;
    let mut whites = whites;
    for i in 0..=(within - pattern.size) {
        if (blacks & filter & !pattern.blmask) == pattern.blacks
            && (whites & filter & !pattern.whmask) == pattern.whites
        {
            let start = i + pattern.offset;
            let row = LineRow {
                kind: kind,
                start: start,
                size: pattern.row.size,
                eyes: pattern
                    .row
                    .eyes
                    .to_vec()
                    .iter()
                    .map(|eye| eye + start)
                    .collect(),
            };
            result.push(row);
        }
        blacks = blacks >> 1;
        whites = whites >> 1;
    }
    return result;
}
