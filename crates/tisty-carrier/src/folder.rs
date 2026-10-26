use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tisty_core::signing::SigningKey;

use crate::{
    Brought, Carrier, Elsewhere, Here, Holding, Keep, Kin, LetGo, Moved, Round, STORE, Signed,
    Stitched, Told, Trouble,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    at: PathBuf,
}

impl Folder {
    pub fn at(at: PathBuf) -> Self {
        Self { at }
    }

    fn shelved(&self, reference: &str) -> Option<PathBuf> {
        let ours = tisty_core::attach::names_an_attachment(reference);
        tisty_core::attach::resolve(reference, &self.at)
            .ok()
            .filter(|at| ours && at.is_file())
    }

    fn on_the_shelf(&self) -> BTreeMap<String, u64> {
        let mut found = BTreeMap::new();
        let shed = tisty_core::attach::SHED;
        for leaf in std::fs::read_dir(self.at.join(shed))
            .into_iter()
            .flatten()
            .flatten()
        {
            let named = leaf.file_name().to_string_lossy().into_owned();
            let file = leaf.metadata().ok().filter(std::fs::Metadata::is_file);
            if let Some(told) = file
                && tisty_core::attach::stamped(&named)
            {
                found.insert(format!("{shed}/{named}"), told.len());
            }
        }
        found
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

    fn frees_space(&self) -> bool {
        false
    }

    fn lends_from(&self) -> Option<PathBuf> {
        Some(self.at.join("attachments"))
    }

    fn brought(&self, _: &Here, reference: &str) -> Result<Brought, Trouble> {
        Ok(self.shelved(reference).map_or(Brought::Absent, Brought::At))
    }

    fn shelf(&self) -> Option<BTreeMap<String, u64>> {
        self.reachable().then(|| self.on_the_shelf())
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

    // A folder is never sealed, so there is nothing a hearing could learn there.
    fn hear(&self, _: &Here, _: Round) -> Result<Moved, Trouble> {
        Ok(Moved::default())
    }

    fn bring_home(&self, here: &Here, round: Round) -> Result<Moved, Trouble> {
        self.carry(
            here,
            Round {
                way: crate::Way::Pull,
                holds: tisty_core::config::Holds::Everywhere,
                ..round
            },
        )
    }

    fn could_free(&self, _: &Here, _: SystemTime) -> LetGo {
        LetGo::default()
    }

    fn set_free(&self, _: &Here, _: SystemTime, _: Told) -> Result<LetGo, Trouble> {
        Ok(LetGo::default())
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

    fn settle(
        &self,
        here: &Here,
        id: &str,
        keep: Keep,
        seen: Option<&str>,
    ) -> Result<Option<String>, Trouble> {
        tisty_sync::settle(&here.data, &self.at, id, keep, seen)
    }

    fn forget_paper(&self, id: &str) {
        tisty_sync::forget_paper(&self.at, id)
    }

    fn leave_word(&self, said: &str) -> Result<(), Trouble> {
        tisty_sync::refuses(&self.at)?;
        tisty_sync::raise_place(&self.at);
        tisty_core::store::write_atomic(&self.at.join(tisty_core::moved::NAMED), said.as_bytes())
            .map_err(|_| Trouble::NotThere(self.at.display().to_string()))?;
        tisty_core::left_behind::left_notice(&self.at, said);
        Ok(())
    }

    fn word_left(&self) -> Option<String> {
        std::fs::read_to_string(self.at.join(tisty_core::moved::NAMED)).ok()
    }

    fn forget_word(&self) -> Result<(), Trouble> {
        tisty_sync::refuses(&self.at)?;
        tisty_core::left_behind::notice_gone(&self.at);
        match std::fs::remove_file(self.at.join(tisty_core::moved::NAMED)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(Trouble::NotThere(self.at.display().to_string())),
        }
    }
}
