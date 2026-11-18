use std::path::Path;

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compared {
    Told(tisty_markdown::Compared),
    TooLong,
    /// Pruned since it was listed, or a version only another machine keeps.
    NotKept,
}

pub fn compared(data: &Path, id: &str, was: &str, now_or_today: Option<&str>) -> Result<Compared> {
    let docs = data.join("docs");
    crate::docs::resolve(&docs, id)?;
    let Some(before) = super::read(data, id, was) else {
        return Ok(Compared::NotKept);
    };
    let after = match now_or_today {
        Some(print) => super::read(data, id, print),
        None => Some(crate::docs::read(&docs, id)?),
    };
    Ok(match after {
        Some(after) => {
            tisty_markdown::compared(&before, &after).map_or(Compared::TooLong, Compared::Told)
        }
        None => Compared::NotKept,
    })
}

#[cfg(test)]
#[path = "compared_test.rs"]
mod tests;
