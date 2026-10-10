use std::path::{Path, PathBuf};
use std::sync::Arc;

use tisty_core::config::Holds;
use tisty_core::event::{DeviceId, TaskAdd};
use tisty_core::witness;
use tisty_core::{Op, Store};
use ulid::Ulid;

use crate::folder_backed::FolderBacked;
use crate::{Carrier, Cloud, Here, Kin, Moved, Round, STORE, Trouble, Way};

const PAPERS: &str = "docs";

static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The cases the round answers for on a shared folder, run on a cloud whose provider keeps its
/// files in that same folder: what a case plants there, the cloud holds.
struct Shared {
    room: tempfile::TempDir,
    root: PathBuf,
    remote: Arc<FolderBacked>,
}

impl Shared {
    fn new() -> Self {
        let room = tempfile::tempdir().unwrap();
        let root = room.path().join("provider");
        let remote = Arc::new(FolderBacked::at(&root));
        Self { room, root, remote }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn cloud(&self, device: &str, data: &Path) -> (Cloud, Here) {
        let home = self.room.path().join("homes").join(device);
        let here = Here {
            data: data.to_path_buf(),
            aside: home.join("cache"),
            device: device.to_string(),
        };
        (Cloud::over(self.remote.clone(), home.join("cloud")), here)
    }

    fn carry(&self, who: &Machine, way: Way, alive: &[String]) -> Result<Moved, Trouble> {
        let (cloud, here) = self.cloud(&who.device, &who.data);
        let mut saying = |_| {};
        cloud.carry(
            &here,
            Round {
                way,
                alive,
                holds: Holds::Everywhere,
                saying: &mut saying,
            },
        )
    }

    fn kin(&self, who: &Machine) -> Kin {
        let (cloud, here) = self.cloud(&who.device, &who.data);
        cloud.kin(&here)
    }

    fn stirring(&self) -> u64 {
        let (cloud, _) = self.cloud("onlooker", &self.room.path().join("onlooker"));
        cloud.stirring()
    }

    fn signed_at(&self) -> Option<String> {
        let (cloud, _) = self.cloud("onlooker", &self.room.path().join("onlooker"));
        cloud.signed().alias
    }
}

macro_rules! both_sides {
    ($($case:ident),* $(,)?) => {
        mod cloud {
            $(
                #[test]
                fn $case() {
                    super::$case(&super::Shared::new());
                }
            )*
        }
    };
}

macro_rules! cloud_owes {
    ($($case:ident),* $(,)?) => {
        mod cloud_owes {
            $(
                #[test]
                #[ignore = "a cloud answers kin and stirring from its last round's mirror"]
                fn $case() {
                    super::$case(&super::Shared::new());
                }
            )*
        }
    };
}

include!("../../tisty-sync/src/round_cases_test.rs");
