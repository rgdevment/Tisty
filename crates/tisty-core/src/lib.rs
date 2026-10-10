pub mod agent;
pub mod answering;
mod applying;
pub mod arriving;
pub mod attach;
pub mod backup;
pub mod cache;
pub mod called;
pub mod capture;
pub mod config;
pub mod counting;
pub mod docs;
pub mod doubled;
pub mod event;
pub mod herald;
pub mod holes;
pub mod keepers;
pub mod lately;
pub mod machine;
pub mod merge;
pub mod model;
pub mod moving;
pub mod order;
pub mod parcel;
pub mod parting;
mod parts;
pub mod paths;
pub mod refs;
pub mod seal;
pub mod series;
pub mod shape;
mod shelving;
pub mod signing;
pub mod state;
pub mod store;
pub mod story;
pub mod tagging;
pub mod text;
pub mod tidy;
pub mod turned;
pub mod undo;
pub mod unvouched;
pub mod view;
pub mod vouched;
pub mod witness;

pub use applying::Named;
pub use config::Config;
pub use event::{DeviceId, DeviceKind, Event, Op};
pub use model::{
    DateSpec, List, ListId, LogEntry, LogId, Priority, Reading, Status, Step, StepId, Tag, Task,
    TaskId,
};
pub use paths::Paths;
pub use refs::Ref;
pub use state::State;
pub use store::Store;
pub use undo::inverse;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "Tisty could not move your data out of AppData, so it stayed closed to keep the old copy safe: {0}. Close every other Tisty and open it again."
    )]
    StoreNotMoved(String),
    #[error("{0} points outside the store")]
    OutsideTheStore(String),
    #[error("{0} does not hold what its name says it holds")]
    NotForAnAgent(String),
    #[error("{0} is not a Tisty parcel")]
    NotAParcel(String),
    #[error("that parcel was written by a newer Tisty (version {0})")]
    ParcelNewer(u32),
    #[error("that parcel is locked: it was made to be carried to another machine of its own")]
    ParcelLocked,
    #[error("that is not the number this parcel was locked with")]
    WrongNumber,
    #[error("that parcel opened, and then came apart: it did not arrive whole")]
    ParcelTorn,
    #[error("there is not enough room to open that here: it needs {needs} and {free} is free")]
    NoRoom { needs: u64, free: u64 },
    #[error("there is nothing here to carry out")]
    NothingToCarry,
    #[error("a folder called {0} is already there, and an export writes a new one")]
    AlreadyTakenOut(String),
    #[error("that backup belongs to another store ({theirs})")]
    OtherStore { theirs: String },

    #[error("the backup is larger than Tisty will carry")]
    TooBig,
    #[error("that file is {bytes} bytes and the limit is {limit}")]
    AttachmentTooBig { bytes: u64, limit: u64 },
    #[error("that document is {bytes} bytes and the limit is {limit}")]
    DocumentTooBig { bytes: u64, limit: u64 },
    #[error("that text is {bytes} bytes and the limit is {limit}")]
    TextTooLong { bytes: u64, limit: u64 },
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("malformed event at {file}:{line}: {source}")]
    MalformedEvent {
        file: String,
        line: usize,
        source: serde_json::Error,
    },
    #[error("{file} holds {found} events, not what it was closed with: it arrived incomplete")]
    TruncatedSegment {
        file: String,
        found: usize,
        declared: Option<usize>,
    },
    #[error("segment {number:06} of {device} is missing: that slice of history is not here")]
    MissingSegment { number: usize, device: String },
    #[error(
        "event schema version {version}, written by {device}, is newer than this build understands: update Tisty on this machine before going on, or reading half of it would lose work"
    )]
    UnsupportedVersion { version: u32, device: String },
    #[error("another tisty process is using this device's store")]
    AlreadyRunning,
    #[error("could not determine the home directory")]
    NoHomeDirectory,
    #[error("config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("config write error: {0}")]
    ConfigWrite(#[from] toml::ser::Error),
    #[error("invalid tag: {0}")]
    Tag(#[from] model::InvalidTag),
    #[error("invalid priority: {0}")]
    Priority(#[from] model::InvalidPriority),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn coded(&self) -> &'static str {
        match self {
            Error::Io(_) => "io",
            Error::StoreNotMoved(_) => "storeNotMoved",
            Error::OutsideTheStore(_) => "outsideTheStore",
            Error::NotForAnAgent(_) => "notForAnAgent",
            Error::NotAParcel(_) => "notAParcel",
            Error::ParcelNewer(_) => "parcelNewer",
            Error::ParcelLocked => "parcelLocked",
            Error::WrongNumber => "wrongNumber",
            Error::ParcelTorn => "parcelTorn",
            Error::NoRoom { .. } => "noRoom",
            Error::NothingToCarry => "nothingToCarry",
            Error::AlreadyTakenOut(_) => "alreadyTakenOut",
            Error::OtherStore { .. } => "otherStore",
            Error::TooBig => "tooBig",
            Error::AttachmentTooBig { .. } => "attachmentTooBig",
            Error::DocumentTooBig { .. } => "documentTooBig",
            Error::TextTooLong { .. } => "textTooLong",
            Error::Json(_) => "json",
            Error::MalformedEvent { .. } => "malformedEvent",
            Error::TruncatedSegment { .. } => "truncatedSegment",
            Error::MissingSegment { .. } => "missingSegment",
            Error::UnsupportedVersion { .. } => "unsupportedVersion",
            Error::AlreadyRunning => "alreadyRunning",
            Error::NoHomeDirectory => "noHomeDirectory",
            Error::ConfigParse(_) => "configParse",
            Error::ConfigWrite(_) => "configWrite",
            Error::Tag(_) => "badTag",
            Error::Priority(_) => "badPriority",
        }
    }

    pub fn told(&self) -> Vec<(&'static str, witness::Fact)> {
        use witness::Fact;
        let mut facts = vec![("code", Fact::Code(self.coded()))];
        match self {
            Error::Io(e) => facts.push(("why", Fact::Why(e.to_string()))),
            Error::StoreNotMoved(why) => facts.push(("why", Fact::Why(why.clone()))),
            Error::ConfigParse(e) => facts.push(("why", Fact::Why(e.to_string()))),
            Error::ConfigWrite(e) => facts.push(("why", Fact::Why(e.to_string()))),
            Error::OtherStore { theirs } => facts.push(("theirs", Fact::Id(theirs.clone()))),
            Error::MalformedEvent { file, line, .. } => {
                facts.push(("file", Fact::Path(file.into())));
                facts.push(("line", Fact::Count(*line)));
            }
            Error::TruncatedSegment { file, found, .. } => {
                facts.push(("file", Fact::Path(file.into())));
                facts.push(("found", Fact::Count(*found)));
            }
            Error::MissingSegment { number, device } => {
                facts.push(("number", Fact::Count(*number)));
                facts.push(("device", Fact::Id(device.clone())));
            }
            Error::UnsupportedVersion { version, device } => {
                facts.push(("version", Fact::Count(*version as usize)));
                facts.push(("device", Fact::Id(device.clone())));
            }
            Error::AttachmentTooBig { bytes, limit } => {
                facts.push(("bytes", Fact::Bytes(*bytes)));
                facts.push(("limit", Fact::Bytes(*limit)));
            }
            _ => {}
        }
        facts
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "sixteen_test.rs"]
mod sixteen;
