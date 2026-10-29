use tisty_share_proto::envelope::{Envelope, Refused as Unread};
use tisty_share_proto::identity::Verdict;
use tisty_share_proto::limits::Limits;
use zeroize::Zeroizing;

use crate::keys::{self, Randomness, Slowness};
use crate::store::{Doc, Fault, Kept, Saved, Store};

const DAY: i64 = 24 * 60 * 60;
/// A read writes down activity at most this often: days decide an expiry, and every write costs a store.
const ACTIVITY_GRAIN: i64 = 60 * 60;

/// The numbers a server runs on, all of them its own to change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub limits: Limits,
    /// Versions kept of each document, so an owner can still see what someone else changed.
    pub versions: usize,
    /// Seconds a document may stay without activity before it goes.
    pub idle: i64,
    /// Seconds a document stays paused in its owner's trash before it goes.
    pub trash: i64,
    /// Seconds a reserved id waits for its first version.
    pub reserved: i64,
    /// Wrong passwords in a row before the gate closes for a while.
    pub misses: u32,
    pub locked: i64,
    pub slowness: Slowness,
}

impl Default for Policy {
    /// Decisions 33, 49 and 51: five versions, thirty idle days, thirty days in the trash.
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            versions: 5,
            idle: 30 * DAY,
            trash: 30 * DAY,
            reserved: 60 * 60,
            misses: 5,
            locked: 15 * 60,
            slowness: Slowness::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("there is no such document")]
    Missing,
    /// It existed and is gone: expired, deleted or emptied from the trash.
    #[error("this document is no longer shared")]
    Gone,
    #[error("its owner put this document away for now")]
    Paused,
    #[error("this link does not give that permission")]
    Forbidden,
    #[error("this document asks for its password")]
    PasswordAsked,
    #[error("that is not its password")]
    PasswordWrong,
    #[error("too many wrong passwords; try again later")]
    PasswordLocked { until: i64 },
    /// Someone else uploaded first; Tisty shows both and lets the person choose.
    #[error("there is a newer version than the one this was written on")]
    Behind { latest: Option<u64> },
    #[error("this envelope is not for this document")]
    Misaddressed,
    #[error("the envelope was refused: {0}")]
    Envelope(Unread),
    /// A signature the envelope carries does not hold, so it would claim someone it is not.
    #[error("the envelope's signature does not hold")]
    Unsigned,
    #[error("no free id could be found; try again")]
    Crowded,
    #[error("there was no randomness to mint with")]
    NoRandomness,
    #[error("{0}")]
    Store(#[from] Fault),
}

/// The three links of a new document, handed back once: the server keeps only their hashes.
#[derive(Debug)]
pub struct Reserved {
    pub id: String,
    pub edit: Zeroizing<String>,
    pub admin: Zeroizing<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Served {
    pub version: u64,
    pub envelope: Vec<u8>,
}

/// What a management link can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Manage {
    /// The owner looked in on it, which counts as activity.
    Visit,
    /// A new password, or none.
    Password(Option<String>),
    /// A shorter idle time than the server's, in seconds.
    Idle(i64),
    Pause,
    Restore,
    Delete,
}

pub struct Share<S, R> {
    pub store: S,
    pub random: R,
    pub policy: Policy,
}

impl<S: Store, R: Randomness> Share<S, R> {
    pub fn new(store: S, random: R, policy: Policy) -> Self {
        Self {
            store,
            random,
            policy,
        }
    }

    /// The id comes first, since a Tisty seals the envelope for that id and signs it.
    pub async fn reserve(&mut self, now: i64) -> Result<Reserved, Refused> {
        let edit = keys::minted_token(&mut self.random).ok_or(Refused::NoRandomness)?;
        let admin = keys::minted_token(&mut self.random).ok_or(Refused::NoRandomness)?;
        for _ in 0..8 {
            let id = keys::minted_id(&mut self.random).ok_or(Refused::NoRandomness)?;
            let doc = Doc {
                id: id.clone(),
                edit: keys::hashed(&edit),
                admin: keys::hashed(&admin),
                gate: None,
                versions: Vec::new(),
                created: now,
                active: now,
                idle: self.policy.idle,
                ends: now + self.policy.reserved,
                paused: None,
                revision: 0,
            };
            if self.store.save(&doc, None).await? == Saved::Done {
                return Ok(Reserved { id, edit, admin });
            }
        }
        Err(Refused::Crowded)
    }

    /// Takes a version only if it was written on the latest one, so no change lands over another unseen.
    pub async fn upload(
        &mut self,
        id: &str,
        edit: &str,
        bytes: Vec<u8>,
        now: i64,
    ) -> Result<u64, Refused> {
        let mut doc = self.live(id, now).await?;
        if keys::hashed(edit) != doc.edit {
            return Err(Refused::Forbidden);
        }
        if doc.paused.is_some() {
            return Err(Refused::Paused);
        }
        let envelope = Envelope::read(bytes, &self.policy.limits).map_err(Refused::Envelope)?;
        let manifest = envelope.manifest();
        if manifest.id != doc.id {
            return Err(Refused::Misaddressed);
        }
        if matches!(
            envelope.sender(&[]),
            Some(Verdict::Refused | Verdict::Unreadable)
        ) {
            return Err(Refused::Unsigned);
        }
        let latest = doc.latest();
        if manifest.from != latest || manifest.version != latest.map_or(1, |one| one + 1) {
            return Err(Refused::Behind { latest });
        }
        let kept = Kept {
            version: manifest.version,
            digest: envelope.digest(),
        };
        self.store.keep(id, &kept, envelope.as_bytes()).await?;
        doc.versions.push(kept);
        let dropped: Vec<Kept> = doc
            .versions
            .drain(..doc.versions.len().saturating_sub(self.policy.versions))
            .collect();
        touched(&mut doc, now);
        if let Err(lost) = self.saved(&mut doc).await {
            // The race was lost, so what was kept for it is nobody's and goes.
            self.store.drop_envelope(id, &kept).await?;
            return Err(lost);
        }
        for old in &dropped {
            self.store.drop_envelope(id, old).await?;
        }
        Ok(kept.version)
    }

    /// What a reader gets: only ever the latest version, so nobody is served one already replaced.
    pub async fn read(
        &mut self,
        id: &str,
        password: Option<&str>,
        now: i64,
    ) -> Result<Served, Refused> {
        let mut doc = self.live(id, now).await?;
        if doc.paused.is_some() {
            return Err(Refused::Paused);
        }
        let kept = *doc.versions.last().ok_or(Refused::Missing)?;
        let gated = doc.gate.is_some();
        self.let_in(&mut doc, password, now).await?;
        let envelope = self
            .store
            .envelope(id, &kept)
            .await?
            .ok_or(Refused::Missing)?;
        if gated || now - doc.active >= ACTIVITY_GRAIN {
            touched(&mut doc, now);
            // Another reader saving the same activity first loses nothing, so a race is no refusal.
            match self.saved(&mut doc).await {
                Ok(()) | Err(Refused::Behind { .. }) => {}
                Err(other) => return Err(other),
            }
        }
        Ok(Served {
            version: kept.version,
            envelope,
        })
    }

    /// The latest version's number, for a Tisty looking for changes; asking is not activity (decision 50).
    pub async fn latest(&self, id: &str, now: i64) -> Result<Option<u64>, Refused> {
        let doc = self.live(id, now).await?;
        if doc.paused.is_some() {
            return Err(Refused::Paused);
        }
        Ok(doc.latest())
    }

    /// An earlier version, for the link that edits or manages, so a change can be seen before it is chosen.
    pub async fn version(
        &self,
        id: &str,
        token: &str,
        version: u64,
        now: i64,
    ) -> Result<Served, Refused> {
        let doc = self.live(id, now).await?;
        let hash = keys::hashed(token);
        if hash != doc.edit && hash != doc.admin {
            return Err(Refused::Forbidden);
        }
        let kept = doc
            .versions
            .iter()
            .find(|kept| kept.version == version)
            .ok_or(Refused::Missing)?;
        let envelope = self
            .store
            .envelope(id, kept)
            .await?
            .ok_or(Refused::Missing)?;
        Ok(Served { version, envelope })
    }

    pub async fn manage(
        &mut self,
        id: &str,
        admin: &str,
        what: Manage,
        now: i64,
    ) -> Result<(), Refused> {
        let mut doc = self.live(id, now).await?;
        if keys::hashed(admin) != doc.admin {
            return Err(Refused::Forbidden);
        }
        match what {
            Manage::Delete => return Ok(self.store.forget(id).await?),
            Manage::Visit => touched(&mut doc, now),
            Manage::Password(None) => doc.gate = None,
            Manage::Password(Some(password)) => {
                let mut salt = [0u8; 16];
                if !self.random.fill(&mut salt) {
                    return Err(Refused::NoRandomness);
                }
                doc.gate = Some(
                    keys::gate(&password, salt, self.policy.slowness)
                        .ok_or(Refused::NoRandomness)?,
                );
            }
            Manage::Idle(seconds) => {
                doc.idle = seconds.min(self.policy.idle).max(1);
                touched(&mut doc, now);
            }
            // Paused, the idle clock stops and the trash's starts (decision 51).
            Manage::Pause => {
                doc.paused = Some(now);
                doc.ends = now + self.policy.trash;
            }
            Manage::Restore => {
                doc.paused = None;
                touched(&mut doc, now);
            }
        }
        self.saved(&mut doc).await
    }

    /// Everything whose time ran out goes, links and envelopes alike; a Worker runs this on a schedule.
    pub async fn sweep(&self, now: i64) -> Result<usize, Refused> {
        let ending = self.store.ending(now).await?;
        for id in &ending {
            self.store.forget(id).await?;
        }
        Ok(ending.len())
    }

    /// A document whose time ran out reads as gone even before a sweep takes it away.
    async fn live(&self, id: &str, now: i64) -> Result<Doc, Refused> {
        match self.store.doc(id).await? {
            None => Err(Refused::Missing),
            Some(doc) if doc.ends < now => {
                self.store.forget(id).await?;
                Err(Refused::Gone)
            }
            Some(doc) => Ok(doc),
        }
    }

    /// A wrong password counts, and enough in a row close the gate for a while (decision 32).
    async fn let_in(
        &mut self,
        doc: &mut Doc,
        password: Option<&str>,
        now: i64,
    ) -> Result<(), Refused> {
        let Some(gate) = doc.gate.as_mut() else {
            return Ok(());
        };
        if let Some(until) = gate.locked_until.filter(|until| *until > now) {
            return Err(Refused::PasswordLocked { until });
        }
        let Some(password) = password else {
            return Err(Refused::PasswordAsked);
        };
        if keys::opens(gate, password, self.policy.slowness) {
            gate.missed = 0;
            gate.locked_until = None;
            return Ok(());
        }
        gate.missed += 1;
        if gate.missed >= self.policy.misses {
            gate.missed = 0;
            gate.locked_until = Some(now + self.policy.locked);
        }
        self.saved(doc).await?;
        Err(Refused::PasswordWrong)
    }

    async fn saved(&self, doc: &mut Doc) -> Result<(), Refused> {
        let was = doc.revision;
        doc.revision += 1;
        match self.store.save(doc, Some(was)).await? {
            Saved::Done => Ok(()),
            // Whoever saved first changed what this was decided on, so the caller learns what is there now.
            Saved::Raced => Err(Refused::Behind {
                latest: self
                    .store
                    .doc(&doc.id)
                    .await?
                    .and_then(|held| held.latest()),
            }),
        }
    }
}

/// Activity pushes the end back; a paused document keeps the trash's end until it is restored.
fn touched(doc: &mut Doc, now: i64) {
    doc.active = now;
    if doc.paused.is_none() {
        doc.ends = now + doc.idle;
    }
}

#[cfg(test)]
#[path = "share_test.rs"]
mod tests;
