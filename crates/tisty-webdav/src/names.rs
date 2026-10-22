use tisty_carrier::{Hitch, named_well};

// Many servers keep their files on Windows or a share that follows its rules, so only what every one of them holds is written.
const FORBIDDEN: &[char] = &['"', '*', ':', '<', '>', '?', '\\', '|'];

/// What some WebDAV server would refuse or change is refused here, before anything is asked of it.
pub fn fitting(name: &str) -> Result<(), Hitch> {
    named_well(name)?;
    let refused = |why: &str| {
        Err(Hitch::Broke(format!(
            "a WebDAV server may not hold «{name}»: {why}"
        )))
    };
    for part in name.split('/') {
        if part.contains(FORBIDDEN) {
            return refused("it has a character some servers forbid");
        }
        if part.starts_with(' ') || part.ends_with(' ') || part.ends_with('.') {
            return refused("it starts or ends with a space, or ends with a dot");
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "names_test.rs"]
mod tests;
