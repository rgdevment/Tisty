//! From a document kept on this machine to its envelopes: what it names is found and put in place, a window colours and draws it, and it is sealed.
//! It sends nothing. Whoever sends the envelopes tells this what went, so the next version can keep what did not change.

mod finishing;
mod gathering;
mod packing;
mod receiving;
mod remembering;

pub use finishing::{Carrying, Finished, Sealed, Unfinished};
pub use gathering::{Asked, Desk, Gathered, Under, citable, gathered, gathered_under, signer};
pub use packing::Told;
pub use receiving::{CARRIED_CITED, Received, received};
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
    #[error("it could not be read: {0}")]
    Unread(String),
    #[error("what is remembered of it could not be written: {0}")]
    Unwritten(String),
    #[error("Folio already holds as many languages of it as it takes")]
    LanguagesFull,
    #[error("the window did not finish {0}")]
    Unfinished(String),
    #[error(transparent)]
    Refused(#[from] tisty_envelope::sealing::Refused),
}
