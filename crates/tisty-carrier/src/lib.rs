use std::path::{Path, PathBuf};
use std::sync::Arc;

use tisty_core::config::{Holds, Sync};
use tisty_core::signing::SigningKey;

pub use tisty_core::turned;
pub use tisty_sync::{
    Deferred, Holding, Keep, Kin, LetGo, Moved, Reached, Reason, STORE, Signed, Stage, Stitched,
    Trouble, Undecided, Way,
};

mod cloud;
mod folder;
mod remote;

pub use cloud::Cloud;
pub use folder::Folder;
pub use remote::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Watch, named_well};

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
    fn reachable(&self) -> bool;
    fn theirs(&self) -> Option<String>;
    fn been_here(&self, here: &Here) -> bool;
    fn kin(&self, here: &Here) -> Kin;
    fn signed(&self) -> Signed;
    fn stirring(&self) -> u64;
    fn unclaimed(&self) -> Holding;
    fn carry(&self, here: &Here, round: Round) -> Result<Moved, Trouble>;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    Alone,
    Folder,
    Later,
}

pub struct Keeping {
    pub carrier: Option<Shared>,
    pub chosen: Chosen,
}

// The one match on the way of syncing: everything else asks the carrier it was handed.
pub fn chosen(sync: Option<&Sync>) -> Keeping {
    match sync {
        None | Some(Sync::Local) => Keeping {
            carrier: None,
            chosen: Chosen::Alone,
        },
        Some(Sync::Folder(at)) => Keeping {
            carrier: Some(Arc::new(Folder::at(at.clone()))),
            chosen: Chosen::Folder,
        },
        // A way a later build knows: nothing is carried, and the choice is left as it was read.
        Some(Sync::Unknown(_)) => Keeping {
            carrier: None,
            chosen: Chosen::Later,
        },
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
#[path = "remote_test.rs"]
mod remote_tests;
