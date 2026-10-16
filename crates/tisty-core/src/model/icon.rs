pub use tisty_markdown::vocab::ICONS;

/// How many of ICONS each family takes, in order: the window rules the picker off these cuts.
pub const FAMILIES: &[(&str, usize)] = &[
    ("home", 105),
    ("table", 62),
    ("work", 70),
    ("dev", 106),
    ("gear", 51),
    ("account", 65),
    ("talk", 40),
    ("file", 63),
    ("body", 81),
    ("money", 78),
    ("away", 103),
    ("study", 101),
    ("media", 50),
    ("wild", 89),
    ("thing", 91),
    ("symbol", 226),
];

const MARK_AT_MOST: usize = 8;

pub fn known(key: &str) -> bool {
    kept(key).is_some()
}

pub fn kept(key: &str) -> Option<String> {
    if let Some(named) = ICONS.iter().copied().find(|named| *named == key) {
        return Some(named.to_string());
    }
    a_mark(key).then(|| key.to_string())
}

pub fn a_mark(key: &str) -> bool {
    let many = key.chars().count();
    if many == 0 || many > MARK_AT_MOST {
        return false;
    }
    if key.chars().any(|one| one as u32 == 0x20E3) {
        return keyed(key);
    }
    key.chars().next().is_some_and(drawn) && key.chars().all(|one| drawn(one) || joins(one))
}

fn keyed(key: &str) -> bool {
    let capped = |one: char| one.is_ascii_digit() || one == '#' || one == '*';
    key.chars().next().is_some_and(capped) && key.chars().all(|one| capped(one) || joins(one))
}

fn drawn(one: char) -> bool {
    matches!(one as u32,
        0x00A9 | 0x00AE | 0x203C | 0x2049 | 0x2122 | 0x2139
        | 0x2194..=0x21AA | 0x231A..=0x231B | 0x2328
        | 0x23CF..=0x23FA | 0x24C2 | 0x25AA..=0x25FE
        | 0x2600..=0x27BF | 0x2934..=0x2935 | 0x2B00..=0x2BFF
        | 0x3030 | 0x303D | 0x3297 | 0x3299
        | 0x1F000..=0x1FAFF)
}

fn joins(one: char) -> bool {
    matches!(one as u32, 0xFE0F | 0x200D | 0x20E3 | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F)
}

#[cfg(test)]
#[path = "icon_test.rs"]
mod tests;
