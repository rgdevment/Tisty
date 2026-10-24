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

/// Leaving Tisty keeps what starts with this, since a seed nobody wrote down cannot be signed back in.
pub const KEPT_ON_LEAVING: &str = "identity-";
const SEED: &str = "seed";

/// One seed per store, so a sandbox beside the real install never reads or writes the real one.
fn scope(paths: &Paths) -> String {
    let store = paths.store();
    let named = store.to_string_lossy();
    format!(
        "{KEPT_ON_LEAVING}{}",
        tisty_core::attach::printed(named.as_bytes())
    )
}

/// A keychain that cannot answer is not one that holds nothing: only the second may make a seed.
pub fn kept(paths: &Paths) -> Result<Option<Seed>, String> {
    let secrets = crate::reaching::secrets_in(&scope(paths)).map_err(|e| format!("{e:?}"))?;
    let Some(said) = secrets.read(SEED).map_err(|e| format!("{e:?}"))? else {
        return Ok(None);
    };
    let said = Zeroizing::new(said);
    let bytes = Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(said.as_bytes())
            .map_err(|_| "the seed kept here does not read".to_string())?,
    );
    let mut seed = Zeroizing::new([0u8; 32]);
    if bytes.len() != seed.len() {
        return Err("the seed kept here is not one".into());
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
    let secrets =
        crate::reaching::secrets_in(&scope(paths)).map_err(|e| refused(format!("{e:?}")))?;
    let said = Zeroizing::new(URL_SAFE_NO_PAD.encode(seed.to_bytes().as_ref()));
    secrets
        .write(SEED, &said)
        .map_err(|e| refused(format!("{e:?}")))
}

fn unread(why: String) -> Refusal {
    witness::warn(
        channel::STORE,
        "the identity's seed could not be read on this computer",
        &[("why", Fact::Why(why))],
    );
    Refusal::of("identityUnread")
}

/// A seed for a root the log does not name is an old one, and never speaks for the person.
fn ours(seed: &Seed, state: &State) -> bool {
    state.persons.names(&seed.root().person())
}

/// Read off the session lock and only once the log names a person; nobody else meets a keychain.
pub async fn for_a_round(session: &tauri::State<'_, Mutex<Session>>) -> Option<Seed> {
    let (paths, wanted) = looked(session);
    if !wanted {
        return None;
    }
    match elsewhere(move || kept(&paths)).await {
        Ok(Ok(seed)) => seed,
        Ok(Err(why)) => {
            unread(why);
            None
        }
        Err(_) => None,
    }
}

/// Read off the session lock, for a machine about to be taken off the list.
pub fn for_removing(session: &tauri::State<'_, Mutex<Session>>) -> Option<Seed> {
    let (paths, wanted) = looked(session);
    wanted
        .then(|| kept(&paths).map_err(unread).ok().flatten())
        .flatten()
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
    ops.extend(person::left(seed, &session.state.persons, &machines, now));
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

/// With the seed here, the keys go with the machine at once; without it, the round of the
/// machine that has it takes them back.
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
    /// This machine keeps the seed, so its phrase can be shown again.
    pub held: bool,
    /// The log names a person whose seed is not here: the phrase or another machine brings it.
    pub elsewhere: bool,
    pub fingerprint: Option<String>,
    pub suffix: Option<String>,
}

fn words(session: &Session) -> Words {
    match tisty_core::model::spoken(session.locale.as_deref()).starts_with("es") {
        true => Words::Spanish,
        false => Words::English,
    }
}

fn looked(session: &tauri::State<'_, Mutex<Session>>) -> (Paths, bool) {
    let session = held(session);
    (
        session.paths.clone(),
        session.state.persons.names_a_person(),
    )
}

#[tauri::command(async)]
pub fn identity_shown(session: tauri::State<'_, Mutex<Session>>) -> Answer<Shown> {
    let (paths, named) = looked(&session);
    let seed = kept(&paths).map_err(unread)?;
    let session = held(&session);
    let person = person::person_of(&session.state, &session.config.device_id);
    let held = seed.as_ref().is_some_and(|seed| ours(seed, &session.state));
    Ok(Shown {
        held,
        elsewhere: named && !held,
        fingerprint: person.map(|one| one.fingerprint()),
        suffix: person.map(|one| one.suffix()),
    })
}

/// Made only when the person asks for what needs it, and never a second time: a seed kept here
/// that never reached the log is finished, not replaced.
#[tauri::command(async)]
pub fn identity_begin(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let (paths, named) = looked(&session);
    if named {
        return Err(Refusal::of(match kept(&paths) {
            Ok(Some(_)) => "identityExists",
            _ => "identityElsewhere",
        }));
    }
    let seed = match kept(&paths).map_err(unread)? {
        Some(seed) => seed,
        None => {
            let fresh = Seed::minted().ok_or_else(|| Refusal::of("identityNotKept"))?;
            keep(&paths, &fresh)?;
            fresh
        }
    };
    let mut session = held(&session);
    let shown =
        phrase::phrase(&seed, words(&session)).ok_or_else(|| Refusal::of("identityNotKept"))?;
    begun(&mut session, &seed)?;
    Ok(shown.to_string())
}

#[tauri::command(async)]
pub fn identity_phrase(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let (paths, _) = looked(&session);
    let seed = kept(&paths)
        .map_err(unread)?
        .ok_or_else(|| Refusal::of("identityNone"))?;
    let session = held(&session);
    phrase::phrase(&seed, words(&session))
        .map(|said| said.to_string())
        .ok_or_else(|| Refusal::of("identityNone"))
}

/// The phrase must be the person the log names, and never takes the place of another seed kept here.
#[tauri::command(async)]
pub fn identity_recover(session: tauri::State<'_, Mutex<Session>>, said: String) -> Answer<()> {
    let said = Zeroizing::new(said);
    let seed = phrase::seed_from(&said).map_err(|unread| match unread {
        phrase::Unread::Count(count) => Refusal::about("phraseCount", count.to_string()),
        phrase::Unread::Word(at) => Refusal::about("phraseWord", at.to_string()),
        phrase::Unread::Checksum => Refusal::of("phraseChecksum"),
    })?;
    let (paths, named) = looked(&session);
    if named && !held(&session).state.persons.names(&seed.root().person()) {
        return Err(Refusal::of("phraseOtherPerson"));
    }
    match kept(&paths).map_err(unread)? {
        Some(here) if *here.to_bytes() != *seed.to_bytes() => {
            return Err(Refusal::of("identityExists"));
        }
        Some(_) => {}
        None => keep(&paths, &seed)?,
    }
    begun(&mut held(&session), &seed)
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
