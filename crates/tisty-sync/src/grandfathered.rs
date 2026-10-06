use std::path::Path;

/// True when the copy we hold is a history written before signing existed, and every segment of
/// it is still there, unchanged, at the start of the folder's copy.
pub(crate) fn wrote_here_before_signing(mine: &Path, theirs: &Path) -> bool {
    let Ok(held) = tisty_core::store::segments_in(mine) else {
        return false;
    };
    !held.is_empty()
        && held.iter().any(|one| holds_an_unsigned_line(one))
        && held.iter().all(|one| {
            one.file_name()
                .is_some_and(|name| still_begins(one, &theirs.join(name)))
        })
}

fn holds_an_unsigned_line(segment: &Path) -> bool {
    std::fs::read_to_string(segment).is_ok_and(|text| {
        text.lines().any(|line| {
            line.strip_prefix("{\"v\":")
                .map(|rest| {
                    rest.split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap_or("")
                })
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|v| v < tisty_core::event::SIGNED_FROM)
        })
    })
}

fn still_begins(ours: &Path, theirs: &Path) -> bool {
    match (std::fs::read(ours), std::fs::read(theirs)) {
        (Ok(ours), Ok(theirs)) => theirs.starts_with(&ours),
        _ => false,
    }
}
