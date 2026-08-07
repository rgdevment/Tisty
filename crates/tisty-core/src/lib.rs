//! Tisty's domain core. Nothing here prints: the GUI is a client too, and
//! anything written to the terminal reaches it as garbage.

pub mod cache;
pub mod capture;
pub mod config;
pub mod event;
pub mod model;
pub mod order;
pub mod paths;
pub mod state;
pub mod store;
pub mod undo;

pub use config::Config;
pub use event::{DeviceId, Event, Op};
pub use model::{
    DateSpec, List, ListId, LogEntry, LogId, Priority, Status, Step, StepId, Tag, Task, TaskId,
};
pub use paths::Paths;
pub use state::State;
pub use store::Store;
pub use undo::inverse;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
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
