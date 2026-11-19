use super::{Card, Kind};

pub const PROPOSAL_AT_MOST: usize = 64_000;
// The card compares both texts whole up to here, and the person approves only what was shown.
pub const PROPOSAL_LINES_AT_MOST: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unfit {
    NoCard,
    Memory,
    Retargeted,
    TooLong,
    TooManyLines,
    NotWritable,
}

pub fn proposable(approved: &str, proposed: &str) -> Result<(), Unfit> {
    let card = Card::of(approved).ok_or(Unfit::NoCard)?;
    if card.kind == Kind::Memory {
        return Err(Unfit::Memory);
    }
    if Card::of(proposed).as_ref() != Some(&card) {
        return Err(Unfit::Retargeted);
    }
    if proposed.chars().count() > PROPOSAL_AT_MOST {
        return Err(Unfit::TooLong);
    }
    let lines = |text: &str| text.lines().count();
    if lines(approved).max(lines(proposed)) > PROPOSAL_LINES_AT_MOST {
        return Err(Unfit::TooManyLines);
    }
    super::rendered(proposed)
        .map(|_| ())
        .map_err(|_| Unfit::NotWritable)
}

#[cfg(test)]
#[path = "proposal_test.rs"]
mod tests;
