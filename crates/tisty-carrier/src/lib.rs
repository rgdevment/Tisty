use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tisty_core::config::{CloudAt, Holds, Sync};
use tisty_core::signing::SigningKey;

pub use tisty_core::turned;
pub use tisty_sync::{
    Deferred, Holding, Keep, Kin, LetGo, Moved, Reached, Reason, STORE, Signed, Stage, Stitched,
    Trouble, Undecided, Way,
};

mod cloud;
mod folder;
mod keychain;
mod remote;
mod shift;
mod tokens;

pub use cloud::{Cloud, forgotten_later, policy};
pub use folder::Folder;
pub use keychain::Keychain;

pub use remote::{
    BYTES_A_DAY, Changes, Costs, Expect, Hitch, Limits, Remote, Seen, UNITS_A_DAY, Watch,
    named_well,
};
pub use shift::{Owed, Shift, filling, homing, shift};
pub use tokens::{Renew, Renewed, Secrets, Tokens};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Here {
    pub data: PathBuf,
    pub aside: PathBuf,
    pub device: String,
}

impl Here {
    pub fn of(paths: &tisty_core::Paths, config: &tisty_core::Config) -> Self {
        Self {
            data: paths.data().to_path_buf(),
            aside: paths.cache().to_path_buf(),
            device: config.device_id.0.clone(),
        }
    }
}

pub type Shared = Arc<dyn Carrier>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Brought {
    At(PathBuf),
    /// Held only to its name, since no history answers for it yet: the caller writes `AttachKept`.
    Unvouched {
        at: PathBuf,
        sha256: String,
        bytes: u64,
    },
    Absent,
    Later,
}

pub struct Round<'a> {
    pub way: Way,
    pub alive: &'a [String],
    pub holds: Holds,
    pub saying: &'a mut dyn FnMut(Reached),
}

pub type Elsewhere<'a> = &'a dyn Fn(&str) -> bool;
pub type Told<'a> = &'a mut dyn FnMut(&LetGo) -> bool;

pub trait Carrier: Send + std::marker::Sync {
    fn place(&self) -> Option<&Path>;
    fn lends_from(&self) -> Option<PathBuf>;
    /// Attachments always stay here and space is freed by hand: there is no copy kept only there.
    fn frees_space(&self) -> bool;
    fn brought(&self, here: &Here, reference: &str) -> Result<Brought, Trouble>;
    fn shelf(&self) -> Option<BTreeMap<String, u64>>;
    fn reachable(&self) -> bool;
    fn theirs(&self) -> Option<String>;
    fn been_here(&self, here: &Here) -> bool;
    fn kin(&self, here: &Here) -> Kin;
    fn signed(&self) -> Signed;
    fn stirring(&self) -> u64;
    fn unclaimed(&self) -> Holding;
    fn carry(&self, here: &Here, round: Round) -> Result<Moved, Trouble>;
    fn bring_home(&self, here: &Here, round: Round) -> Result<Moved, Trouble>;
    fn could_free(&self, here: &Here, unused_since: SystemTime) -> LetGo;
    fn set_free(&self, here: &Here, unused_since: SystemTime, told: Told)
    -> Result<LetGo, Trouble>;
    fn stitch(&self, here: &Here, key: Option<SigningKey>) -> Result<Stitched, Trouble>;
    fn let_go(
        &self,
        here: &Here,
        above: u64,
        elsewhere: Elsewhere,
        told: Told,
    ) -> Result<LetGo, Trouble>;
    fn paper_waiting(&self, id: &str) -> bool;
    fn paper_print(&self, id: &str) -> Option<String>;
    fn both_papers(&self, here: &Here, id: &str) -> Result<(String, String), Trouble>;
    fn settle(&self, here: &Here, id: &str, keep: Keep) -> Result<Option<String>, Trouble>;
    fn forget_paper(&self, id: &str);
    /// A way to hear the other side change, for a carrier whose provider can say so; a folder is only looked at.
    fn hears(&self) -> Option<Box<dyn Watch>> {
        None
    }
    /// Signed word that this machine stopped syncing here; a carrier that keeps nothing keeps no word either.
    fn leave_word(&self, _said: &str) -> Result<(), Trouble> {
        Ok(())
    }
    fn word_left(&self) -> Option<String> {
        None
    }
    fn forget_word(&self) -> Result<(), Trouble> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    Alone,
    Folder,
    Cloud,
    Later,
}

/// What only the window can build: the command line and the agents never reach the network.
pub trait Reach: Send + std::marker::Sync {
    fn cloud(&self, at: &CloudAt) -> Option<Shared>;
}

pub struct Keeping {
    pub carrier: Option<Shared>,
    pub chosen: Chosen,
}

pub fn chosen(sync: Option<&Sync>) -> Keeping {
    chosen_with(sync, None)
}

// The one match on the way of syncing: everything else asks the carrier it was handed.
pub fn chosen_with(sync: Option<&Sync>, reach: Option<&dyn Reach>) -> Keeping {
    match sync {
        None | Some(Sync::Local) => Keeping {
            carrier: None,
            chosen: Chosen::Alone,
        },
        Some(Sync::Folder(at)) => Keeping {
            carrier: Some(Arc::new(Folder::at(at.clone()))),
            chosen: Chosen::Folder,
        },
        Some(Sync::Cloud(at)) => Keeping {
            carrier: reach.and_then(|reach| reach.cloud(at)),
            chosen: Chosen::Cloud,
        },
        // A way a later build knows: nothing is carried, and the choice is left as it was read.
        Some(Sync::Unknown(_)) => Keeping {
            carrier: None,
            chosen: Chosen::Later,
        },
    }
}

pub fn cloud_of(sync: Option<&Sync>) -> Option<&CloudAt> {
    match sync {
        Some(Sync::Cloud(at)) => Some(at),
        _ => None,
    }
}

// A folder somebody is only considering: nothing is chosen, so only what a place answers for itself is asked.
pub fn considering(at: impl Into<PathBuf>) -> Shared {
    Arc::new(Folder::at(at.into()))
}

pub fn place_of(sync: Option<&Sync>) -> Option<PathBuf> {
    chosen(sync)
        .carrier
        .and_then(|carrier| carrier.place().map(Path::to_path_buf))
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "remote_fakes.rs"]
mod fakes;

#[cfg(test)]
#[path = "remote_folder_backed.rs"]
mod folder_backed;

#[cfg(test)]
#[path = "round_test.rs"]
mod round_tests;

#[cfg(test)]
#[path = "remote_test.rs"]
mod remote_tests;
