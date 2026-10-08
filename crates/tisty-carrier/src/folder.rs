use std::path::{Path, PathBuf};

use tisty_core::signing::SigningKey;

use crate::{
    Carrier, Elsewhere, Here, Holding, Keep, Kin, LetGo, Moved, Round, STORE, Signed, Stitched,
    Told, Trouble,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    at: PathBuf,
}

impl Folder {
    pub fn at(at: PathBuf) -> Self {
        Self { at }
    }
}

impl Carrier for Folder {
    fn place(&self) -> Option<&Path> {
        Some(&self.at)
    }

    fn reachable(&self) -> bool {
        self.at.is_dir()
    }

    fn theirs(&self) -> Option<String> {
        tisty_sync::theirs(&self.at)
    }

    fn been_here(&self, here: &Here) -> bool {
        tisty_sync::been_here(&here.aside, &self.at, &here.device)
    }

    fn kin(&self, here: &Here) -> Kin {
        tisty_sync::kinship(&here.data.join(STORE), &self.at)
    }

    fn signed(&self) -> Signed {
        tisty_sync::signed_here(&self.at)
    }

    fn stirring(&self) -> u64 {
        tisty_sync::stirring(&self.at)
    }

    fn unclaimed(&self) -> Holding {
        tisty_sync::unclaimed(&self.at)
    }

    fn carry(&self, here: &Here, round: Round) -> Result<Moved, Trouble> {
        tisty_sync::carry_telling(
            &here.data,
            Some(&here.aside),
            &here.device,
            &self.at,
            round.way,
            round.alive,
            round.holds,
            round.saying,
        )
    }

    fn stitch(&self, here: &Here, key: Option<SigningKey>) -> Result<Stitched, Trouble> {
        tisty_sync::stitch(&here.data, &here.device, &self.at, key)
    }

    fn let_go(
        &self,
        here: &Here,
        above: u64,
        elsewhere: Elsewhere,
        told: Told,
    ) -> Result<LetGo, Trouble> {
        tisty_sync::let_go_telling(&here.data, &self.at, above, elsewhere, told)
    }

    fn paper_waiting(&self, id: &str) -> bool {
        tisty_sync::paper_waiting(&self.at, id)
    }

    fn paper_print(&self, id: &str) -> Option<String> {
        tisty_sync::held_there(&self.at, id)
    }

    fn both_papers(&self, here: &Here, id: &str) -> Result<(String, String), Trouble> {
        tisty_sync::both_papers(&here.data, &self.at, id)
    }

    fn settle(&self, here: &Here, id: &str, keep: Keep) -> Result<Option<String>, Trouble> {
        tisty_sync::settle(&here.data, &self.at, id, keep)
    }

    fn forget_paper(&self, id: &str) {
        tisty_sync::forget_paper(&self.at, id)
    }
}
