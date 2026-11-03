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

#[cfg(test)]
#[path = "name_test.rs"]
mod tests;
