use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tisty_core::signing::SigningKey;
use tisty_sync::carry_through;

use crate::{
    Carrier, Deferred, Elsewhere, Here, Holding, Keep, Kin, LetGo, Moved, Reason, Remote, Round,
    STORE, Signed, Stitched, Told, Trouble,
};

mod index;
mod judge;
mod lock;
mod mirror;
mod shelf;

use index::Index;
use judge::{Judged, judge};
use shelf::Shelf;

const TREE: &str = "tree";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pause {
    reason: Reason,
    wait: Option<Duration>,
    left: usize,
}

struct Done<T> {
    value: Option<T>,
    pause: Option<Pause>,
}

pub struct Cloud {
    remote: Arc<dyn Remote>,
    home: PathBuf,
}

impl Cloud {
    pub fn over(remote: Arc<dyn Remote>, home: impl Into<PathBuf>) -> Self {
        Self {
            remote,
            home: home.into(),
        }
    }

    fn tree(&self) -> PathBuf {
        self.home.join(TREE)
    }

    fn with_mirror<T>(
        &self,
        work: impl FnOnce(&Path, &mut Index, &dyn Remote) -> Result<(T, Option<Pause>), Trouble>,
    ) -> Result<Done<T>, Trouble> {
        let Some(_round) = lock::Round::take(&self.home) else {
            return Ok(Done::paused(Reason::Busy, None, 0));
        };
        let tree = self.tree();
        std::fs::create_dir_all(&tree).map_err(|e| Trouble::Broke(e.to_string()))?;
        let remote: &dyn Remote = &*self.remote;
        let mut index = Index::load(&self.home);
        if let Err(hitch) = mirror::pull(remote, &tree, &mut index) {
            let _ = index.save(&self.home);
            return match judge(hitch) {
                Judged::Pause(reason, wait) => Ok(Done::paused(reason, wait, 0)),
                Judged::Stop(trouble) => Err(trouble),
            };
        }
        let worked = work(&tree, &mut index, remote);
        let (pushed, hitch) = mirror::push(remote, &tree, &mut index);
        let saved = index.save(&self.home);
        let (value, held) = worked?;
        let pause = match hitch.map(judge) {
            Some(Judged::Stop(trouble)) => return Err(trouble),
            Some(Judged::Pause(reason, wait)) => Some(Pause {
                reason,
                wait,
                left: pushed.left,
            }),
            None => held,
        };
        saved.map_err(|e| Trouble::Broke(e.to_string()))?;
        Ok(Done {
            value: Some(value),
            pause,
        })
    }

    fn only<T>(&self, work: impl FnOnce(&Path) -> Result<T, Trouble>) -> Result<T, Trouble> {
        let done = self.with_mirror(|tree, _, _| Ok((work(tree)?, None)))?;
        done.value
            .ok_or_else(|| Trouble::Broke(unavailable(done.pause)))
    }

    fn fresh(&self) -> PathBuf {
        if !Index::load(&self.home).pulled() {
            let _ = self.with_mirror(|_, _, _| Ok(((), None)));
        }
        self.tree()
    }
}

impl<T> Done<T> {
    fn paused(reason: Reason, wait: Option<Duration>, left: usize) -> Self {
        Self {
            value: None,
            pause: Some(Pause { reason, wait, left }),
        }
    }
}

fn unavailable(pause: Option<Pause>) -> String {
    match pause {
        Some(Pause { reason, .. }) => format!("the cloud cannot be used now: {reason:?}"),
        None => "the cloud cannot be used now".to_string(),
    }
}

impl Carrier for Cloud {
    fn place(&self) -> Option<&Path> {
        None
    }

    fn reachable(&self) -> bool {
        true
    }

    fn theirs(&self) -> Option<String> {
        tisty_sync::theirs(&self.fresh())
    }

    fn been_here(&self, here: &Here) -> bool {
        tisty_sync::been_here(&here.aside, &self.tree(), &here.device)
    }

    fn kin(&self, here: &Here) -> Kin {
        tisty_sync::kinship(&here.data.join(STORE), &self.fresh())
    }

    fn signed(&self) -> Signed {
        tisty_sync::signed_here(&self.fresh())
    }

    fn stirring(&self) -> u64 {
        tisty_sync::stirring(&self.tree())
    }

    fn unclaimed(&self) -> Holding {
        tisty_sync::unclaimed(&self.fresh())
    }

    fn carry(&self, here: &Here, round: Round) -> Result<Moved, Trouble> {
        let done = self.with_mirror(|tree, index, remote| {
            let mut shelf = Shelf::over(remote, index);
            let moved = carry_through(
                &here.data,
                Some(&here.aside),
                &here.device,
                tree,
                round.way,
                round.alive,
                round.holds,
                round.saying,
                &mut shelf,
            )?;
            let held = shelf
                .paused()
                .map(|(reason, wait, left)| Pause { reason, wait, left });
            Ok((moved, held))
        })?;
        let mut moved = done.value.unwrap_or_default();
        moved.deferred = done.pause.map(|one| Deferred {
            reason: one.reason,
            left: one.left,
            retry_after: one.wait,
        });
        Ok(moved)
    }

    fn stitch(&self, here: &Here, key: Option<SigningKey>) -> Result<Stitched, Trouble> {
        self.only(|tree| tisty_sync::stitch(&here.data, &here.device, tree, key))
    }

    fn let_go(
        &self,
        here: &Here,
        above: u64,
        elsewhere: Elsewhere,
        told: Told,
    ) -> Result<LetGo, Trouble> {
        let done = self.with_mirror(|_, index, remote| {
            Ok((
                shelf::let_go(remote, index, &here.data, above, elsewhere, told),
                None,
            ))
        })?;
        done.value
            .ok_or_else(|| Trouble::Broke(unavailable(done.pause)))
    }

    fn paper_waiting(&self, id: &str) -> bool {
        tisty_sync::paper_waiting(&self.fresh(), id)
    }

    fn paper_print(&self, id: &str) -> Option<String> {
        tisty_sync::held_there(&self.fresh(), id)
    }

    fn both_papers(&self, here: &Here, id: &str) -> Result<(String, String), Trouble> {
        self.only(|tree| tisty_sync::both_papers(&here.data, tree, id))
    }

    fn settle(&self, here: &Here, id: &str, keep: Keep) -> Result<Option<String>, Trouble> {
        self.only(|tree| tisty_sync::settle(&here.data, tree, id, keep))
    }

    fn forget_paper(&self, id: &str) {
        let _ = self.only(|tree| {
            tisty_sync::forget_paper(tree, id);
            Ok(())
        });
    }
}

#[cfg(test)]
#[path = "cloud_test.rs"]
mod tests;
