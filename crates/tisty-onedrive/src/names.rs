use tisty_carrier::{Hitch, named_well};

// OneDrive counts the whole path from its root, and the app's folder sits under a localised "Apps/<name>".
pub const PATH_AT_MOST: usize = 360;

const FORBIDDEN: &[char] = &['"', '*', ':', '<', '>', '?', '\\', '|'];
const RESERVED: &[&str] = &[".lock", "con", "prn", "aux", "nul", "desktop.ini"];

/// What OneDrive would refuse is refused here, before anything is asked of it.
pub fn fitting(name: &str) -> Result<(), Hitch> {
    named_well(name)?;
    let refused = |why: &str| {
        Err(Hitch::Broke(format!(
            "OneDrive cannot hold «{name}»: {why}"
        )))
    };
    if name.chars().count() > PATH_AT_MOST {
        return refused("the path is too long");
    }
    for part in name.split('/') {
        let lower = part.to_lowercase();
        if part.contains(FORBIDDEN) {
            return refused("it has a character OneDrive forbids");
        }
        if part.starts_with(' ') || part.ends_with(' ') || part.ends_with('.') {
            return refused("it starts or ends with a space, or ends with a dot");
        }
        if RESERVED.contains(&lower.as_str()) || numbered_device(&lower) {
            return refused("the name is reserved");
        }
        if lower.contains("_vti_") || part.starts_with("~$") {
            return refused("the name is reserved");
        }
    }
    Ok(())
}

fn numbered_device(lower: &str) -> bool {
    let mut chars = lower.chars();
    let stem: String = chars.by_ref().take(3).collect();
    let rest: String = chars.collect();
    (stem == "com" || stem == "lpt")
        && rest.len() == 1
        && rest.chars().all(|one| one.is_ascii_digit())
}

#[cfg(test)]
#[path = "names_test.rs"]
mod tests;
