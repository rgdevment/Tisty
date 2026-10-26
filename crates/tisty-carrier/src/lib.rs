use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tisty_core::config::{CloudAt, Holds, Sync};
use tisty_core::signing::SigningKey;

pub use tisty_core::turned;
pub use tisty_sync::{
    Cause, Deferred, Holding, Keep, Kin, LetGo, Moved, NAMED, NEEDS, Reached, Reason, SEALS, SHAPE,
    STORE, Sealing, Signed, Stage, Stitched, TURNS, Trouble, Turn, Undecided, Way, claim_in,
    claimed, phase, raised, raised_told, refuses, seal_word, sealing_in, sealing_of,
};

mod cloud;
mod folder;
#[cfg(not(windows))]
mod keychain;
pub mod loopback;
#[cfg(feature = "net")]
pub mod net;
pub mod off;
mod remote;
pub mod said;
mod sealed;
mod shift;
mod temp;
#[cfg(feature = "testing")]
pub mod testing;
mod tokens;
pub mod uploads;

pub use cloud::sealing::{Left, Stepped, Turning, idle_for, turn_of, turn_size, wipe_behind};
pub use cloud::standing::{
    CARRIED, LastRound, OWED_BACK, Standing, cloud_joined, kin_mirrored, mirrored_at,
    round_forgotten, round_noted, standing,
};
pub use cloud::{Cloud, forgotten_later, knock, only_up_there, pause_for, policy};
pub use folder::Folder;
#[cfg(not(windows))]
pub use keychain::Keychain;

pub use remote::{
    BYTES_A_DAY, Changes, Costs, Expect, Hitch, Limits, Remote, Seen, UNITS_A_DAY, Watch,
    named_well, renamed_by_copy,
};
pub use sealed::Sealed;
pub use shift::{Owed, Shift, all_home, filling, homing, homing_up_to, shift};
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
    /// It was in the cloud and left it, trashed or deleted there, and nothing up there owes it.
    Gone,
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
    /// Takes an attachment however a body names it, by its leaf or whole, since the window passes what it read.
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
    fn settle(
        &self,
        here: &Here,
        id: &str,
        keep: Keep,
        seen: Option<&str>,
    ) -> Result<Option<String>, Trouble>;
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
    /// Asked of the place itself: nothing is left there but the word that everyone moved on.
    fn only_word_left(&self) -> bool {
        false
    }
    fn forget_word(&self) -> Result<(), Trouble> {
        Ok(())
    }
    /// Asked of the place in one look, never a whole pull, so a first round can be told apart before it starts.
    /// `ours` is the place chosen now; `None` is a place that could not be asked.
    fn peeks_a_store(&self, _ours: bool) -> Option<bool> {
        Some(self.holds_a_store())
    }
    fn holds_a_store(&self) -> bool {
        self.place()
            .is_some_and(|dest| tisty_core::store::inhabited(dest.join(STORE)))
    }
    fn later(&self) -> Option<Trouble> {
        self.place().and_then(tisty_sync::later)
    }
    /// Never overridden: a carrier that empties itself does so in `wipe_along`.
    fn wipe(&self) -> Result<usize, Trouble> {
        self.wipe_along(&mut |_, _| {})
    }
    /// Says how many are gone of how many as it goes.
    fn wipe_along(&self, _along: &mut dyn FnMut(usize, usize)) -> Result<usize, Trouble> {
        Ok(0)
    }
    /// Bytes the place can still take, where it says; a folder's disk is never asked.
    fn room(&self) -> Option<u64> {
        None
    }
    /// Who else writes here, as far as this machine has read the place.
    fn others(&self, here: &Here) -> Vec<String> {
        self.place()
            .map(|dest| tisty_sync::others_in(dest, &here.device))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    Alone,
    Folder,
    Cloud,
    Later,
}

impl Chosen {
    pub fn mode(self) -> &'static str {
        match self {
            Chosen::Alone => "alone",
            Chosen::Folder => "folder",
            Chosen::Cloud => "cloud",
            Chosen::Later => "later",
        }
    }
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

/// A folder this machine carried into may hold attachments freed or shared from here, so only the window's move leaves it.
pub fn strands_on_leaving(was: &Sync, now: &Sync, holds: Holds, here: &Here) -> bool {
    was != now
        && was.leaving() == tisty_core::config::Leaving::Place
        && holds != Holds::Everywhere
        && chosen(Some(was))
            .carrier
            .is_some_and(|carrier| carrier.reachable() && carrier.been_here(here))
}

pub fn place_of(sync: Option<&Sync>) -> Option<PathBuf> {
    chosen(sync)
        .carrier
        .and_then(|carrier| carrier.place().map(Path::to_path_buf))
}

// A cloud's mirror is laid out as a shared folder, so a history that waits is read alike from either.
pub fn histories_of(paths: &tisty_core::Paths, sync: Option<&Sync>) -> Option<PathBuf> {
    place_of(sync).or_else(|| {
        cloud_of(sync).map(|at| mirrored_at(&paths.cloud_home(&at.provider, &at.account)))
    })
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
