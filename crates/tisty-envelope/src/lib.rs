//! What a document becomes to travel: its HTML with everything it names in place, its pictures made anew,
//! and its envelopes sealed and signed. It keeps nothing and asks nothing of a network; whoever sends them is elsewhere.

pub mod clean;
mod colour;
pub mod family;
pub mod pictures;
pub mod reading;
pub mod sealing;
mod tags;
pub mod travelling;

pub use tisty_contracts::{Attachment, Cited, Document, Page, Picture, Sealing, Translation};
