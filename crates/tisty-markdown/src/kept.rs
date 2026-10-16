// How the language names an attachment: one name with a stamp of its bytes, which is also where it lies.
pub const SHED: &str = "attachments";

/// An older body that still says `attachments/xx/<name>` is read by that last step.
pub fn leaf_of(reference: &str) -> Option<&str> {
    let said = reference.trim_start_matches('<').trim_end_matches('>');
    let said = said.split(['?', '#']).next().unwrap_or_default();
    let leaf = match said.strip_prefix("attachments/") {
        Some(rest) => rest.rsplit('/').next().unwrap_or_default(),
        None if !said.contains('/') => said,
        None => return None,
    };
    stamped(leaf).then_some(leaf)
}

pub fn lower_hex(said: &str) -> bool {
    said.bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn stamped_by(leaf: &str) -> &str {
    let stem = leaf.split('.').next().unwrap_or_default();
    stem.rsplit('-').next().unwrap_or_default()
}

pub fn stamped(leaf: &str) -> bool {
    let stamp = stamped_by(leaf);
    (8..=16).contains(&stamp.len())
        && lower_hex(stamp)
        && leaf.len() <= 255
        // A drive that ignores case would take another's name with only its extension in capitals.
        && !leaf.chars().any(char::is_uppercase)
        && !leaf.contains('/')
        && !leaf.contains('\\')
        && leaf.chars().all(|c| !c.is_control())
}
