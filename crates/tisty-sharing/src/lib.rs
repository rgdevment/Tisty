//! From a document kept on this machine to its envelopes: what it names is found and put in place, a window colours and draws it, and it is sealed.
//! It sends nothing. Whoever sends the envelopes tells this what went, so the next version can keep what did not change.

mod finishing;
mod gathering;
mod maker;
mod packing;
mod receiving;
mod remembering;

pub use finishing::{Carrying, Finished, Sealed, Unfinished, Word};
pub use gathering::{
    Asked, Desk, Gathered, Offered, Under, citable, gathered, gathered_under, gathered_within,
    languages_offered, left_out_of, signer, signer_among,
};
pub use maker::{certified_by, signs_for};
pub use packing::Told;
pub use receiving::{
    CARRIED_CITED, Languages, Received, brought_back, editors_brought, languages_carried,
    languages_gone, languages_joined, languages_set_aside, map_brought, received, reviews_carried,
    set_aside, with_reviews,
};
pub use remembering::{forgotten, remembered, sent};
pub use tisty_core::event::Remembered;

#[derive(Debug, thiserror::Error)]
pub enum Unshared {
    #[error("there is no such document")]
    NoSuchDoc,
    #[error("only what is one's own is shared")]
    NotOwn,
    #[error("a locked document stays here")]
    Shut,
    #[error("what is put away stays here")]
    Away,
    #[error("a page goes with its document, and a task's words stay with the task")]
    NotADocument,
    #[error("Brainstorm stays here, and its ideas travel in the documents they go into")]
    Brainstorm,
    #[error("it could not be read: {0}")]
    Unread(String),
    #[error("what is remembered of it could not be written: {0}")]
    Unwritten(String),
    #[error("Folio already holds as many languages of it as it takes")]
    LanguagesFull,
    #[error(
        "its last version came from a newer Tisty, and sealing it here would drop what this one does not know"
    )]
    Newer,
    #[error("the window did not finish {0}")]
    Unfinished(String),
    #[error(transparent)]
    Refused(#[from] tisty_envelope::sealing::Refused),
}
