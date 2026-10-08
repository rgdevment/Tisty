use std::sync::Mutex;

use tauri::{Emitter, Manager};
use tisty_core::Op;
use tisty_core::witness::{self, Fact, channel};

use crate::{
    Answer, Carrying, Filter, OneAtATime, Refusal, Scope, Session, Settling, blamed, elsewhere,
    glimpse, held, report, room, said, session, show, today, within,
};

fn said_now_held(session: &tauri::State<'_, Mutex<Session>>, took_in: &[(String, String, u64)]) {
    if took_in.is_empty() {
        return;
    }
    let said_so = took_in
        .iter()
        .map(|(at, sha256, bytes)| tisty_core::Op::AttachKept {
            d: tisty_core::event::Held {
                at: at.clone(),
                sha256: sha256.clone(),
                bytes: *bytes,
            },
        })
        .collect();
    if let Err(e) = held(session).commit_all(said_so) {
        witness::warn(
            channel::SYNC,
            "bodies came in and the log was not told this machine now holds them",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
}

const SAID_EVERY: std::time::Duration = std::time::Duration::from_millis(250);
const KEPT_EVERY: std::time::Duration = std::time::Duration::from_secs(2);
const KEPT_AT_MOST: usize = 64;

#[derive(Clone, serde::Serialize)]
struct Stuck {
    code: &'static str,
    name: Option<String>,
}

#[derive(Clone, serde::Serialize)]
struct Bringing {
    stage: &'static str,
    done: usize,
    whole: usize,
    joining: bool,
}

/// What landed is told in handfuls, so a round cut short still owns what it already brought.
struct Telling {
    app: tauri::AppHandle,
    kept: Vec<(String, String, u64)>,
    kept_at: std::time::Instant,
    said: Option<(&'static str, std::time::Instant)>,
    joining: bool,
}

impl Telling {
    fn new(app: tauri::AppHandle, joining: bool) -> Self {
        Self {
            app,
            kept: Vec::new(),
            kept_at: std::time::Instant::now(),
            said: None,
            joining,
        }
    }

    fn hear(&mut self, far: tisty_carrier::Reached) {
        match far {
            tisty_carrier::Reached::Log => {
                let _ = self.app.emit("carried", "log");
            }
            tisty_carrier::Reached::Papers => {
                let _ = self.app.emit("carried", "papers");
            }
            tisty_carrier::Reached::Along { stage, done, whole } => {
                let stage = match stage {
                    tisty_carrier::Stage::Log => "log",
                    tisty_carrier::Stage::Papers => "papers",
                    tisty_carrier::Stage::Attachments => "attachments",
                };
                let quiet = self
                    .said
                    .is_some_and(|(was, at)| was == stage && at.elapsed() < SAID_EVERY);
                if quiet && done < whole {
                    return;
                }
                self.said = Some((stage, std::time::Instant::now()));
                let _ = self.app.emit(
                    "bringing",
                    Bringing {
                        stage,
                        done,
                        whole,
                        joining: self.joining,
                    },
                );
            }
            tisty_carrier::Reached::Kept { at, sha256, bytes } => {
                self.kept.push((at, sha256, bytes));
                if self.kept.len() >= KEPT_AT_MOST || self.kept_at.elapsed() >= KEPT_EVERY {
                    self.flush();
                }
            }
        }
    }

    fn flush(&mut self) {
        self.kept_at = std::time::Instant::now();
        if self.kept.is_empty() {
            return;
        }
        let session = self.app.state::<Mutex<Session>>();
        held(&session).fell_behind();
        said_now_held(&session, &std::mem::take(&mut self.kept));
    }
}

fn said_no_longer_held(session: &tauri::State<'_, Mutex<Session>>, let_go: &[String]) {
    if let_go.is_empty() {
        return;
    }
    let said_so = let_go
        .iter()
        .map(|at| tisty_core::Op::AttachLetGo { d: at.clone() })
        .collect();
    if let Err(e) = held(session).commit_all(said_so) {
        witness::warn(
            channel::SYNC,
            "copies were let go of and the log was not told",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
}

pub(crate) async fn catching_up(
    session: &tauri::State<'_, Mutex<Session>>,
    paths: &tisty_core::Paths,
    said: &'static str,
    channel: &'static str,
) -> Answer<()> {
    for _ in 0..2 {
        let at = paths.clone();
        let writes = held(session).writes();
        let fresh = elsewhere(move || session::projected(&at, writes))
            .await?
            .map_err(|e| blamed(channel, said, e))?;
        let mut session = held(session);
        session.adopt(fresh);
        if !session.behind() {
            break;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn settle_in(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
) -> Answer<Settling> {
    let here = env!("CARGO_PKG_VERSION");
    let (was, carriers, paths, home, store, alive, holds) = {
        let session = held(&session);
        let was = session.config.opened_by.clone();
        if was.as_deref() == Some(here) {
            return Ok(Settling {
                ran: false,
                brought: false,
                agrees: true,
                was,
                stuck: None,
            });
        }
        (
            was,
            session.carrier().zip(session.carrier()),
            session.paths.clone(),
            session.here(),
            session.paths.store(),
            session.alive(),
            session.config.holds(),
        )
    };

    let mut brought = false;
    let mut stuck = None;
    let mut arrived = Vec::new();
    let mut carried = carriers.is_none();
    if let Some((carrier, pusher)) = carriers
        && let Some(_done) = alone.inner().claim()
    {
        carried = true;
        let before = tisty_core::cache::fingerprint(&store);
        let pushing = (home.clone(), pusher, alive.clone());
        let mut telling = Telling::new(app.clone(), !carrier.been_here(&home));
        let carried = tauri::async_runtime::spawn_blocking(move || {
            let mut saying = |far| telling.hear(far);
            let done = carrier.carry(
                &home,
                tisty_carrier::Round {
                    way: tisty_carrier::Way::Both,
                    alive: &alive,
                    holds,
                    saying: &mut saying,
                },
            );
            telling.flush();
            done
        })
        .await;
        if let Ok(Ok(done)) = &carried {
            answering(&session, &done.to_answer(), pushing, holds).await;
        }
        brought = tisty_core::cache::fingerprint(&store) != before;
        match carried {
            Ok(Err(why)) => {
                let refusal = said(why);
                witness::warn(
                    channel::SYNC,
                    "the carry on opening did not finish",
                    &[("code", Fact::Code(refusal.code))],
                );
                stuck = Some(refusal);
            }
            Err(_) => witness::warn(channel::SYNC, "the carry on opening never ran", &[]),
            Ok(Ok(done)) => {
                said_no_longer_held(&session, &done.let_go);
                still_asked(&session, &done.undecided);
                arrived = done.arrived;
            }
        }
        let _ = app.emit("brought", ());
    }

    if brought {
        catching_up(
            &session,
            &paths,
            "the store would not project after carrying",
            channel::SYNC,
        )
        .await?;
    }
    let (books, since) = {
        let session = held(&session);
        (session.books_among(&arrived), session.writes())
    };
    let at = paths.clone();
    let read = elsewhere(move || tisty_core::tidy::bodies_of(&at, &books)).await?;
    held(&session).settle_what_came(&read, since);

    let at = paths.clone();
    let audit = elsewhere(move || tisty_core::cache::audit(&at.store(), at.cache()))
        .await?
        .map_err(|e| {
            witness::error(
                channel::CACHE,
                "the cache could not be audited on settling in",
                &[("why", Fact::Why(e.to_string()))],
            );
            match e {
                tisty_core::Error::UnsupportedVersion { .. } => Refusal::of("storeNewer"),
                _ => Refusal::of("internal"),
            }
        })?;
    let agrees = matches!(audit, tisty_core::cache::Audit::Agrees { .. });
    if !agrees {
        let _done = alone.inner().claim();
        let at = paths.clone();
        let writes = {
            let mut session = held(&session);
            if let Some(cache) = session.cache.as_mut() {
                cache.invalidate();
            }
            session.writes()
        };
        let fresh = elsewhere(move || session::projected(&at, writes))
            .await?
            .map_err(|e| {
                blamed(
                    channel::CACHE,
                    "the store would not project without a cache",
                    e,
                )
            })?;
        held(&session).adopt(fresh);
        if held(&session).behind() {
            catching_up(
                &session,
                &paths,
                "the store would not project without a cache",
                channel::CACHE,
            )
            .await?;
        }
    }

    if carried {
        held(&session).keep(|c| c.opened_by = Some(here.to_string()))?;
    }
    Ok(Settling {
        ran: true,
        brought,
        agrees,
        was,
        stuck,
    })
}

#[tauri::command(async)]
pub fn sync_state(session: tauri::State<'_, Mutex<Session>>) -> Answer<Carrying> {
    let (config, paths, mut held, open, archived, lists, let_go) = {
        let session = held(&session);
        let named: Vec<String> = session
            .state
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        (
            session.config.clone(),
            session.paths.clone(),
            named,
            session.state.matching(&Filter::default(), today()).len(),
            session
                .state
                .matching(
                    &Filter {
                        scope: Scope::Archived,
                        ..Default::default()
                    },
                    today(),
                )
                .len(),
            session.state.lists.len(),
            let_go_to(&session),
        )
    };
    held.extend(tisty_core::docs::referenced(&paths.docs()));

    let held_at = tisty_carrier::place_of(config.sync.as_ref());
    let said = held_at.as_deref().map(told);

    Ok(Carrying {
        chosen: held_at.as_deref().map(|at| at.display().to_string()),
        keeper: said.as_ref().map(|one| one.keeper.clone()),
        kept_by: said.and_then(|one| one.named),
        asked: config.sync.is_some(),
        last: config.synced_at.map(|at| at.to_string()),
        heard: config.heard_at.map(|at| at.to_string()),
        loose: tisty_core::attach::loose(paths.data(), &held).files(),
        open,
        archived,
        lists,
        attachments: report::attachments(paths.data()).files,
        weight: report::weighed(paths.data())
            + report::also_weighed(paths.data(), let_go.as_deref()),
        carries: tisty_core::backup::AT_MOST,
        shared_was: config
            .shared_was
            .as_ref()
            .map(|at| at.display().to_string()),
        backed_up_at: config.backed_up_at.map(|at| at.to_string()),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offering {
    key: String,
    named: String,
    at: Option<String>,
    into: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Told {
    keeper: String,
    named: Option<String>,
    into: String,
}

#[tauri::command]
pub fn keepers() -> Vec<Offering> {
    tisty_core::keepers::offers()
        .into_iter()
        .map(|one| Offering {
            key: one.key.to_string(),
            named: one.named.to_string(),
            into: one.at.as_deref().map(|at| room(at).display().to_string()),
            at: one.at.map(|at| at.display().to_string()),
        })
        .collect()
}

#[tauri::command]
pub fn keeper_of(at: String) -> Told {
    told(&std::path::PathBuf::from(at))
}

#[tauri::command(async)]
pub fn strays_at(at: String) -> Strays {
    match tisty_carrier::considering(at).unclaimed() {
        tisty_carrier::Holding::Whole => Strays::default(),
        tisty_carrier::Holding::Strays(adrift) => Strays {
            adrift,
            unreadable: false,
        },
        tisty_carrier::Holding::Unreadable => Strays {
            adrift: 0,
            unreadable: true,
        },
    }
}

#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Strays {
    adrift: usize,
    unreadable: bool,
}

#[tauri::command]
pub fn glimpse_kept(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
) -> Option<glimpse::Glimpse> {
    let cache = held(&session).paths.cache().to_path_buf();
    glimpse::kept(&cache, &at)
}

/// Only ever from a press of somebody's: the card draws itself out of the address until then.
#[tauri::command(async)]
pub async fn glimpse_fetch(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
) -> Answer<Option<glimpse::Glimpse>> {
    let cache = held(&session).paths.cache().to_path_buf();
    let asked = at.clone();
    let found = tauri::async_runtime::spawn_blocking(move || glimpse::fetch(&asked))
        .await
        .map_err(|_| Refusal::of("internal"))?;
    if let Some(one) = &found {
        glimpse::keep(&cache, &at, one);
    }
    Ok(found)
}

#[tauri::command]
pub fn make_room(at: String) -> Answer<()> {
    std::fs::create_dir_all(&at).map_err(|e| Refusal::about("cannotWrite", e.to_string()))
}

pub(crate) fn let_go_to(session: &Session) -> Option<std::path::PathBuf> {
    session
        .dest()
        .filter(|_| session.config.holds() == tisty_core::config::Holds::Shared)
}

/// Only a folder that is still there, and that a round ever finished with, can hold what we let go of.
fn stranded_by_leaving(
    session: &Session,
    chosen: &tisty_core::config::Sync,
) -> Option<std::path::PathBuf> {
    let carrier = session.carrier()?;
    let old = carrier.place()?.to_path_buf();
    (session.config.holds() != tisty_core::config::Holds::Everywhere
        && session.config.sync.as_ref() != Some(chosen)
        && old.is_dir()
        && carrier.been_here(&session.here()))
    .then_some(old)
}

pub(crate) fn went_back_on(session: &Session, at: &std::path::Path) -> bool {
    session.config.restored_at.is_some()
        && tisty_carrier::considering(at)
            .theirs()
            .is_some_and(|theirs| {
                tisty_core::store::peek_identity(session.paths.store())
                    .is_some_and(|ours| ours.trim() == theirs.trim())
            })
}

#[tauri::command]
pub fn choose_sync(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    dest: Option<String>,
) -> Answer<()> {
    let mut session = held(&session);
    let chosen = match dest
        .map(|one| one.trim().to_string())
        .filter(|one| !one.is_empty())
    {
        Some(dest) => {
            let at = std::path::PathBuf::from(&dest);
            let data = session.paths.data();
            let tangled = at.starts_with(data)
                || data.starts_with(&at)
                || at
                    .canonicalize()
                    .ok()
                    .zip(data.canonicalize().ok())
                    .is_some_and(|(a, b)| a.starts_with(&b) || b.starts_with(&a));
            if tangled {
                return Err(Refusal::about("remoteInsideStore", dest));
            }
            tisty_core::config::Sync::Folder(at)
        }
        None => tisty_core::config::Sync::Local,
    };
    if let tisty_core::config::Sync::Folder(at) = &chosen
        && went_back_on(&session, at)
    {
        return Err(Refusal::about("restoredApart", at.display().to_string()));
    }
    if let Some(old) = stranded_by_leaving(&session, &chosen) {
        return Err(Refusal::about(
            "sharedAwayToLeave",
            old.display().to_string(),
        ));
    }
    session.keep(|c| c.sync = Some(chosen))?;
    let _ = app.emit("unstuck", ());
    Ok(())
}

#[tauri::command]
pub async fn sync_now(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    way: Option<String>,
) -> Answer<Settled> {
    let Some(_done) = alone.inner().claim() else {
        return Ok(Settled {
            carried: "busy",
            undecided: Vec::new(),
            unreadable: Vec::new(),
            disowned: Vec::new(),
            unconfirmed: Vec::new(),
            waiting: Vec::new(),
            astray: Vec::new(),
            unprojected: false,
            joined: Vec::new(),
        });
    };
    let was = held(&session).config.sync.clone();
    let done = carried_round(&app, &session, way).await;
    let now = held(&session);
    let _ = app.emit(
        "brought",
        stuck_after(done.as_ref().err(), &was, &now.config.sync),
    );
    drop(now);
    done
}

fn stuck_after(
    why: Option<&Refusal>,
    was: &Option<tisty_core::config::Sync>,
    now: &Option<tisty_core::config::Sync>,
) -> Option<Stuck> {
    why.filter(|one| one.code != "noRemote" && was == now)
        .map(|why| Stuck {
            code: why.code,
            name: why.name.clone(),
        })
}

async fn carried_round(
    app: &tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    way: Option<String>,
) -> Answer<Settled> {
    let (carrier, pusher, paths, home, store, alive, holds) = {
        let session = held(session);
        let carrier = session.carrying()?;
        if let Some(dest) = carrier.place()
            && went_back_on(&session, dest)
        {
            return Err(Refusal::about("restoredApart", dest.display().to_string()));
        }
        (
            carrier,
            session.carrying()?,
            session.paths.clone(),
            session.here(),
            session.paths.store(),
            session.alive(),
            session.config.holds(),
        )
    };

    let before = tisty_core::cache::fingerprint(&store);
    let way = match way.as_deref() {
        Some("push") => tisty_carrier::Way::Push,
        Some("pull") => tisty_carrier::Way::Pull,
        Some("again") => tisty_carrier::Way::Again,
        _ => tisty_carrier::Way::Both,
    };

    let joining = !carrier.been_here(&home);
    let mut telling = Telling::new(app.clone(), joining);
    let pushing = (home.clone(), pusher, alive.clone());
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut saying = |far| telling.hear(far);
        let done = carrier.carry(
            &home,
            tisty_carrier::Round {
                way,
                alive: &alive,
                holds,
                saying: &mut saying,
            },
        );
        telling.flush();
        done
    })
    .await;
    let done = done.map_err(|_| Refusal::of("internal"))?.map_err(said)?;
    answering(session, &done.to_answer(), pushing, holds).await;

    let moved = tisty_core::cache::fingerprint(&store) != before;
    said_no_longer_held(session, &done.let_go);
    still_asked(session, &done.undecided);
    if moved {
        catching_up(
            session,
            &paths,
            "the store would not project after syncing",
            channel::SYNC,
        )
        .await?;
    }
    let (job, was_swept) = {
        let session = held(session);
        let job = session.sweeping(false);
        let was = job.already();
        (job, was)
    };
    let at = paths.clone();
    let walked = elsewhere(move || {
        tisty_core::parcel::swept(at.data());
        tisty_core::attach::swept(at.data());
        job.walk()
    })
    .await?;

    let (books, since) = {
        let session = held(session);
        (session.books_among(&done.arrived), session.writes())
    };
    let at = paths.clone();
    let read = elsewhere(move || tisty_core::tidy::bodies_of(&at, &books)).await?;

    let mut session = held(session);
    session.settle_what_came(&read, since);
    session.swept(&was_swept, walked);
    if let Err(e) = session.take_a_seat() {
        witness::warn(
            channel::SYNC,
            "this machine could not put itself on the list",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
    let unsettled = done.undecided.len()
        + done.unreadable.len()
        + done.disowned.len()
        + done.unconfirmed.len()
        + done.waiting.len()
        + done.astray.len();
    let facts = [
        ("moved", Fact::Word(if moved { "yes" } else { "no" })),
        ("sent", Fact::Count(done.sent)),
        ("brought", Fact::Count(done.brought)),
        ("arrived", Fact::Count(done.arrived.len())),
        ("undecided", Fact::Count(done.undecided.len())),
        ("unreadable", Fact::Count(done.unreadable.len())),
        ("disowned", Fact::Count(done.disowned.len())),
        ("unconfirmed", Fact::Count(done.unconfirmed.len())),
        ("waiting", Fact::Count(done.waiting.len())),
        ("coming", Fact::Count(done.coming.len())),
        ("astray", Fact::Count(done.astray.len())),
        ("joined", Fact::Count(done.joined.len())),
    ];
    let carried = done.sent + done.brought + done.arrived.len() + done.joined.len();
    if unsettled > 0 {
        witness::warn(
            channel::SYNC,
            "a carry finished, and left work behind",
            &facts,
        );
    } else if carried > 0 || moved {
        witness::note(channel::SYNC, "a carry finished", &facts);
    }
    for one in done
        .unreadable
        .iter()
        .chain(done.disowned.iter())
        .chain(done.astray.iter())
    {
        witness::warn(
            channel::SYNC,
            "a carry could not settle a document",
            &[("at", Fact::Id(one.clone()))],
        );
    }
    let heard = done.brought > 0;
    session.keep(|c| {
        c.synced_at = Some(jiff::Timestamp::now());
        if heard {
            c.heard_at = c.synced_at;
        }
    })?;
    Ok(Settled {
        carried: match (done.sent > 0, moved || done.brought > 0) {
            (true, true) => "both",
            (true, false) => "sent",
            (false, true) => "came",
            (false, false) => "same",
        },
        undecided: done.undecided.into_iter().map(|one| one.id).collect(),
        unreadable: done.unreadable,
        disowned: done.disowned,
        unconfirmed: done.unconfirmed,
        waiting: done.waiting,
        astray: done.astray,
        unprojected: done.unprojected,
        joined: done.joined,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settled {
    carried: &'static str,
    undecided: Vec<String>,
    unreadable: Vec<String>,
    disowned: Vec<String>,
    unconfirmed: Vec<String>,
    waiting: Vec<String>,
    astray: Vec<String>,
    unprojected: bool,
    joined: Vec<String>,
}

#[tauri::command]
pub async fn back_up(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    let (data, aside, also) = {
        let session = held(&session);
        (
            session.paths.data().to_path_buf(),
            session.paths.cache().to_path_buf(),
            let_go_to(&session),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::write(&data, &at, &aside, also.as_deref())
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "the backup could not be written",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::UnsupportedVersion { .. } => Refusal::of("storeNewer"),
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            _ => Refusal::about("cannotWrite", into),
        }
    })?;

    let now = jiff::Timestamp::now();
    held(&session).keep(|config| config.backed_up_at = Some(now))?;
    Ok(made.bytes)
}

#[tauri::command(async)]
pub fn retire_attachment(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<()> {
    held(&session).retire(&[reference]).map(|_| ())
}

#[tauri::command(async)]
pub fn retire_attachments(
    session: tauri::State<'_, Mutex<Session>>,
    references: Vec<String>,
) -> Answer<usize> {
    held(&session).retire(&references)
}

#[tauri::command]
pub fn remove_machine(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let mut session = held(&session);
    let who = tisty_core::event::DeviceId(id.clone());
    if who == session.config.device_id {
        return Err(Refusal::of("notThisMachine"));
    }

    session
        .commit(Op::DeviceRemove { d: who })
        .map_err(|e| blamed(channel::STORE, "the machine could not be removed", e))?;
    session.reproject().map_err(|e| {
        blamed(
            channel::CACHE,
            "the store would not project after removing",
            e,
        )
    })?;

    witness::note(
        channel::SYNC,
        "a machine was removed and what it wrote was left where everyone can still read it",
        &[("at", Fact::Id(id))],
    );
    Ok(())
}

/// The key travels back from the window so a claim that changed between the reading and the
/// answering is refused instead of confirmed.
#[tauri::command]
pub fn confirm_machine_key(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    key: String,
) -> Answer<()> {
    let session = held(&session);
    let who = tisty_core::event::DeviceId(id.clone());
    let says = session.state.keys.get(&who).cloned().or_else(|| {
        let at = session.place()?;
        tisty_core::store::key_said_in(&at.join(tisty_carrier::STORE).join(&id), &who)
    });
    if says.as_deref() != Some(key.as_str()) {
        return Err(Refusal::of("keyMoved"));
    }
    if !tisty_core::vouched::confirm(session.paths.data(), &who, &key) {
        return Err(Refusal::of("keyNotConfirmed"));
    }

    tisty_carrier::turned::let_through(session.paths.data(), &id);

    witness::note(
        channel::SYNC,
        "somebody answered for the key a machine signs with",
        &[("at", Fact::Id(id))],
    );
    Ok(())
}

#[tauri::command]
pub async fn restore(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    from: String,
) -> Answer<usize> {
    let _done = alone.inner().taken()?;
    let paths = { held(&session).paths.clone() };

    let at = std::path::PathBuf::from(&from);
    let done = tauri::async_runtime::spawn_blocking(move || tisty_core::backup::read(&paths, &at))
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map_err(|e| match e {
            tisty_core::Error::OtherStore { theirs } => Refusal::about("otherStore", theirs),
            tisty_core::Error::Io(why) => Refusal::about("restoreFailed", why.to_string()),
            tisty_core::Error::UnsupportedVersion { .. } => Refusal::of("storeNewer"),
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            _ => Refusal::about("cannotRead", from.clone()),
        })?;

    let fresh = elsewhere(Session::open).await?.map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after a restore",
            e,
        )
    })?;
    *held(&session) = fresh;
    Ok(done.files)
}

#[tauri::command]
pub fn roomy() -> u64 {
    tisty_core::docs::BODY_ROOMY
}

#[tauri::command]
pub fn revealed(session: tauri::State<'_, Mutex<Session>>, path: String) -> Answer<()> {
    let at = std::path::Path::new(&path);

    let plain = at.components().next().is_none_or(|first| {
        !matches!(first, std::path::Component::Prefix(at) if !matches!(at.kind(), std::path::Prefix::Disk(_)))
    });
    if !plain || !at.is_absolute() {
        return Err(Refusal::about("cannotOpen", path));
    }

    let (data, config, shared) = {
        let session = held(&session);
        (
            session.paths.data().to_path_buf(),
            session.paths.config().to_path_buf(),
            session.place(),
        )
    };
    let ours: Vec<std::path::PathBuf> = [Some(data), Some(config), shared]
        .into_iter()
        .flatten()
        .collect();
    if !within(at, &ours) {
        return Err(Refusal::about("cannotOpen", path));
    }
    show(at, &path)
}

fn told(at: &std::path::Path) -> Told {
    let into = room(at).display().to_string();
    match tisty_core::keepers::keeper(at) {
        tisty_core::keepers::Keeper::Cloud(key) => Told {
            keeper: "cloud".into(),
            named: tisty_core::keepers::offers()
                .into_iter()
                .find(|one| one.key == key)
                .map(|one| one.named.to_string()),
            into,
        },
        tisty_core::keepers::Keeper::Away => Told {
            keeper: "away".into(),
            named: None,
            into,
        },
        tisty_core::keepers::Keeper::Plain => Told {
            keeper: "plain".into(),
            named: None,
            into,
        },
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Freeing {
    gone: usize,
    freed: u64,
    done: bool,
}

#[derive(Default)]
pub(crate) struct Stopping(std::sync::atomic::AtomicBool);

/// Turning it on is the only change that moves anything, so it is asked for rather than done on
/// the way past: it can take an afternoon, and somebody may want it to stop.
#[tauri::command(async)]
pub async fn free_up(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    stopping: tauri::State<'_, Stopping>,
) -> Answer<Freeing> {
    let _done = alone.inner().taken()?;
    stopping
        .0
        .store(false, std::sync::atomic::Ordering::Relaxed);
    let (carrier, home, above, held_away) = {
        let session = held(&session);
        let carrier = session.carrying()?;
        let mine = session.store.device().clone();
        let held_away: std::collections::BTreeSet<String> = session
            .state
            .holders
            .iter()
            .filter(|(_, who)| who.iter().any(|one| one != &mine))
            .map(|(at, _)| at.clone())
            .collect();
        (
            carrier,
            session.here(),
            session.config.only_shared_above(),
            held_away,
        )
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut said = 0;
        let elsewhere = |at: &str| held_away.contains(at);
        carrier.let_go(&home, above, &elsewhere, &mut |far| {
            if far.gone != said {
                said = far.gone;
                let _ = telling.emit(
                    "freeing",
                    Freeing {
                        gone: far.gone,
                        freed: far.freed,
                        done: false,
                    },
                );
            }
            !telling
                .state::<Stopping>()
                .0
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    said_no_longer_held(&session, &done.let_go);
    witness::note(
        channel::SYNC,
        "big attachments were left to the shared folder",
        &[
            ("count", Fact::Count(done.gone)),
            ("bytes", Fact::Bytes(done.freed)),
        ],
    );
    let now = Freeing {
        gone: done.gone,
        freed: done.freed,
        done: true,
    };
    let _ = app.emit("freeing", now.clone());
    Ok(now)
}

#[tauri::command]
pub fn stop_freeing(stopping: tauri::State<'_, Stopping>) {
    stopping.0.store(true, std::sync::atomic::Ordering::Relaxed);
}

type Pushing = (
    tisty_carrier::Here,
    Box<dyn tisty_carrier::Carrier>,
    Vec<String>,
);

async fn answering(
    session: &tauri::State<'_, Mutex<Session>>,
    files: &[String],
    (home, pusher, alive): Pushing,
    holds: tisty_core::config::Holds,
) {
    if files.is_empty() {
        return;
    }
    let wrote = {
        let mut session = held(session);
        let docs = session.paths.docs();
        files
            .iter()
            .filter(|file| {
                tisty_core::docs::read(&docs, file)
                    .is_ok_and(|body| session.retell(file, &body, None))
            })
            .count()
    };
    if wrote == 0 {
        return;
    }
    let pushed = tauri::async_runtime::spawn_blocking(move || {
        let mut quiet = |_: tisty_carrier::Reached| {};
        pusher.carry(
            &home,
            tisty_carrier::Round {
                way: tisty_carrier::Way::Push,
                alive: &alive,
                holds,
                saying: &mut quiet,
            },
        )
    })
    .await;
    if !matches!(pushed, Ok(Ok(_))) {
        witness::warn(
            channel::SYNC,
            "a document was written down but not handed on yet",
            &[],
        );
    }
}

fn still_asked(session: &tauri::State<'_, Mutex<Session>>, undecided: &[tisty_carrier::Undecided]) {
    held(session).asked.extend(
        undecided
            .iter()
            .map(|one| (one.id.clone(), one.theirs.clone())),
    );
}

#[cfg(test)]
#[path = "storing_test.rs"]
mod tests;
