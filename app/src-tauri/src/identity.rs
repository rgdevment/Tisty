use std::collections::BTreeSet;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Serialize;
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Op, Paths, State, person};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::phrase::{self, Words};
use tisty_share_proto::seed::Seed;
use zeroize::Zeroizing;

use crate::{Answer, Refusal, Session, blamed, elsewhere, held};

const SEED: &str = "seed";
const CHECKED: usize = 3;
const THIRD: usize = phrase::WORDS / CHECKED;

/// Named after the store and not its folder, so moving it keeps the seed and a sandbox never meets it.
fn scope(paths: &Paths) -> Result<String, String> {
    tisty_core::store::peek_identity(paths.store())
        .map(|store| format!("identity-{store}"))
        .ok_or_else(|| "this store has no name yet".to_string())
}

// Joining stores renames this one, so a seed kept under a store it took in is brought under its name now.
pub fn kept(
    paths: &Paths,
    forebears: &BTreeSet<String>,
    fits: &dyn Fn(&Seed) -> bool,
) -> Result<Option<Seed>, String> {
    let Ok(here) = scope(paths) else {
        return Ok(None);
    };
    let own = kept_at(&here)?;
    if own.as_ref().is_some_and(fits) {
        return Ok(own);
    }
    for was in forebears
        .iter()
        .map(|store| format!("identity-{store}"))
        .filter(|was| *was != here)
    {
        let found = kept_at(&was).unwrap_or_else(|why| {
            unread(why);
            None
        });
        if let Some(seed) = found.filter(|seed| fits(seed)) {
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
    Ok(own)
}

fn fitting(persons: tisty_core::person::Persons) -> impl Fn(&Seed) -> bool {
    move |seed| fits_person(&persons, seed)
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
    let (paths, forebears, persons, wanted) = {
        let session = held(session);
        (
            session.paths.clone(),
            session.state.forebears.clone(),
            session.state.persons.clone(),
            gate(&session),
        )
    };
    if !wanted {
        return None;
    }
    let fits = fitting(persons);
    match elsewhere(move || kept(&paths, &forebears, &fits)).await {
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

/// Read only when the sealed cloud this place names is not the one this machine already reads, so the keychain is seldom asked.
pub async fn seal_the_cloud(session: &tauri::State<'_, Mutex<Session>>) {
    let (at, tree) = {
        let session = held(session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            session.reach.seal_with(None);
            return;
        };
        let tree = tisty_carrier::mirrored_at(&session.reach.home_of(&at));
        (at, tree)
    };
    let said = match tisty_carrier::sealing_of(&tree) {
        tisty_carrier::Sealing::Sealed(cloud) => Some(cloud),
        _ => None,
    };
    let (paths, forebears, cloud, named) = {
        let session = held(session);
        let wanted = match said {
            Some(cloud) => session
                .state
                .sealed
                .get(&cloud)
                .map(|named| (cloud.clone(), named.clone())),
            None => session
                .state
                .sealed_cloud()
                .map(|(cloud, named)| (cloud.to_string(), named.to_string())),
        };
        let looked = wanted
            .clone()
            .map(|(cloud, named)| (cloud, named, session.state.forebears.len()));
        if wanted == session.reach.sealed() || (looked.is_some() && looked == session.reach.tried())
        {
            return;
        }
        let Some((cloud, named)) = wanted else {
            session.reach.tried_for(None);
            session.reach.seal_with(None);
            return;
        };
        (
            session.paths.clone(),
            session.state.forebears.clone(),
            cloud,
            named,
        )
    };
    let (looking, wanted) = (cloud.clone(), named.clone());
    let opens = move |seed: &Seed| CloudKey::of(seed).named_for(&looking) == wanted;
    let known = forebears.len();
    let seed = match elsewhere(move || kept(&paths, &forebears, &opens)).await {
        Ok(Ok(seed)) => seed,
        Ok(Err(why)) => {
            unread(why);
            None
        }
        Err(_) => None,
    };
    let key = seed
        .map(|seed| CloudKey::of(&seed))
        .filter(|key| key.named_for(&cloud) == named);
    let session = held(session);
    match key {
        Some(key) => {
            session.reach.tried_for(None);
            session.reach.seal_with(Some(crate::reaching::Sealing {
                at,
                cloud,
                named,
                key,
            }));
        }
        None => {
            witness::note(
                channel::CLOUD,
                "the cloud is sealed with a phrase this machine does not hold, so it is read as it stands",
                &[],
            );
            session.reach.tried_for(Some((cloud, named, known)));
            session.reach.seal_with(None);
        }
    }
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
    pub confirmed: bool,
    pub alias: Option<String>,
    pub machines: Vec<Machine>,
}

#[derive(Serialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub here: bool,
    pub ours: bool,
}

fn words(session: &Session) -> Words {
    match crate::refusing::spanish(session.locale.as_deref()) {
        true => Words::Spanish,
        false => Words::English,
    }
}

fn current(session: &Session) -> Answer<Option<Seed>> {
    Ok(counted(
        kept(&session.paths, &session.state.forebears, &|seed| {
            fits(seed, &session.state)
        })
        .map_err(unread)?,
        &session.state,
    ))
}

/// While the log names nobody, a seed kept here is one that began and never reached it.
fn counted(seed: Option<Seed>, state: &State) -> Option<Seed> {
    seed.filter(|seed| fits(seed, state))
}

fn fits(seed: &Seed, state: &State) -> bool {
    fits_person(&state.persons, seed)
}

fn fits_person(persons: &tisty_core::person::Persons, seed: &Seed) -> bool {
    !persons.names_a_person() || persons.names(&seed.root().person())
}

/// Read off the lock, so the keychain never holds up a round.
fn seed_here(session: &tauri::State<'_, Mutex<Session>>) -> Answer<Option<Seed>> {
    let (paths, forebears, persons) = {
        let session = held(session);
        (
            session.paths.clone(),
            session.state.forebears.clone(),
            session.state.persons.clone(),
        )
    };
    kept(&paths, &forebears, &fitting(persons)).map_err(unread)
}

#[tauri::command(async)]
pub fn identity_shown(session: tauri::State<'_, Mutex<Session>>) -> Answer<Shown> {
    let seed = seed_here(&session)?;
    let session = held(&session);
    let held = seed.as_ref().is_some_and(|seed| ours(seed, &session.state));
    let person = match seed.filter(|_| held) {
        Some(seed) => Some(seed.root().person()),
        None => person::person_of(&session.state, &session.config.device_id),
    };
    let fingerprint = person.map(|one| one.fingerprint());
    let signed = &session.state.signed;
    Ok(Shown {
        held,
        elsewhere: session.state.persons.names_a_person() && !held,
        confirmed: held && fingerprint.is_some() && session.config.phrase_confirmed == fingerprint,
        fingerprint,
        suffix: person.map(|one| one.suffix()),
        alias: signed.alias.clone().or_else(|| signed.name.clone()),
        machines: person::machines(&session.state)
            .into_iter()
            .filter(|device| !session.state.dropped.contains(device))
            .map(|device| Machine {
                name: session
                    .state
                    .named
                    .get(&device)
                    .map(|one| one.name.clone())
                    .unwrap_or_else(|| tisty_core::config::nicknamed(&device.0)),
                here: device == session.config.device_id,
                ours: person.is_some() && session.state.persons.person(&device) == person,
                id: device.0,
            })
            .collect(),
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
    session.reach.tried_for(None);
    begun(&mut session, &seed)?;
    Ok(shown.to_string())
}

#[tauri::command(async)]
pub fn identity_phrase(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let seed = seed_here(&session)?;
    let session = held(&session);
    let seed = counted(seed, &session.state).ok_or_else(|| Refusal::of("identityNone"))?;
    phrase::phrase(&seed, words(&session))
        .map(|said| said.to_string())
        .ok_or_else(|| Refusal::of("identityNone"))
}

#[tauri::command(async)]
pub fn identity_confirm(
    session: tauri::State<'_, Mutex<Session>>,
    said: Vec<(usize, String)>,
) -> Answer<Vec<usize>> {
    let said: Vec<(usize, Zeroizing<String>)> = said
        .into_iter()
        .map(|(at, word)| (at, Zeroizing::new(word)))
        .collect();
    let seed = seed_here(&session)?;
    let mut session = held(&session);
    let seed = counted(seed, &session.state).ok_or_else(|| Refusal::of("identityNone"))?;
    let spoken = [Words::Spanish, Words::English]
        .into_iter()
        .map(|words| phrase::phrase(&seed, words))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| Refusal::of("identityNone"))?;
    // Either language opens the same seed, so a phrase written down in the other one still counts.
    let right = |at: usize, typed: &str| {
        at.checked_sub(1).is_some_and(|at| {
            spoken.iter().any(|one| {
                one.split(' ')
                    .nth(at)
                    .is_some_and(|word| phrase::same_word(typed, word))
            })
        })
    };
    let thirds: BTreeSet<usize> = said
        .iter()
        .filter_map(|(at, _)| at.checked_sub(1))
        .filter(|at| *at < phrase::WORDS)
        .map(|at| at / THIRD)
        .collect();
    if thirds.len() < CHECKED {
        return Ok(said.iter().map(|(at, _)| *at).collect());
    }
    let wrong: Vec<usize> = said
        .iter()
        .filter(|(at, typed)| !right(*at, typed))
        .map(|(at, _)| *at)
        .collect();
    if wrong.is_empty() {
        let fingerprint = seed.root().person().fingerprint();
        session.keep(|config| config.phrase_confirmed = Some(fingerprint))?;
    }
    Ok(wrong)
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
    let here = kept(&session.paths, &session.state.forebears, &|_| false).map_err(unread)?;
    match here {
        Some(here) if *here.to_bytes() == *seed.to_bytes() => {}
        Some(here) if ours(&here, &session.state) => return Err(Refusal::of("identityExists")),
        _ => keep(&session.paths, &seed)?,
    }
    session.reach.tried_for(None);
    begun(&mut session, &seed)?;
    let fingerprint = seed.root().person().fingerprint();
    let _ = session.keep(|config| config.phrase_confirmed = Some(fingerprint));
    Ok(())
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
