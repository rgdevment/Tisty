use tisty_share_proto::envelope::CITED_AT_MOST;

use crate::keys::{self, Randomness};
use crate::share::{Refused, Share};
use crate::store::{Doc, Store};

/// Shares one document may be held by, so a reader's check of who holds it stays a few reads.
pub const HOLDERS_AT_MOST: usize = 16;

/// The end a held document shows the sweep: past any day a server runs, and exact as a number in JavaScript.
pub const HELD: i64 = 253_402_300_799;

impl<S: Store, R: Randomness> Share<S, R> {
    /// The holder takes up a document it cites, proven its owner's by that document's edit link (decision 64).
    /// A share that holds is never held, and one that is held never holds, so no ring keeps itself alive.
    pub(crate) async fn hold(
        &self,
        holder: &mut Doc,
        id: &str,
        edit: &str,
        lives: bool,
        now: i64,
    ) -> Result<(), Refused> {
        let mut held = self.live(id, now).await?;
        if keys::hashed(edit) != held.edit {
            return Err(Refused::Forbidden);
        }
        if id == holder.id || !holder.holders.is_empty() || !held.holds.is_empty() {
            return Err(Refused::Forbidden);
        }
        if held.blocked.is_some() {
            return Err(Refused::Blocked);
        }
        if held.holders.contains(&holder.id) {
            return Ok(());
        }
        if holder.holds.len() >= CITED_AT_MOST || held.holders.len() >= HOLDERS_AT_MOST {
            return Err(Refused::TooLong);
        }
        // The holder says it holds first: a holder naming one that does not hold back costs nothing, the other way round keeps it alive.
        if !holder.holds.iter().any(|one| one == id) {
            holder.holds.push(id.to_owned());
        }
        self.saved(holder).await?;
        for _ in 0..4 {
            if held.holders.is_empty() {
                held.own = lives.then_some(held.ends);
                held.ends = HELD;
            }
            held.holders.push(holder.id.clone());
            match self.saved(&mut held).await {
                Ok(()) => return Ok(()),
                Err(Refused::Behind { .. }) => held = self.live(id, now).await?,
                Err(other) => return Err(other),
            }
            if held.holders.contains(&holder.id) {
                return Ok(());
            }
        }
        Err(Refused::Behind { latest: None })
    }

    /// The holder lets go of what it cited; one with no life of its own goes once nothing holds it.
    pub(crate) async fn release(&self, holder: &mut Doc, id: &str) -> Result<(), Refused> {
        holder.holds.retain(|one| one != id);
        self.saved(holder).await?;
        self.let_go(id, &holder.id).await
    }

    /// Everything a document held or was held by forgets it before it goes, so nothing it held outlives it unseen.
    pub(crate) async fn forgotten(&self, doc: &Doc) -> Result<(), Refused> {
        for held in &doc.holds {
            self.let_go(held, &doc.id).await?;
        }
        for holder in &doc.holders {
            self.changed(holder, |one| {
                let before = one.holds.len();
                one.holds.retain(|held| *held != doc.id);
                one.holds.len() != before
            })
            .await?;
        }
        Ok(self.store.forget(&doc.id).await?)
    }

    /// A held document reads while one of its holders does, or while its own life lasts (decision 64).
    pub(crate) async fn held_up(&self, doc: &mut Doc, now: i64) -> Result<(), Refused> {
        if doc.holders.is_empty() {
            return Ok(());
        }
        let mut paused = false;
        let mut lost = Vec::new();
        for holder in &doc.holders {
            match self.store.doc(holder).await? {
                Some(one) if one.ends < now => lost.push(holder.clone()),
                Some(one) if one.paused.is_some() || one.blocked.is_some() => paused = true,
                Some(_) => return Ok(()),
                None => lost.push(holder.clone()),
            }
        }
        // A holder that went without letting go, its store failing midway, is let go of here.
        for holder in &lost {
            self.let_go(&doc.id, holder).await?;
        }
        if doc.own.is_some_and(|end| end >= now) {
            return Ok(());
        }
        match paused {
            true => Err(Refused::Paused),
            false => Err(Refused::Gone),
        }
    }

    /// One holder lets go: the document keeps its own end again, or goes when it had none.
    async fn let_go(&self, id: &str, holder: &str) -> Result<(), Refused> {
        for _ in 0..4 {
            let Some(mut doc) = self.store.doc(id).await? else {
                return Ok(());
            };
            if !doc.holders.iter().any(|one| one == holder) {
                return Ok(());
            }
            doc.holders.retain(|one| one != holder);
            if doc.holders.is_empty() {
                match (doc.blocked, doc.own.take()) {
                    // Blocked, it is evidence, kept as long as moderation keeps any (decision 42).
                    (Some(blocked), _) => doc.ends = blocked + self.policy.evidence,
                    (None, Some(end)) => doc.ends = end,
                    (None, None) => return Ok(self.store.forget(id).await?),
                }
            }
            match self.saved(&mut doc).await {
                Ok(()) => return Ok(()),
                Err(Refused::Behind { .. }) => continue,
                Err(other) => return Err(other),
            }
        }
        Err(Refused::Behind { latest: None })
    }

    /// Applies `change` to what is stored now until it saves, or not at all when it is gone or nothing changes.
    async fn changed(&self, id: &str, change: impl Fn(&mut Doc) -> bool) -> Result<(), Refused> {
        for _ in 0..4 {
            let Some(mut doc) = self.store.doc(id).await? else {
                return Ok(());
            };
            if !change(&mut doc) {
                return Ok(());
            }
            match self.saved(&mut doc).await {
                Ok(()) => return Ok(()),
                Err(Refused::Behind { .. }) => continue,
                Err(other) => return Err(other),
            }
        }
        Err(Refused::Behind { latest: None })
    }
}

/// The end of a document's own life: a held one keeps it apart, so its holders' life is what the sweep sees.
pub(crate) fn ending_at(doc: &mut Doc, end: i64) {
    match (doc.holders.is_empty(), doc.own.as_mut()) {
        (true, _) => doc.ends = end,
        (false, Some(own)) => *own = end,
        (false, None) => {}
    }
}
