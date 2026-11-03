use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tisty_core::Op;

use super::shares::{apart, doc_of, sharer, signing_name};
use crate::sharing::{self, Net};
use crate::{Answer, Refusal, Session, held};

/// Shared documents' saves and the server's hour as this machine spends it, so uploads keep within what it allows.
#[derive(Default)]
pub struct Following {
    saves: Mutex<HashMap<String, (u64, Instant)>>,
    sent: Mutex<VecDeque<Instant>>,
    reserved: Mutex<VecDeque<Instant>>,
    going: Mutex<HashSet<String>>,
    /// For each share whose holds last matched its words, the print of the cards those words had then.
    kept: Mutex<HashMap<String, String>>,
    held_back: Mutex<HashMap<String, Refusal>>,
}

// Saved words go up once the person stops this long, so a burst of saves is one version within the server's allowance.
const QUIET: Duration = Duration::from_secs(120);

// Below the server's 60 uploads an hour from one network, whose excess shuts that network out for hours or days.
const UPLOADS_AN_HOUR: usize = 40;

// Below the server's 20 new links an hour, which a share and what it cites spend together (decision 64).
const RESERVES_AN_HOUR: usize = 15;

const HOUR: Duration = Duration::from_secs(3600);

/// What `alone` takes for the round that keeps what shares cite in step, which only one runs at a time.
pub(crate) const CITING: &str = "\0citing";

impl Following {
    // Every document counts toward one allowance; what is not sent now stays owed for the next round.
    fn paced(&self) -> bool {
        self.spend(0, 1)
    }

    /// Takes room for this many new links and uploads in the hour, all of it or none.
    pub(crate) fn spend(&self, reserves: usize, uploads: usize) -> bool {
        let mut sent = self.sent.lock().unwrap_or_else(PoisonError::into_inner);
        let mut reserved = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        let now = pruned(&mut sent, &mut reserved);
        if sent.len() + uploads > UPLOADS_AN_HOUR || reserved.len() + reserves > RESERVES_AN_HOUR {
            return false;
        }
        sent.extend(std::iter::repeat_n(now, uploads));
        reserved.extend(std::iter::repeat_n(now, reserves));
        true
    }

    /// Takes `fixed` new links with an upload each and up to `wanted` more the hour has room for: how many more, or none if `fixed` does not fit.
    pub(crate) fn take(&self, fixed: usize, wanted: usize) -> Option<usize> {
        let mut sent = self.sent.lock().unwrap_or_else(PoisonError::into_inner);
        let mut reserved = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        let now = pruned(&mut sent, &mut reserved);
        let room = (UPLOADS_AN_HOUR.saturating_sub(sent.len()))
            .min(RESERVES_AN_HOUR.saturating_sub(reserved.len()));
        if fixed > room {
            return None;
        }
        let more = wanted.min(room - fixed);
        sent.extend(std::iter::repeat_n(now, fixed + more));
        reserved.extend(std::iter::repeat_n(now, fixed + more));
        Some(more)
    }

    /// Gives back room taken for what never reached the server; what reached it the server counts, so it stays spent.
    pub(crate) fn refund(&self, reserves: usize, uploads: usize) {
        let mut sent = self.sent.lock().unwrap_or_else(PoisonError::into_inner);
        let mut reserved = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        let keep = sent.len().saturating_sub(uploads);
        sent.truncate(keep);
        let keep = reserved.len().saturating_sub(reserves);
        reserved.truncate(keep);
    }

    /// One document goes up once at a time, so two rounds never reserve what it cites twice.
    pub(crate) fn alone(&self, file: &str) -> Option<Alone<'_>> {
        let mut going = self.going.lock().unwrap_or_else(PoisonError::into_inner);
        going.insert(file.to_owned()).then(|| Alone {
            following: self,
            file: file.to_owned(),
        })
    }

    /// Whether a share's words still have the cards they had when its holds last matched them, so nothing needs walking.
    pub(crate) fn unchanged(&self, file: &str, cards: &str) -> bool {
        let kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        kept.get(file).is_some_and(|was| was == cards)
    }

    pub(crate) fn settled(&self, file: &str, cards: Option<String>) {
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        match cards {
            Some(cards) => kept.insert(file.to_owned(), cards),
            None => kept.remove(file),
        };
    }

    /// A held document changed its own cards, which could reach something new, so every share walks again.
    pub(crate) fn unsettled(&self) {
        self.kept
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    // A try that never reached the server, or that it turned away, spends nothing of its allowance.
    fn unpaced(&self) {
        self.sent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_back();
    }

    pub(crate) fn held_back(&self, file: &str, why: Option<Refusal>) {
        let mut held_back = self
            .held_back
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        match why {
            Some(why) => held_back.insert(file.to_owned(), why),
            None => held_back.remove(file),
        };
    }

    pub(crate) fn held_back_of(&self, file: &str) -> Option<Refusal> {
        let held_back = self
            .held_back
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        held_back.get(file).cloned()
    }

    fn resting(&self, file: &str) -> bool {
        let saves = self.saves.lock().unwrap_or_else(PoisonError::into_inner);
        saves.get(file).is_none_or(|(_, at)| at.elapsed() >= QUIET)
    }
}

// What went more than an hour ago no longer counts against it.
fn pruned(sent: &mut VecDeque<Instant>, reserved: &mut VecDeque<Instant>) -> Instant {
    let now = Instant::now();
    for spent in [sent, reserved] {
        while spent
            .front()
            .is_some_and(|at| now.duration_since(*at) > HOUR)
        {
            spent.pop_front();
        }
    }
    now
}

/// Waits up to a minute for a turn another round holds, so a press is not refused for a round running in the background.
pub(crate) async fn waited<'a>(following: &'a Following, key: &str) -> Option<Alone<'a>> {
    for _ in 0..240 {
        if let Some(alone) = following.alone(key) {
            return Some(alone);
        }
        let _ = tauri::async_runtime::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(250));
        })
        .await;
    }
    None
}

pub(crate) struct Alone<'a> {
    following: &'a Following,
    file: String,
}

impl Drop for Alone<'_> {
    fn drop(&mut self) {
        let mut going = self
            .following
            .going
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        going.remove(&self.file);
    }
}

/// Whether this file is shared and follows what is saved, asked by whoever just saved it.
pub(crate) fn follows(session: &Session, file: &str) -> bool {
    doc_of(session, file)
        .ok()
        .and_then(|doc| session.state.shares.get(&doc))
        .is_some_and(|one| !one.still)
}

/// Puts a shared document's saved words up as a new version once its saves rest (decision 48).
pub(crate) fn follow(app: &tauri::AppHandle, file: &str) {
    if !sharing::ON {
        return;
    }
    use tauri::Manager;
    let mine = {
        let following = app.state::<Following>();
        let mut saves = following
            .saves
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let (count, at) = saves.entry(file.to_owned()).or_insert((0, Instant::now()));
        *count += 1;
        *at = Instant::now();
        *count
    };
    let (app, file) = (app.clone(), file.to_owned());
    std::thread::spawn(move || {
        std::thread::sleep(QUIET);
        let following = app.state::<Following>();
        let last = following
            .saves
            .lock()
            .ok()
            .and_then(|saves| saves.get(&file).map(|(count, _)| *count));
        if last == Some(mine) {
            tauri::async_runtime::block_on(followed(&app, &file));
        }
    });
}

async fn followed(app: &tauri::AppHandle, file: &str) {
    use tauri::Manager;
    let following = app.state::<Following>();
    let Some(_alone) = following.alone(file) else {
        // Still going up from the round before: these words wait for it and go next.
        follow(app, file);
        return;
    };
    // What the words reach now goes up before them, so their envelope can name it.
    let kept_up = super::citing::kept_up(app, file).await;
    let limited = kept_up.as_ref().err().and_then(limiting);
    if let Err(refused) = kept_up {
        // The hour or another round was in the way: these words try again once it rests, rather than at the next save.
        if refused.code == "shareSlow" {
            follow(app, file);
        }
        tisty_core::witness::note(
            tisty_core::witness::channel::WINDOW,
            "what a shared document cites could not be kept up; the next round tries again",
            &[(
                "why",
                tisty_core::witness::Fact::Why(refused.code.to_string()),
            )],
        );
    }
    let session = app.state::<Mutex<Session>>();
    let gathered = {
        let session = held(&session);
        let doc = doc_of(&session, file).ok();
        let shared = doc.and_then(|doc| session.state.shares.get(&doc).cloned());
        let text = tisty_core::docs::read(&session.paths.docs(), file).ok();
        match (doc, shared.filter(|one| !one.still), text) {
            (Some(doc), Some(shared), Some(text)) => Some((
                doc,
                super::citing::cited_in(&text, &super::citing::held_with(&session, doc)),
                text,
                sharer(&session),
                signing_name(&session.state.signed, shared.signs),
                shared,
                session.paths.data().to_path_buf(),
            )),
            _ => None,
        }
    };
    let Some((doc, cited, text, voucher, name, shared, data)) = gathered else {
        return;
    };
    let print = super::citing::printed(&text, &cited);
    // Words already up, from here or another of the person's machines, are not put up again.
    if print == shared.print {
        following.held_back(file, limited);
        return;
    }
    // A share from before prints were kept counts its words as up; what was owed then goes with the next save.
    if shared.print.is_empty() {
        let _ = held(&session).commit(Op::ShareVersion {
            id: doc,
            link: shared.link,
            version: shared.version,
            print,
        });
        return;
    }
    let done = match voucher {
        None => Err(Refusal::of("shareNoKey")),
        Some(_) if !following.paced() => {
            // The hour's allowance is spent; this tries again once a little of it frees up.
            follow(app, file);
            Err(Refusal::of("shareSlow"))
        }
        Some((voucher, _)) => {
            let asked = shared.clone();
            let up = apart(move || {
                let wire = Net::to(&asked.server)?;
                let cited = match asked.taken {
                    true => super::citing::kept_from(&wire, &asked, &text)?,
                    false => cited,
                };
                let (text, pictures) =
                    sharing::pictured(&text, |target| sharing::attachment(target, &data));
                let words = sharing::Words {
                    text: &text,
                    cited: &cited,
                    pictures: &pictures,
                    voucher: &voucher,
                    name: name.as_deref(),
                };
                sharing::update(&wire, &asked, &words)
            })
            .await;
            match up {
                Ok(version) => held(&session)
                    .commit(Op::ShareVersion {
                        id: doc,
                        link: shared.link,
                        version,
                        print,
                    })
                    .map_err(Refusal::from),
                // Someone put a version up since: theirs comes beside these words, which go up after it.
                Err(refused) if sharing::behind(&refused) => {
                    let brought = brought(app, file).await;
                    if matches!(brought, Ok(true)) {
                        follow(app, file);
                    }
                    brought.map(|_| ())
                }
                Err(refused) => {
                    following.unpaced();
                    // One's own share the server let go of is forgotten, as its row would on a look.
                    if sharing::already_gone(&refused) {
                        let _ = held(&session).commit(Op::ShareStop {
                            id: doc,
                            link: shared.link,
                        });
                    }
                    Err(refused)
                }
            }
        }
    };
    following.held_back(file, done.as_ref().err().and_then(limiting).or(limited));
    if let Err(refused) = done {
        tisty_core::witness::note(
            tisty_core::witness::channel::WINDOW,
            "a shared document's saved words did not go up; the next round tries again",
            &[(
                "why",
                tisty_core::witness::Fact::Why(refused.code.to_string()),
            )],
        );
    }
}

pub(crate) fn limiting(refused: &Refusal) -> Option<Refusal> {
    matches!(
        refused.code,
        "sharePicturesFull" | "shareTooManyCited" | "shareTooBig"
    )
    .then(|| refused.clone())
}

/// Brings the newest version someone else put up of a share this store holds, beside the person's own words (decision 9).
pub(crate) async fn brought(app: &tauri::AppHandle, file: &str) -> Answer<bool> {
    use tauri::Manager;
    let session = app.state::<Mutex<Session>>();
    let (doc, shared, mine, data, cache) = {
        let session = held(&session);
        let doc = doc_of(&session, file)?;
        let Some(shared) = session.state.shares.get(&doc).cloned() else {
            return Ok(false);
        };
        let me = session.state.persons.person(&session.config.device_id);
        (
            doc,
            shared,
            me.map(|one| one.fingerprint()),
            session.paths.data().to_path_buf(),
            session.paths.cache().to_path_buf(),
        )
    };
    let asked = shared.clone();
    let got = apart(move || {
        let wire = Net::to(&asked.server)?;
        let newer = sharing::latest(&wire, &asked)?.filter(|one| *one > asked.version);
        let Some(latest) = newer else {
            return Ok(None);
        };
        let said = sharing::version(&wire, &asked, latest)?;
        // The number is the server's word and the version inside is the writer's: they must agree, or neither is believed.
        let got = crate::received::opened_from(said, &asked.key).and_then(|got| {
            match got.version == latest {
                true => Ok(got),
                false => Err(Refusal::of("shareOpenOlder")),
            }
        });
        Ok(Some((latest, got)))
    })
    .await;
    let (latest, got) = match got {
        Ok(Some((latest, Ok(got)))) => (latest, got),
        // A version nobody can vouch for is passed over, so the person's own words go up after it.
        Ok(Some((latest, Err(unopened)))) => {
            tisty_core::witness::warn(
                tisty_core::witness::channel::WINDOW,
                "a share's newer version did not open or did not hold, and was passed over",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(unopened.code.to_string()),
                )],
            );
            held(&session)
                .commit(Op::ShareVersion {
                    id: doc,
                    link: shared.link,
                    version: latest,
                    print: String::new(),
                })
                .map_err(Refusal::from)?;
            return Ok(true);
        }
        Ok(None) => return Ok(false),
        // Someone else's share that went, or no longer takes this token, is forgotten here and stays theirs.
        Err(refused)
            if shared.taken && (sharing::already_gone(&refused) || forbidden(&refused)) =>
        {
            held(&session)
                .commit(Op::ShareStop {
                    id: doc,
                    link: shared.link,
                })
                .map_err(Refusal::from)?;
            return Ok(false);
        }
        Err(refused) => return Err(refused),
    };
    let mut session = held(&session);
    // What this person's own key wrote came from another of their machines, which the sync brings whole.
    if mine.as_deref() != Some(got.print.as_str()) {
        let (body, told) = crate::received::kept_pictures(&got, &data, &cache);
        if !told.is_empty() {
            session.commit_all(told).map_err(Refusal::from)?;
        }
        // A name anyone can choose, so the fingerprint that cannot be chosen goes beside it.
        let by = format!(
            "{} · {}",
            got.signer.as_deref().unwrap_or("Tisty Share"),
            got.print
        );
        super::papers::other_version_kept(&mut session, file, body, Some(&by))?;
    }
    session
        .commit(Op::ShareVersion {
            id: doc,
            link: shared.link,
            version: latest,
            print: String::new(),
        })
        .map_err(Refusal::from)?;
    Ok(true)
}

fn forbidden(refused: &Refusal) -> bool {
    refused.code == "shareRefused" && refused.name.as_deref() == Some("forbidden")
}

/// At the start and with each round: brings every share's versions put up by others, and puts up what is owed.
pub(crate) async fn brought_all(app: &tauri::AppHandle) {
    if !sharing::ON {
        return;
    }
    use tauri::Manager;
    let following = app.state::<Following>();
    let files: Vec<String> = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        session
            .state
            .shares
            .keys()
            .filter_map(|doc| session.state.docs.get(doc).map(|one| one.file.clone()))
            .collect()
    };
    for file in files {
        if let Err(refused) = brought(app, &file).await {
            tisty_core::witness::note(
                tisty_core::witness::channel::WINDOW,
                "a share's newer version could not be brought; the next round asks again",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(refused.code.to_string()),
                )],
            );
        }
        // Saved and not up, as when Tisty closed before the words rested; words still being written wait for their rest.
        if following.resting(&file) {
            followed(app, &file).await;
        }
    }
}
