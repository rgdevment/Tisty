use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Serialize;
use tisty_core::person;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::phrase::{self, Words};
use tisty_share_proto::seed::Seed;
use zeroize::Zeroizing;

use crate::{Answer, Refusal, Session, blamed, held};

const SCOPE: &str = "identity";
const SEED: &str = "seed";

/// Kept like the cloud's tokens, so nobody is asked for anything to read it back.
pub fn kept() -> Option<Seed> {
    let secrets = crate::reaching::secrets_in(SCOPE).ok()?;
    let said = Zeroizing::new(secrets.read(SEED).ok()??);
    let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(said.as_bytes()).ok()?);
    let seed: [u8; 32] = bytes.as_slice().try_into().ok()?;
    Some(Seed::from_bytes(&seed))
}

fn keep(seed: &Seed) -> Answer<()> {
    let secrets = crate::reaching::secrets_in(SCOPE)
        .map_err(|e| Refusal::about("internalNamed", format!("{e:?}")))?;
    let said = Zeroizing::new(URL_SAFE_NO_PAD.encode(seed.to_bytes().as_ref()));
    secrets
        .write(SEED, &said)
        .map_err(|e| Refusal::about("internalNamed", format!("{e:?}")))
}

/// Nothing here runs until a person exists, so whoever never needs one never meets a keychain.
pub fn after_a_round(session: &mut Session) {
    if session.state.persons.certificates.is_empty() {
        return;
    }
    let mut ops = Vec::new();
    ops.extend(person::introduced(
        &session.paths,
        &session.config.device_id,
        &session.state.persons,
    ));
    if let Some(seed) = kept() {
        ops.extend(person::certified(
            &seed,
            &session.state.persons,
            &person::machines(&session.state),
            jiff::Timestamp::now(),
        ));
    }
    if ops.is_empty() {
        return;
    }
    if let Err(e) = session.commit_all(ops) {
        witness::warn(
            channel::STORE,
            "this machine could not say its share key or vouch for the others",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
}

/// Taking a machine off the list also takes back its keys, or it would go on signing as the person.
pub fn removing(session: &Session, device: tisty_core::event::DeviceId) -> Vec<tisty_core::Op> {
    let mut ops = match (session.state.persons.certificates.is_empty(), kept()) {
        (false, Some(seed)) => person::revoked(
            &seed,
            &session.state.persons,
            &device,
            jiff::Timestamp::now(),
        ),
        _ => Vec::new(),
    };
    ops.push(tisty_core::Op::DeviceRemove { d: device });
    ops
}

#[derive(Serialize)]
pub struct Shown {
    pub held: bool,
    pub fingerprint: Option<String>,
    pub suffix: Option<String>,
}

fn words(said: &str) -> Words {
    match said {
        "es" => Words::Spanish,
        _ => Words::English,
    }
}

fn begin(session: &mut Session, seed: &Seed) -> Answer<()> {
    keep(seed)?;
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

#[tauri::command]
pub fn identity_shown(session: tauri::State<'_, Mutex<Session>>) -> Answer<Shown> {
    let session = held(&session);
    let person = person::person_of(&session.state, &session.config.device_id);
    Ok(Shown {
        held: kept().is_some(),
        fingerprint: person.map(|one| one.fingerprint()),
        suffix: person.map(|one| one.suffix()),
    })
}

/// Made only when the person asks for what needs it; there is never a second one to replace the first.
#[tauri::command]
pub fn identity_begin(session: tauri::State<'_, Mutex<Session>>, tongue: String) -> Answer<String> {
    if kept().is_some() {
        return Err(Refusal::of("identityExists"));
    }
    let seed = Seed::minted().ok_or_else(|| Refusal::of("identityNotKept"))?;
    let shown =
        phrase::phrase(&seed, words(&tongue)).ok_or_else(|| Refusal::of("identityNotKept"))?;
    begin(&mut held(&session), &seed)?;
    Ok(shown.to_string())
}

#[tauri::command]
pub fn identity_phrase(tongue: String) -> Answer<String> {
    let seed = kept().ok_or_else(|| Refusal::of("identityNotKept"))?;
    phrase::phrase(&seed, words(&tongue))
        .map(|said| said.to_string())
        .ok_or_else(|| Refusal::of("identityNotKept"))
}

#[tauri::command]
pub fn identity_recover(session: tauri::State<'_, Mutex<Session>>, said: String) -> Answer<()> {
    let said = Zeroizing::new(said);
    let seed = phrase::seed_from(&said).map_err(|unread| match unread {
        phrase::Unread::Count(count) => Refusal::about("phraseCount", count.to_string()),
        phrase::Unread::Word(at) => Refusal::about("phraseWord", at.to_string()),
        phrase::Unread::Checksum => Refusal::of("phraseChecksum"),
    })?;
    begin(&mut held(&session), &seed)
}
