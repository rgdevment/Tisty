use std::collections::BTreeSet;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Serialize;
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Op, Paths, State, person};
use tisty_share_proto::phrase::{self, Words};
use tisty_share_proto::seed::Seed;
use zeroize::Zeroizing;

use crate::{Answer, Refusal, Session, blamed, elsewhere, held};

const SEED: &str = "seed";

/// Named after the store and not its folder, so moving it keeps the seed and a sandbox never meets it.
fn scope(paths: &Paths) -> Result<String, String> {
    tisty_core::store::peek_identity(paths.store())
        .map(|store| format!("identity-{store}"))
        .ok_or_else(|| "this store has no name yet".to_string())
}

// Joining stores renames this one, so a seed kept under a store it took in is brought under its name now.
pub fn kept(paths: &Paths, forebears: &BTreeSet<String>) -> Result<Option<Seed>, String> {
    let Ok(here) = scope(paths) else {
        return Ok(None);
    };
    if let Some(seed) = kept_at(&here)? {
        return Ok(Some(seed));
    }
    for was in forebears
        .iter()
        .map(|store| format!("identity-{store}"))
        .filter(|was| *was != here)
    {
        if let Some(seed) = kept_at(&was)? {
            if let Err(why) = keep_at(&here, &seed) {
                witness::warn(
                    channel::STORE,
                    "the identity's seed was found under a store this one took in, and could not be kept under its name",
                    &[("why", Fact::Why(why))],
                );
            }
            return Ok(Some(seed));
        }
    }
    Ok(None)
}

fn kept_at(scope: &str) -> Result<Option<Seed>, String> {
    let secrets = crate::reaching::seed_secrets(scope).map_err(|e| format!("{e:?}"))?;
    let Some(said) = secrets.read(SEED).map_err(|e| format!("{e:?}"))? else {
        return Ok(None);
    };
    let said = Zeroizing::new(said);
    let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(said.as_bytes()).unwrap_or_default());
    let mut seed = Zeroizing::new([0u8; 32]);
    if bytes.len() != seed.len() {
        witness::warn(
            channel::STORE,
            "what was kept as the identity's seed is not one, so it counts as none",
            &[],
        );
        return Ok(None);
    }
    seed.copy_from_slice(&bytes);
    Ok(Some(Seed::from_bytes(&seed)))
}

fn keep(paths: &Paths, seed: &Seed) -> Answer<()> {
    let refused = |why: String| {
        witness::error(
            channel::STORE,
            "the identity's seed could not be kept on this computer",
            &[("why", Fact::Why(why))],
        );
        Refusal::of("identityNotKept")
    };
    let scope = scope(paths).map_err(refused)?;
    keep_at(&scope, seed).map_err(refused)
}

fn keep_at(scope: &str, seed: &Seed) -> Result<(), String> {
    let secrets = crate::reaching::seed_secrets(scope).map_err(|e| format!("{e:?}"))?;
    let said = Zeroizing::new(URL_SAFE_NO_PAD.encode(seed.to_bytes().as_ref()));
    secrets.write(SEED, &said).map_err(|e| format!("{e:?}"))
}

fn unread(why: String) -> Refusal {
    witness::warn(
        channel::STORE,
        "the identity's seed could not be read on this computer",
        &[("why", Fact::Why(why))],
    );
    Refusal::of("identityUnread")
}

fn ours(seed: &Seed, state: &State) -> bool {
    state.persons.names(&seed.root().person())
}

async fn off_the_lock(
    session: &tauri::State<'_, Mutex<Session>>,
    gate: impl FnOnce(&Session) -> bool,
) -> Option<Seed> {
    let (paths, forebears, wanted) = {
        let session = held(session);
        (
            session.paths.clone(),
            session.state.forebears.clone(),
            gate(&session),
        )
    };
    if !wanted {
        return None;
    }
    match elsewhere(move || kept(&paths, &forebears)).await {
        Ok(Ok(seed)) => seed,
        Ok(Err(why)) => {
            unread(why);
            None
        }
        Err(_) => None,
    }
}

pub async fn for_a_round(session: &tauri::State<'_, Mutex<Session>>) -> Option<Seed> {
    off_the_lock(session, |session| {
        let state = &session.state;
        state.persons.names_a_person()
            && state.devices.contains(&session.config.device_id)
            && state.persons.awaits_the_seed(&state.devices)
    })
    .await
}

/// Read only when the sealed cloud the log names is not the one this machine already reads, so the keychain is seldom asked.
pub async fn seal_the_cloud(session: &tauri::State<'_, Mutex<Session>>) {
    let (paths, forebears, wanted) = {
        let session = held(session);
        let wanted = session
            .state
            .sealed_cloud()
            .map(|(cloud, key)| (cloud.to_string(), key.to_string()));
        let cloud = wanted.as_ref().map(|(cloud, _)| cloud.clone());
        if cloud == session.reach.sealed() || (cloud.is_some() && cloud == session.reach.tried()) {
            return;
        }
        (
            session.paths.clone(),
            session.state.forebears.clone(),
            wanted,
        )
    };
    let Some((cloud, named)) = wanted else {
        held(session).reach.seal_with(None);
        return;
    };
    let seed = match elsewhere(move || kept(&paths, &forebears)).await {
        Ok(Ok(seed)) => seed,
        Ok(Err(why)) => {
            unread(why);
            None
        }
        Err(_) => None,
    };
    let key = seed
        .map(|seed| tisty_share_proto::cloud::CloudKey::of(&seed))
        .filter(|key| key.named_for(&cloud) == named);
    let session = held(session);
    session
        .reach
        .tried_for(key.is_none().then(|| cloud.clone()));
    if key.is_none() {
        witness::note(
            channel::CLOUD,
            "the cloud is sealed with a phrase this machine does not hold, so it is read as it stands",
            &[],
        );
    }
    session
        .reach
        .seal_with(key.map(|key| crate::reaching::Sealing { cloud, key }));
}

pub async fn for_removing(session: &tauri::State<'_, Mutex<Session>>) -> Option<Seed> {
    off_the_lock(session, |session| session.state.persons.names_a_person()).await
}

pub fn after_a_round(session: &mut Session, seed: Option<&Seed>) {
    let device = session.config.device_id.clone();
    if !session.state.persons.names_a_person() || !session.state.devices.contains(&device) {
        return;
    }
    let mine = person::introduced(&session.paths, &device, &session.state.persons);
    if let Some(op) = mine
        && let Err(e) = session.commit(op)
    {
        said_nothing(&e);
        return;
    }
    let Some(seed) = seed.filter(|seed| ours(seed, &session.state)) else {
        return;
    };
    let machines = person::machines(&session.state);
    let now = jiff::Timestamp::now();
    let mut ops = person::certified(seed, &session.state.persons, &machines, now);
    ops.extend(person::left(
        seed,
        &session.state.persons,
        &session.state.devices,
        now,
    ));
    if !ops.is_empty()
        && let Err(e) = session.commit_all(ops)
    {
        said_nothing(&e);
    }
}

fn said_nothing(e: &tisty_core::Error) {
    witness::warn(
        channel::STORE,
        "this machine could not say its share key or vouch for the others",
        &[("why", Fact::Why(e.to_string()))],
    );
}

pub fn removing(session: &Session, device: DeviceId, seed: Option<&Seed>) -> Vec<Op> {
    let mut ops = match seed.filter(|seed| ours(seed, &session.state)) {
        Some(seed) => person::revoked(
            seed,
            &session.state.persons,
            &device,
            jiff::Timestamp::now(),
        ),
        None => Vec::new(),
    };
    ops.push(Op::DeviceRemove { d: device });
    ops
}

#[derive(Serialize)]
pub struct Shown {
    pub held: bool,
    pub elsewhere: bool,
    pub fingerprint: Option<String>,
    pub suffix: Option<String>,
}

fn words(session: &Session) -> Words {
    match crate::refusing::spanish(session.locale.as_deref()) {
        true => Words::Spanish,
        false => Words::English,
    }
}

fn current(session: &Session) -> Answer<Option<Seed>> {
    Ok(counted(
        kept(&session.paths, &session.state.forebears).map_err(unread)?,
        &session.state,
    ))
}

/// While the log names nobody, a seed kept here is one that began and never reached it.
fn counted(seed: Option<Seed>, state: &State) -> Option<Seed> {
    let named = state.persons.names_a_person();
    seed.filter(|seed| !named || ours(seed, state))
}

#[tauri::command(async)]
pub fn identity_shown(session: tauri::State<'_, Mutex<Session>>) -> Answer<Shown> {
    let (paths, forebears) = {
        let session = held(&session);
        (session.paths.clone(), session.state.forebears.clone())
    };
    let seed = kept(&paths, &forebears).map_err(unread)?;
    let session = held(&session);
    let person = person::person_of(&session.state, &session.config.device_id);
    let held = seed.as_ref().is_some_and(|seed| ours(seed, &session.state));
    Ok(Shown {
        held,
        elsewhere: session.state.persons.names_a_person() && !held,
        fingerprint: person.map(|one| one.fingerprint()),
        suffix: person.map(|one| one.suffix()),
    })
}

// Held whole under the session lock: two quick asks, or a round in between, must never make two roots.
#[tauri::command(async)]
pub fn identity_begin(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let mut session = held(&session);
    if session.state.persons.names_a_person() {
        return Err(Refusal::of(match current(&session)? {
            Some(_) => "identityExists",
            None => "identityElsewhere",
        }));
    }
    let seed = match current(&session)? {
        Some(unfinished) => unfinished,
        None => {
            let fresh = Seed::minted().ok_or_else(|| Refusal::of("identityNotKept"))?;
            keep(&session.paths, &fresh)?;
            fresh
        }
    };
    let shown =
        phrase::phrase(&seed, words(&session)).ok_or_else(|| Refusal::of("identityNotKept"))?;
    begun(&mut session, &seed)?;
    Ok(shown.to_string())
}

#[tauri::command(async)]
pub fn identity_phrase(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let (paths, forebears) = {
        let session = held(&session);
        (session.paths.clone(), session.state.forebears.clone())
    };
    let seed = kept(&paths, &forebears).map_err(unread)?;
    let session = held(&session);
    let seed = counted(seed, &session.state).ok_or_else(|| Refusal::of("identityNone"))?;
    phrase::phrase(&seed, words(&session))
        .map(|said| said.to_string())
        .ok_or_else(|| Refusal::of("identityNone"))
}

#[tauri::command(async)]
pub fn identity_recover(session: tauri::State<'_, Mutex<Session>>, said: String) -> Answer<()> {
    let said = Zeroizing::new(said);
    let seed = phrase::seed_from(&said).map_err(|unread| match unread {
        phrase::Unread::Count(count) => Refusal::about("phraseCount", count.to_string()),
        phrase::Unread::Word(at) => Refusal::about("phraseWord", at.to_string()),
        phrase::Unread::Checksum => Refusal::of("phraseChecksum"),
    })?;
    let mut session = held(&session);
    if session.state.persons.names_a_person() && !ours(&seed, &session.state) {
        return Err(Refusal::of("phraseOtherPerson"));
    }
    let here = kept(&session.paths, &session.state.forebears).map_err(unread)?;
    match here {
        Some(here) if *here.to_bytes() == *seed.to_bytes() => {}
        Some(here) if ours(&here, &session.state) => return Err(Refusal::of("identityExists")),
        _ => keep(&session.paths, &seed)?,
    }
    begun(&mut session, &seed)
}

fn begun(session: &mut Session, seed: &Seed) -> Answer<()> {
    let ops = person::begun(
        &session.paths,
        &session.config.device_id,
        seed,
        &session.state.persons,
        jiff::Timestamp::now(),
    )
    .ok_or_else(|| Refusal::of("identityNotKept"))?;
    if ops.is_empty() {
        return Ok(());
    }
    session
        .commit_all(ops)
        .map_err(|e| blamed(channel::STORE, "the identity could not be written", e))
}
