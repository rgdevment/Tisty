use tisty_core::State;
use tisty_core::model::Kept;
use tisty_core::places::{Places, Ruled};

use super::{Refused, doc_named};

/// The core rules what Ideas, the diary and AGENTS take; here it is only worded for an assistant.
pub(super) fn refused(state: &State, file: &str, ruled: Ruled) -> Refused {
    match ruled.0 {
        "ideasHoldNoPages" | "ideasStayPut" | "ideasHoldOnlyIdeas" | "ideasAreOne" => {
            super::ideas::no_pages_on_a_sheet(state, file)
        }
        _ => Refused::Tool(format!(
            "{} keeps only its own: the person's diary holds only its days, and AGENTS only the \
             assistants' files. Nothing was changed.",
            doc_named(state, file)
        )),
    }
}

pub(super) fn filed_refused(state: &State, file: &str, ruled: Ruled) -> Refused {
    match ruled.0 {
        "ideasStayPut" => super::ideas::sheet_stays_put(state, file),
        _ => refused(state, file, ruled),
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
