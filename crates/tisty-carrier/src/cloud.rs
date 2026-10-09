use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tisty_core::signing::SigningKey;
use tisty_sync::{carry_through, forget_carried_to, forget_shape};

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
const FORGOTTEN: &str = "forgotten";
const FORGETTING: &str = "forgotten.working";
const PATIENCE: Duration = Duration::from_secs(5);

struct Done<T> {
    value: Option<T>,
    deferred: Option<Deferred>,
    stopped: Option<Trouble>,
    uploaded: usize,
}

pub struct Cloud {
    remote: Arc<dyn Remote>,
    home: PathBuf,
    tried: AtomicBool,
}

impl Cloud {
    pub fn over(remote: Arc<dyn Remote>, home: impl Into<PathBuf>) -> Self {
        Self {
            remote,
            home: home.into(),
            tried: AtomicBool::new(false),
        }
    }

    fn tree(&self) -> PathBuf {
        self.home.join(TREE)
    }

    fn with_mirror<T>(
        &self,
        here: Option<&Here>,
        patient: bool,
        work: impl FnOnce(&Path, &mut Index, &dyn Remote) -> Result<(T, Option<Deferred>), Trouble>,
    ) -> Result<Done<T>, Trouble> {
        let taken = match patient {
            true => lock::Round::wait_for(&self.home, PATIENCE),
            false => lock::Round::take(&self.home),
        };
        let _round = match taken {
            Ok(Some(round)) => round,
            Ok(None) => return Ok(Done::deferred(Reason::Busy, None)),
            Err(why) => return Err(Trouble::Broke(why.to_string())),
        };
        let tree = self.tree();
        std::fs::create_dir_all(&tree).map_err(|e| Trouble::Broke(e.to_string()))?;
        self.forget_asked(&tree);
        mirror::sweep(&tree);
        let remote: &dyn Remote = &*self.remote;
        let mut index = Index::load(&self.home);
        let data = here.map(|here| here.data.as_path());
        if let Err(hitch) = mirror::pull(remote, &tree, &mut index, data) {
            let _ = index.save(&self.home);
            return match judge(hitch) {
                Judged::Pause(reason, wait) => Ok(Done::deferred(reason, wait)),
                Judged::Stop(trouble) => Err(trouble),
            };
        }
        if let Some(here) = here
            && !index.confirmed
        {
            forget_shape(&here.data, &tree);
            forget_carried_to(&here.aside, &tree);
        }
        let worked = work(&tree, &mut index, remote);
        let (value, held) = match worked {
            Ok(worked) => worked,
            Err(trouble) => {
                let _ = index.save(&self.home);
                return Err(trouble);
            }
        };
        let rated = held.is_some_and(|one| matches!(one.reason, Reason::Limit | Reason::Budget));
        let (pushed, hitch) = match rated {
            true => (mirror::Pushed::default(), None),
            false => mirror::push(remote, &tree, &mut index),
        };
        let mut deferred = held;
        let mut stopped = None;
        match hitch.map(judge) {
            Some(Judged::Pause(reason, wait)) => {
                deferred = merged(
                    deferred,
                    Some(Deferred {
                        reason,
                        left: pushed.left,
                        retry_after: wait,
                    }),
                );
            }
            Some(Judged::Stop(trouble)) => stopped = Some(trouble),
            None if pushed.left > 0 => {
                deferred = merged(
                    deferred,
                    Some(Deferred {
                        reason: Reason::Changed,
                        left: pushed.left,
                        retry_after: None,
                    }),
                );
            }
            None => {}
        }
        if let Err(why) = index.save(&self.home) {
            stopped = stopped.or(Some(Trouble::Broke(why.to_string())));
        }
        Ok(Done {
            value: Some(value),
            deferred,
            stopped,
            uploaded: pushed.put,
        })
    }

    fn only<T>(
        &self,
        here: Option<&Here>,
        work: impl FnOnce(&Path) -> Result<T, Trouble>,
    ) -> Result<T, Trouble> {
        let done = self.with_mirror(here, true, |tree, _, _| Ok((work(tree)?, None)))?;
        match done.value {
            Some(value) => Ok(value),
            None => Err(Trouble::Broke(unavailable(done.deferred))),
        }
    }

    fn fresh(&self) -> PathBuf {
        if !Index::load(&self.home).pulled() && !self.tried.swap(true, Ordering::Relaxed) {
            let _ = self.with_mirror(None, false, |_, _, _| Ok(((), None)));
        }
        self.tree()
    }

    fn forget_asked(&self, tree: &Path) {
        let (list, working) = (self.home.join(FORGOTTEN), self.home.join(FORGETTING));
        for turn in 0..2 {
            if turn == 1 {
                let _ = std::fs::rename(&list, &working);
            }
            let Ok(text) = std::fs::read_to_string(&working) else {
                continue;
            };
            for id in text.lines().filter(|line| !line.is_empty()) {
                tisty_sync::forget_paper(tree, id);
            }
            let _ = std::fs::remove_file(&working);
        }
    }
}

impl<T> Done<T> {
    fn deferred(reason: Reason, wait: Option<Duration>) -> Self {
        Self {
            value: None,
            deferred: Some(Deferred {
                reason,
                left: 0,
                retry_after: wait,
            }),
            stopped: None,
            uploaded: 0,
        }
    }
}

fn merged(first: Option<Deferred>, second: Option<Deferred>) -> Option<Deferred> {
    match (first, second) {
        (Some(one), Some(other)) => Some(Deferred {
            reason: one.reason,
            left: one.left + other.left,
            retry_after: one.retry_after.max(other.retry_after),
        }),
        (one, other) => one.or(other),
    }
}

fn unavailable(deferred: Option<Deferred>) -> String {
    match deferred {
        Some(one) => format!("the cloud cannot be used now: {:?}", one.reason),
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
        let done = self.with_mirror(Some(here), false, |tree, index, remote| {
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
            Ok(((moved, shelf.sent()), shelf.paused()))
        })?;
        let (mut moved, attached) = done.value.unwrap_or_default();
        let mirrored = moved.sent.saturating_sub(attached);
        moved.sent = attached + mirrored.min(done.uploaded);
        if let Some(trouble) = done.stopped {
            if moved == Moved::default() {
                return Err(trouble);
            }
            moved.refused = Some(trouble);
        }
        moved.deferred = done.deferred;
        Ok(moved)
    }

    fn stitch(&self, here: &Here, key: Option<SigningKey>) -> Result<Stitched, Trouble> {
        self.only(Some(here), |tree| {
            tisty_sync::stitch(&here.data, &here.device, tree, key)
        })
    }

    fn let_go(
        &self,
        here: &Here,
        above: u64,
        elsewhere: Elsewhere,
        told: Told,
    ) -> Result<LetGo, Trouble> {
        let done = self.with_mirror(Some(here), true, |_, index, remote| {
            Ok((
                shelf::let_go(remote, index, &here.data, above, elsewhere, told),
                None,
            ))
        })?;
        match done.value {
            Some(freed) => Ok(freed),
            None => Err(Trouble::Broke(unavailable(done.deferred))),
        }
    }

    fn paper_waiting(&self, id: &str) -> bool {
        tisty_sync::paper_waiting(&self.fresh(), id)
    }

    fn paper_print(&self, id: &str) -> Option<String> {
        tisty_sync::held_there(&self.fresh(), id)
    }

    fn both_papers(&self, here: &Here, id: &str) -> Result<(String, String), Trouble> {
        self.only(Some(here), |tree| {
            tisty_sync::both_papers(&here.data, tree, id)
        })
    }

    fn settle(&self, here: &Here, id: &str, keep: Keep) -> Result<Option<String>, Trouble> {
        self.only(Some(here), |tree| {
            tisty_sync::settle(&here.data, tree, id, keep)
        })
    }

    fn forget_paper(&self, id: &str) {
        if tisty_core::docs::resolve(Path::new(""), id).is_err()
            || std::fs::create_dir_all(&self.home).is_err()
        {
            return;
        }
        let asked = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.home.join(FORGOTTEN));
        if let Ok(mut list) = asked {
            let _ = writeln!(list, "{id}");
        }
        if let Ok(Some(_round)) = lock::Round::take(&self.home) {
            self.forget_asked(&self.tree());
        }
    }
}

#[cfg(test)]
#[path = "cloud_test.rs"]
mod tests;
