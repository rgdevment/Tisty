mod agreeing;
mod card;
mod found;
mod gate;
mod homes;
mod joined;
mod kept;
mod ledger;
mod proposal;
mod rooted;
mod shown;
mod standing;
mod subs;
mod twins;

pub use agreeing::{Agreeing, Approved, Hand, agreeing};
pub use card::{Card, Endings, GLOBAL, KEY, Kind, brought, doc_of, rendered};
pub use found::{Found, Place, Skip, brought_in, found};
pub use gate::{Admitted, Asked, Gate, Seen, gate, unclaimed};
pub use homes::{
    KNOWN, Told, assistant_folders, assistant_root, home_here, inside_an_assistant, told_here,
};
pub use joined::joined;
pub use kept::{KEPT_AT_MOST, Kept, kept, read_kept};
pub use ledger::Ledger;
pub use proposal::{PROPOSAL_AT_MOST, PROPOSAL_LINES_AT_MOST, Unfit, proposable};
pub use rooted::{FOLDER, ICON, holds, holds_folder, root_of, roots, tree, unhooks};
pub use shown::{OwnFront, Shown, own_front, shown};
pub use standing::{
    Standing, Target, adopted, bring, lacks, lead_in, lead_out, overwrite, removed, restore, round,
    standing, wanted,
};
pub use subs::{sub_folders, sub_id};
pub use twins::renamed_twins;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    NoConsent,
    Off,
    Held,
    AgentsOwn,
    Empty,
    NoCard,
    Name,
    NotText,
    BrokenFence,
    TooBig,
    Linked,
    Skipped(Skip),
    Secret(String),
    Unattributed,
    Changed,
    NoSuchBackup,
    Io(String),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let said = match self {
            Self::NoConsent => "this computer has not been allowed to read or write agent files",
            Self::Off => "sync is turned off for this assistant",
            Self::Held => {
                "a document in this file changed without the person's approval, so the file is left as it is"
            }
            Self::AgentsOwn => "the agent writes this file itself, so Tisty only reads it",
            Self::Empty => "there is nothing to write, and an agent file is never emptied",
            Self::NoCard => "the document carries no agent card",
            Self::Name => "the name or level cannot be written as an agent address",
            Self::NotText => "the file is not UTF-8 text",
            Self::BrokenFence => "the code block that holds the file is no longer whole",
            Self::TooBig => "the file is larger than a document may hold",
            Self::Linked => "the file is a link, and writing would replace the link",
            Self::Skipped(why) => return write!(f, "the file is left out: {why:?}"),
            Self::Secret(told) => {
                return write!(f, "the file reads like it holds a credential ({told})");
            }
            Self::Unattributed => {
                "the change cannot be told apart from the other documents in the file"
            }
            Self::Changed => "the file changed while it was being written, so it was left alone",
            Self::NoSuchBackup => "there is no such backup",
            Self::Io(why) => why,
        };
        f.write_str(said)
    }
}

impl From<std::io::Error> for Refused {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<crate::Error> for Refused {
    fn from(e: crate::Error) -> Self {
        Self::Io(e.to_string())
    }
}

// Outside every folder that sync, parcels and backups carry, so nothing here leaves this computer.
pub fn folder(data: &std::path::Path) -> std::path::PathBuf {
    data.join("agents")
}

fn link(at: &std::path::Path) -> bool {
    std::fs::symlink_metadata(at).is_ok_and(|meta| meta.file_type().is_symlink())
}
