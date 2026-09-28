use std::sync::Mutex;

use tauri::{Emitter, Manager};
use tisty_core::Op;
use tisty_core::witness::{self, Fact, channel};

use crate::{
    Answer, Carrying, Filter, OneAtATime, Refusal, Scope, Session, Settling, blamed, elsewhere,
    glimpse, held, report, room, said, session, show, today, within,
};

#[tauri::command]
pub async fn settle_in(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
) -> Answer<Settling> {
    let here = env!("CARGO_PKG_VERSION");
    let (was, dest, paths, data, store, aside, device, alive, holds) = {
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
        let dest = match &session.config.sync {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
            _ => None,
        };
        (
            was,
            dest,
            session.paths.clone(),
            session.paths.data().to_path_buf(),
            session.paths.store(),
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
            session.alive(),
            session.config.holds(),
        )
    };

    let mut brought = false;
    let mut stuck = None;
    let mut arrived = Vec::new();
    let mut carried = dest.is_none();
    let _done = alone.inner().claim();
    if let Some(dest) = dest
        && _done.is_some()
    {
        carried = true;
        let before = tisty_core::cache::fingerprint(&store);
        let carried = tauri::async_runtime::spawn_blocking(move || {
            tisty_sync::carry_holding(
                &data,
                Some(&aside),
                &device,
                &dest,
                tisty_sync::Way::Both,
                &alive,
                holds,
            )
        })
        .await;
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
            Ok(Ok(done)) => arrived = done.arrived,
        }
        brought = tisty_core::cache::fingerprint(&store) != before;
    }

    if brought {
        let at = paths.clone();
        let writes = held(&session).writes();
        let fresh = elsewhere(move || session::projected(&at, writes))
            .await?
            .map_err(|e| {
                blamed(
                    channel::SYNC,
                    "the store would not project after carrying",
                    e,
                )
            })?;
        held(&session).adopt(fresh);
    }
    let books = held(&session).books_among(&arrived);
    let at = paths.clone();
    let read = elsewhere(move || tisty_core::tidy::bodies_of(&at, &books)).await?;
    held(&session).settle_what_came(&read);

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
                tisty_core::Error::UnsupportedVersion(_) => Refusal::of("storeNewer"),
                _ => Refusal::of("internal"),
            }
        })?;
    let agrees = matches!(audit, tisty_core::cache::Audit::Agrees { .. });
    if !agrees {
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

#[tauri::command]
pub fn sync_state(session: tauri::State<'_, Mutex<Session>>) -> Answer<Carrying> {
    let session = held(&session);
    let config = &session.config;

    let mut held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    held.extend(tisty_core::docs::referenced(&session.paths.docs()));

    let held_at = match &config.sync {
        Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
        _ => None,
    };
    let said = held_at.as_deref().map(told);

    Ok(Carrying {
        chosen: held_at.as_deref().map(|at| at.display().to_string()),
        keeper: said.as_ref().map(|one| one.keeper.clone()),
        kept_by: said.and_then(|one| one.named),
        asked: config.sync.is_some(),
        last: config.synced_at.map(|at| at.to_string()),
        heard: config.heard_at.map(|at| at.to_string()),
        loose: tisty_core::attach::loose(session.paths.data(), &held).files(),
        open: session.state.matching(&Filter::default(), today()).len(),
        archived: session
            .state
            .matching(
                &Filter {
                    scope: Scope::Archived,
                    ..Default::default()
                },
                today(),
            )
            .len(),
        lists: session.state.lists.len(),
        attachments: report::attachments(session.paths.data()).files,
        weight: report::weighed(session.paths.data())
            + report::also_weighed(session.paths.data(), session.dest().as_deref()),
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
    match tisty_sync::unclaimed(&std::path::PathBuf::from(at)) {
        tisty_sync::Holding::Whole => Strays::default(),
        tisty_sync::Holding::Strays(adrift) => Strays {
            adrift,
            unreadable: false,
        },
        tisty_sync::Holding::Unreadable => Strays {
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

#[tauri::command]
pub fn choose_sync(session: tauri::State<'_, Mutex<Session>>, dest: Option<String>) -> Answer<()> {
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
    // Leaving a folder that holds what this machine let go of takes them with it — but only while
    // it is there to bring them back from. Gone, refusing would trap somebody with nowhere to go.
    if session.config.holds() != tisty_core::config::Holds::Everywhere
        && session.config.sync != Some(chosen.clone())
        && let Some(tisty_core::config::Sync::Folder(old)) = session.config.sync.clone()
        && old.is_dir()
    {
        return Err(Refusal::about(
            "sharedAwayToLeave",
            old.display().to_string(),
        ));
    }
    session.keep(|c| c.sync = Some(chosen))
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
            astray: Vec::new(),
            joined: Vec::new(),
        });
    };

    let (dest, paths, data, store, aside, device, alive, holds) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            dest,
            session.paths.clone(),
            session.paths.data().to_path_buf(),
            session.paths.store(),
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
            session.alive(),
            session.config.holds(),
        )
    };

    let before = tisty_core::cache::fingerprint(&store);
    let way = match way.as_deref() {
        Some("push") => tisty_sync::Way::Push,
        Some("pull") => tisty_sync::Way::Pull,
        Some("again") => tisty_sync::Way::Again,
        _ => tisty_sync::Way::Both,
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        tisty_sync::carry_telling(
            &data,
            Some(&aside),
            &device,
            &dest,
            way,
            &alive,
            holds,
            &mut |far| {
                let _ = telling.emit(
                    "carried",
                    match far {
                        tisty_sync::Reached::Log => "log",
                        tisty_sync::Reached::Papers => "papers",
                    },
                );
            },
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    let moved = tisty_core::cache::fingerprint(&store) != before;
    if moved {
        let at = paths.clone();
        let writes = held(&session).writes();
        let fresh = elsewhere(move || session::projected(&at, writes))
            .await?
            .map_err(|e| {
                blamed(
                    channel::SYNC,
                    "the store would not project after syncing",
                    e,
                )
            })?;
        held(&session).adopt(fresh);
    }
    let (job, was_swept) = {
        let session = held(&session);
        let job = session.sweeping(false);
        let was = job.already();
        (job, was)
    };
    let at = paths.clone();
    let (_, swept) = elsewhere(move || {
        tisty_core::parcel::swept(at.data());
        tisty_core::attach::swept(at.data());
        job.run()
    })
    .await?;

    let books = held(&session).books_among(&done.arrived);
    let at = paths.clone();
    let read = elsewhere(move || tisty_core::tidy::bodies_of(&at, &books)).await?;

    let mut session = held(&session);
    session.settle_what_came(&read);
    session.swept(&was_swept, &swept);
    if let Err(e) = session.take_a_seat() {
        witness::warn(
            channel::SYNC,
            "this machine could not put itself on the list",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
    let unsettled = done.undecided.len() + done.unreadable.len() + done.astray.len();
    let facts = [
        ("moved", Fact::Word(if moved { "yes" } else { "no" })),
        ("sent", Fact::Count(done.sent)),
        ("brought", Fact::Count(done.brought)),
        ("arrived", Fact::Count(done.arrived.len())),
        ("undecided", Fact::Count(done.undecided.len())),
        ("unreadable", Fact::Count(done.unreadable.len())),
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
    for one in done.unreadable.iter().chain(done.astray.iter()) {
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
        astray: done.astray,
        joined: done.joined,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settled {
    carried: &'static str,
    undecided: Vec<String>,
    unreadable: Vec<String>,
    astray: Vec<String>,
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
            session.dest(),
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
            tisty_core::Error::UnsupportedVersion(_) => Refusal::of("storeNewer"),
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            _ => Refusal::about("cannotWrite", into),
        }
    })?;

    let now = jiff::Timestamp::now();
    held(&session).keep(|config| config.backed_up_at = Some(now))?;
    Ok(made.bytes)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Torn {
    rifts: Vec<tisty_core::merge::Rift>,
    print: String,
}

fn three_bodies(session: &Session, id: &str) -> Answer<Option<(String, String, String)>> {
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    let Some(base) = tisty_core::docs::read_carried(session.paths.data(), id) else {
        return Ok(None);
    };
    match tisty_sync::both_papers(session.paths.data(), &dest, id) {
        Ok((mine, theirs)) => Ok(Some((base, mine, theirs))),
        Err(_) => Ok(None),
    }
}

fn print_of_three(base: &str, mine: &str, theirs: &str) -> String {
    tisty_core::attach::printed(format!("{base}\u{0}{mine}\u{0}{theirs}").as_bytes())
}

#[tauri::command(async)]
pub fn paper_rifts(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Torn> {
    let session = held(&session);
    if session.state.shut_tight(&id) {
        return Err(Refusal::of(match session.state.away(&id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Ok(Torn {
            rifts: Vec::new(),
            print: String::new(),
        });
    };
    Ok(Torn {
        print: print_of_three(&base, &mine, &theirs),
        rifts: tisty_core::merge::rifts(&base, &mine, &theirs),
    })
}

#[tauri::command(async)]
pub fn weave_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    picks: Vec<String>,
    print: String,
) -> Answer<()> {
    let mut session = held(&session);
    // Settling what two machines already wrote is not writing into it, so being archived is no
    // reason to leave the rift with no way out. Only a lock guards the text itself.
    if session.state.shut_tight(&id) {
        return Err(Refusal::of("documentLocked"));
    }
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Err(Refusal::of("noBase"));
    };
    if print_of_three(&base, &mine, &theirs) != print {
        return Err(Refusal::of("movedUnderfoot"));
    }
    let picked: Vec<tisty_core::merge::Pick> = picks
        .iter()
        .map(|one| match one.as_str() {
            "mine" => tisty_core::merge::Pick::Mine,
            "theirs" => tisty_core::merge::Pick::Theirs,
            _ => tisty_core::merge::Pick::Both,
        })
        .collect();
    let whole = tisty_core::merge::woven_with(&base, &mine, &theirs, &picked)
        .ok_or_else(|| Refusal::of("cannotWeave"))?;

    let papers = session.paths.docs();
    tisty_core::docs::kept_before(session.paths.data(), &id, &mine, &whole)
        .map_err(|e| blamed(channel::SYNC, "what it was could not be kept", e))?;
    tisty_core::docs::write(&papers, &id, &whole).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        e => blamed(channel::SYNC, "the woven body could not be written", e),
    })?;
    session.mind(&id);
    Ok(())
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

#[tauri::command]
pub async fn join_them(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (paths, aside, also) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.paths.cache().to_path_buf(),
            session.dest(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::reset(&paths, &at, &aside, also.as_deref())
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "nothing was reset because the backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::about("cannotWrite", into)
    })?;

    let fresh = elsewhere(Session::open).await?.map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after being reset",
            e,
        )
    })?;
    *held(&session) = fresh;
    Ok(made.bytes)
}

#[tauri::command]
pub async fn take_over(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (dest, aside, ours) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        let ours = tisty_core::store::identity(session.paths.store())
            .map_err(|e| blamed(channel::SYNC, "this machine has no name of its own", e))?;
        (dest, session.paths.cache().to_path_buf(), ours)
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::take_over(&dest, &ours, &at, &aside)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "the folder was left alone because its backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::about("cannotWrite", into)
    })?;

    Ok(made.bytes)
}

fn kinned(store: &std::path::Path, dest: &std::path::Path) -> &'static str {
    match tisty_sync::kinship(store, dest) {
        tisty_sync::Kin::SameLineage => "sameLineage",
        tisty_sync::Kin::Clash(_) => "clash",
        tisty_sync::Kin::Unsure(_) => "unsure",
        tisty_sync::Kin::Strangers => "strangers",
    }
}

/// The folder is walked with the session let go of: a cloud folder can take its time, and every
/// other command waits behind whoever holds it.
#[tauri::command(async)]
pub fn folder_astir(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let dest = {
        let session = held(&session);
        match session.config.sync.clone() {
            Some(tisty_core::config::Sync::Folder(dest)) => dest,
            _ => return Err(Refusal::of("noRemote")),
        }
    };
    Ok(tisty_sync::stirring(&dest).to_string())
}

#[tauri::command(async)]
pub fn sync_kin(session: tauri::State<'_, Mutex<Session>>) -> Answer<&'static str> {
    let session = held(&session);
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    Ok(kinned(&session.paths.store(), &dest))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joining {
    kin: &'static str,
    fresh: bool,
    holds: bool,
    alias: Option<String>,
}

#[tauri::command(async)]
pub fn joining(session: tauri::State<'_, Mutex<Session>>) -> Answer<Joining> {
    let session = held(&session);
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    let store = session.paths.store();
    Ok(Joining {
        kin: kinned(&store, &dest),
        fresh: !tisty_core::store::inhabited(&store),
        holds: tisty_core::store::inhabited(dest.join(tisty_sync::STORE)),
        alias: tisty_sync::signed_at(&dest),
    })
}

#[tauri::command]
pub async fn merge_stores(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<bool> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (data, dest, aside, device) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            session.paths.data().to_path_buf(),
            dest,
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let seam = tauri::async_runtime::spawn_blocking(move || -> Answer<tisty_sync::Stitched> {
        tisty_core::backup::write(&data, &at, &aside, Some(&dest)).map_err(|e| {
            witness::error(
                channel::BACKUP,
                "nothing was joined because the backup did not land",
                &[("why", Fact::Why(e.to_string()))],
            );
            match e {
                tisty_core::Error::TooBig => Refusal::of("tooBig"),
                _ => Refusal::about("cannotWrite", into),
            }
        })?;
        tisty_sync::stitch(&data, &device, &dest).map_err(|trouble| {
            let refusal = said(trouble);
            witness::warn(
                channel::SYNC,
                "the two histories were left apart",
                &[("code", Fact::Code(refusal.code))],
            );
            refusal
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))??;

    let fresh = elsewhere(Session::open).await?.map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after joining",
            e,
        )
    })?;
    *held(&session) = fresh;
    Ok(seam.stitch.is_some())
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
            tisty_core::Error::UnsupportedVersion(_) => Refusal::of("storeNewer"),
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
            match &session.config.sync {
                Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
                _ => None,
            },
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
    let (data, dest, above) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            session.paths.data().to_path_buf(),
            dest,
            session.config.only_shared_above(),
        )
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut said = 0;
        tisty_sync::let_go_telling(&data, &dest, above, &mut |far| {
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
