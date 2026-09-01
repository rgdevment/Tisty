pub mod agent;
pub mod attach;
pub mod backup;
pub mod cache;
pub mod capture;
pub mod config;
pub mod docs;
pub mod event;
pub mod herald;
pub mod icloud;
pub mod merge;
pub mod model;
pub mod order;
pub mod paths;
pub mod refs;
pub mod series;
pub mod shape;
pub mod state;
pub mod store;
pub mod story;
pub mod text;
pub mod tidy;
pub mod undo;
pub mod view;
pub mod witness;

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
    #[error("{0} points outside the store")]
    OutsideTheStore(String),
    #[error("{0} is not a kind of file an assistant may keep")]
    NotForAnAgent(String),
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
    #[error("{file} holds {found} events, not what it was sealed with: it arrived incomplete")]
    TruncatedSegment {
        file: String,
        found: usize,
        declared: Option<usize>,
    },
    #[error("segment {number:06} of {device} is missing: that slice of history is not here")]
    MissingSegment { number: usize, device: String },
    #[error("event schema version {0} is newer than this build understands")]
    UnsupportedVersion(u32),
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
            Error::OutsideTheStore(_) => "outsideTheStore",
            Error::NotForAnAgent(_) => "notForAnAgent",
            Error::OtherStore { .. } => "otherStore",
            Error::TooBig => "tooBig",
            Error::AttachmentTooBig { .. } => "attachmentTooBig",
            Error::DocumentTooBig { .. } => "documentTooBig",
            Error::TextTooLong { .. } => "textTooLong",
            Error::Json(_) => "json",
            Error::MalformedEvent { .. } => "malformedEvent",
            Error::TruncatedSegment { .. } => "truncatedSegment",
            Error::MissingSegment { .. } => "missingSegment",
            Error::UnsupportedVersion(_) => "unsupportedVersion",
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
            Error::UnsupportedVersion(v) => facts.push(("version", Fact::Count(*v as usize))),
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
mod tests {
    use super::*;

    fn under_home(rest: &str) -> String {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/someone".into());
        std::path::Path::new(&home).join(rest).display().to_string()
    }

    fn fact_named<'a>(facts: &'a [(&'static str, witness::Fact)], key: &str) -> &'a witness::Fact {
        &facts.iter().find(|(name, _)| *name == key).expect(key).1
    }

    #[test]
    fn a_broken_segment_names_its_file_without_naming_the_account() {
        let at = under_home("tisty/store/dev_a3f1/000001.tisty");
        let error = Error::MalformedEvent {
            file: at.clone(),
            line: 35,
            source: serde_json::from_str::<u8>("nonsense").unwrap_err(),
        };

        let facts = error.told();

        assert!(
            matches!(fact_named(&facts, "file"), witness::Fact::Path(_)),
            "a path carried as an id skips the redaction and takes the home directory with it"
        );
        assert!(matches!(
            fact_named(&facts, "line"),
            witness::Fact::Count(35)
        ));
    }

    #[test]
    fn a_torn_segment_names_its_file_the_same_way() {
        let error = Error::TruncatedSegment {
            file: under_home("tisty/store/dev_a3f1/000001.tisty"),
            found: 12,
            declared: Some(20),
        };

        assert!(matches!(
            fact_named(&error.told(), "file"),
            witness::Fact::Path(_)
        ));
    }

    #[test]
    fn every_error_says_which_one_it_is() {
        let facts = Error::NoHomeDirectory.told();

        assert!(matches!(
            fact_named(&facts, "code"),
            witness::Fact::Code("noHomeDirectory")
        ));
    }
}
