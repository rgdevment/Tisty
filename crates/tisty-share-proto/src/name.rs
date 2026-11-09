use unicode_normalization::UnicodeNormalization;

/// A person's name or alias, as long as Tisty lets one be.
pub const NAME_AT_MOST: usize = 80;

/// A name is a line a person can read as written: not too long, no edge of spaces, nothing unseen that hides
/// a letter or turns the text around, and no more digits than letters, so it never passes for a fingerprint.
pub fn named_well(name: &str) -> bool {
    let letters = name.chars().filter(|one| one.is_alphabetic()).count();
    let digits = name.chars().filter(char::is_ascii_digit).count();
    name.trim() == name
        && name.chars().count() <= NAME_AT_MOST
        && letters > 0
        && digits <= letters
        && !name.chars().any(|one| one.is_control() || unseen(one))
}

fn unseen(one: char) -> bool {
    matches!(
        one,
        '\u{00AD}'
            | '\u{034F}'
            | '\u{061C}'
            | '\u{115F}'
            | '\u{1160}'
            | '\u{17B4}'..='\u{17B5}'
            | '\u{180B}'..='\u{180F}'
            | '\u{200B}'..='\u{200F}'
            | '\u{2028}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{2800}'
            | '\u{3164}'
            | '\u{FEFF}'
            | '\u{FFA0}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0000}'..='\u{E007F}'
    )
}

// Folio's look-alike skeleton, letter for letter, so both sides catch the same imitations.
const LOOKS_LATIN: [(char, char); 28] = [
    ('а', 'a'),
    ('в', 'b'),
    ('г', 'r'),
    ('е', 'e'),
    ('к', 'k'),
    ('м', 'm'),
    ('н', 'h'),
    ('о', 'o'),
    ('р', 'p'),
    ('с', 'c'),
    ('т', 't'),
    ('у', 'y'),
    ('х', 'x'),
    ('α', 'a'),
    ('ε', 'e'),
    ('ι', 'i'),
    ('κ', 'k'),
    ('μ', 'm'),
    ('ν', 'v'),
    ('ο', 'o'),
    ('ρ', 'p'),
    ('τ', 't'),
    ('υ', 'u'),
    ('0', 'o'),
    ('1', 'i'),
    ('l', 'i'),
    ('3', 'e'),
    ('5', 's'),
];

const UNKNOWN: char = '?';
const UNKNOWN_AT_MOST: usize = 2;
const LONG_ENOUGH_FOR_A_SLIP: usize = 6;

fn looks_latin(one: char) -> char {
    LOOKS_LATIN
        .iter()
        .find(|(from, _)| *from == one)
        .map_or(one, |(_, to)| *to)
}

fn kept(one: char) -> Option<char> {
    match one {
        _ if one.is_ascii_lowercase() || one.is_ascii_digit() => Some(one),
        _ if one.is_alphabetic() || one.is_numeric() => Some(UNKNOWN),
        _ => None,
    }
}

fn skeleton(said: &str) -> Vec<char> {
    let folded: String = said
        .nfkd()
        .collect::<String>()
        .to_lowercase()
        .chars()
        .filter(|one| !unicode_normalization::char::is_combining_mark(*one))
        .map(looks_latin)
        .filter_map(kept)
        .collect();
    folded
        .replace("rn", "m")
        .replace("vv", "w")
        .chars()
        .collect()
}

fn same(alias: char, name: char) -> bool {
    alias == UNKNOWN || alias == name
}

fn slips_apart(alias: &[char], name: &[char]) -> usize {
    let mut rows = vec![vec![0; name.len() + 1]; alias.len() + 1];
    for (at, row) in rows.iter_mut().enumerate() {
        row[0] = at;
    }
    for (column, cell) in rows[0].iter_mut().enumerate() {
        *cell = column;
    }
    for row in 1..=alias.len() {
        for column in 1..=name.len() {
            let cost = usize::from(!same(alias[row - 1], name[column - 1]));
            let mut here = (rows[row - 1][column] + 1)
                .min(rows[row][column - 1] + 1)
                .min(rows[row - 1][column - 1] + cost);
            let swapped = row > 1
                && column > 1
                && same(alias[row - 1], name[column - 2])
                && same(alias[row - 2], name[column - 1]);
            if swapped {
                here = here.min(rows[row - 2][column - 2] + 1);
            }
            rows[row][column] = here;
        }
    }
    rows[alias.len()][name.len()]
}

// A letter the table does not know stands for any one letter; a few at most, or another script would match everything.
fn looks_like(alias: &[char], name: &[char]) -> bool {
    let unknown = alias.iter().filter(|one| **one == UNKNOWN).count();
    if unknown > UNKNOWN_AT_MOST || unknown + 1 >= name.len() {
        return false;
    }
    let slips = usize::from(name.len() >= LONG_ENOUGH_FOR_A_SLIP);
    slips_apart(alias, name) <= slips
}

pub fn imitates(alias: &str, name: &str) -> bool {
    looks_like(&skeleton(alias), &skeleton(name))
}

#[cfg(test)]
#[path = "name_test.rs"]
mod tests;
