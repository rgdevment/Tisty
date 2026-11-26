use tisty_core::model::Kept;
use tisty_core::places::{Places, Ruled};
use tisty_core::{Paths, State};

use super::{Refused, doc_named};

/// The core rules what Ideas, the diary and AGENTS take; here it is only worded for an assistant.
pub(super) fn refused(state: &State, file: &str, ruled: Ruled) -> Refused {
    match ruled.0 {
        "ideasStayPut" | "ideasAreOne" => super::ideas::sheet_stays_put(state, file),
        "ideasHoldNoPages" | "ideasHoldOnlyIdeas" => super::ideas::no_pages_on_a_sheet(state, file),
        _ => Refused::Tool(format!(
            "{} keeps only its own: the person's diary holds only its days, and AGENTS only the \
             assistants' files. Nothing was changed.",
            doc_named(state, file)
        )),
    }
}

/// Copying, translating or joining languages: Ideas and the diary are one each.
pub(super) fn may_copy(places: &Places, state: &State, kept: &Kept) -> Result<(), Refused> {
    places
        .may_copy(state, kept)
        .map_err(|ruled| refused(state, &kept.file, ruled))
}

/// Letting pages go takes each out of its document, so a day leaves no diary and nothing leaves Ideas.
pub(super) fn pages_let_go(paths: &Paths, state: &State, many: &[String]) -> Result<(), Refused> {
    let pages: Vec<String> = many
        .iter()
        .filter_map(|which| super::mistaken::named_document(state, which))
        .filter(|kept| kept.page_of.is_some())
        .map(|kept| kept.file.clone())
        .collect();
    let places = Places::read(&paths.docs(), state);
    match first_staying(state, &places, &pages) {
        Some((kept, ruled)) => Err(refused(state, &kept.file, ruled)),
        None => Ok(()),
    }
}

/// The first document of `many` that may not leave where it stands, with why.
pub(super) fn first_staying<'a>(
    state: &'a State,
    places: &Places,
    many: &[String],
) -> Option<(&'a Kept, Ruled)> {
    many.iter()
        .filter_map(|file| state.docs.values().find(|one| one.file == *file))
        .find_map(|kept| {
            places
                .may_file(state, kept)
                .err()
                .map(|ruled| (kept, ruled))
        })
}

#[cfg(test)]
#[path = "places_matrix_test.rs"]
mod matrix;
