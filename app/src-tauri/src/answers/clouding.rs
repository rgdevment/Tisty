use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager};
use tisty_carrier::{Hitch, Secrets, Tokens};
use tisty_core::config::{CloudAt, Leaving, Sync};

use crate::reaching::{GOOGLE, KEPT_AS, secrets_for};
use crate::{Answer, Refusal, Session, held};

const CONSENT_WAITED: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connected {
    provider: &'static str,
    account: String,
    email: String,
}

pub(crate) fn refused(hitch: Hitch, provider: &str) -> Refusal {
    match hitch {
        Hitch::Lost => Refusal::about("cloudLost", provider.to_string()),
        Hitch::Elsewhere { found } => Refusal::about("cloudElsewhere", found),
        Hitch::Full => Refusal::about("cloudFull", provider.to_string()),
        Hitch::Broke(why) => Refusal::about("syncBroke", why),
        _ => Refusal::of("cloudNotNow"),
    }
}

/// Held in memory only until Google says whose account it is, so the token is filed under the right one.
struct Unfiled(Mutex<Option<String>>);

impl Secrets for Unfiled {
    fn read(&self, _: &str) -> Result<Option<String>, Hitch> {
        Ok(self.0.lock().ok().and_then(|held| held.clone()))
    }
    fn write(&self, _: &str, value: &str) -> Result<(), Hitch> {
        if let Ok(mut held) = self.0.lock() {
            *held = Some(value.to_string());
        }
        Ok(())
    }
    fn forget(&self, _: &str) -> Result<(), Hitch> {
        if let Ok(mut held) = self.0.lock() {
            *held = None;
        }
        Ok(())
    }
}

#[tauri::command(async)]
pub async fn cloud_connect(session: tauri::State<'_, Mutex<Session>>) -> Answer<Connected> {
    let client = tisty_google::client().ok_or_else(|| Refusal::of("cloudNotOffered"))?;
    let aside = held(&session).paths.cache().to_path_buf();
    let consent = tisty_google::Consent::begin(client).map_err(|e| refused(e, "Google Drive"))?;
    tauri_plugin_opener::open_url(&consent.url, None::<&str>)
        .map_err(|_| Refusal::of("cannotOpen"))?;
    tauri::async_runtime::spawn_blocking(move || connected(client, consent, &aside))
        .await
        .map_err(|_| Refusal::of("internal"))?
}

fn connected(
    client: tisty_google::Client,
    consent: tisty_google::Consent,
    aside: &std::path::Path,
) -> Answer<Connected> {
    let at = tisty_google::Endpoints::google();
    let granted = consent.finish(CONSENT_WAITED, &at).map_err(|e| match e {
        Hitch::Lost => Refusal::of("cloudDenied"),
        other => refused(other, "Google Drive"),
    })?;
    let unfiled = Arc::new(Unfiled(Mutex::new(Some(granted.refresh.clone()))));
    let renew =
        tisty_google::GoogleRenew::new(client, at).map_err(|e| refused(e, "Google Drive"))?;
    let tokens = Tokens::new(unfiled.clone(), Arc::new(renew), KEPT_AS, aside);
    let account = tisty_google::Drive::new(Arc::new(tokens), tisty_google::DriveAt::google())
        .and_then(|drive| drive.account())
        .map_err(|e| refused(e, "Google Drive"))?;
    // Learning the account may have renewed it, and Google can hand back a new one then.
    let refresh = unfiled
        .read(KEPT_AS)
        .ok()
        .flatten()
        .unwrap_or(granted.refresh);
    secrets_for(&account.id)
        .and_then(|kept| kept.write(KEPT_AS, &refresh))
        .map_err(|e| refused(e, "Google Drive"))?;
    Ok(Connected {
        provider: GOOGLE,
        account: account.id,
        email: account.email,
    })
}

/// Only a copy that shares with nobody starts on the cloud here; any other way has things to bring over first.
#[tauri::command(async)]
pub async fn cloud_choose(app: tauri::AppHandle, account: String, email: String) -> Answer<()> {
    // Choosing may reach the cloud to clear a word left there, so it runs away from the window's thread.
    tauri::async_runtime::spawn_blocking(move || chosen_cloud_now(&app, account, email))
        .await
        .map_err(|_| Refusal::of("internal"))?
}

fn chosen_cloud_now(app: &tauri::AppHandle, account: String, email: String) -> Answer<()> {
    has_access(&account)?;
    let chosen = Sync::cloud(CloudAt {
        provider: GOOGLE.to_string(),
        account,
        email,
    });
    let state = app.state::<Mutex<Session>>();
    let mut session = held(&state);
    session.refresh_settings();
    let joined = super::storing::cloud_joined(&session);
    let home = tisty_carrier::cloud_of(Some(&chosen)).map(|at| session.reach.home_of(at));
    super::storing::changed_to(&mut session, chosen, |one| choosing_cloud(one, joined))?;
    drop(session);
    if let Some(home) = home {
        tisty_carrier::round_forgotten(&home);
    }
    let _ = app.emit("unstuck", ());
    Ok(())
}

pub(crate) fn choosing_cloud(
    chosen: Sync,
    joined: bool,
) -> impl FnOnce(&mut tisty_core::Config) -> Option<Refusal> {
    let same =
        |was: &CloudAt, now: &CloudAt| was.provider == now.provider && was.account == now.account;
    move |config| match config.sync.as_ref().map(Sync::leaving) {
        None | Some(Leaving::Free) => {
            config.sync = Some(chosen);
            None
        }
        // One never carried into holds nothing, and the same account under a new address is only written down.
        Some(Leaving::Cloud)
            if !joined
                || tisty_carrier::cloud_of(config.sync.as_ref())
                    .zip(tisty_carrier::cloud_of(Some(&chosen)))
                    .is_some_and(|(was, now)| same(was, now)) =>
        {
            config.sync = Some(chosen);
            None
        }
        Some(Leaving::Later) => Some(Refusal::of("syncLaterToLeave")),
        Some(_) => Some(Refusal::of("cloudNeedsMove")),
    }
}

#[tauri::command(async)]
pub async fn cloud_forget(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    account: Option<String>,
) -> Answer<()> {
    let account = whose(&session, account)?;
    tauri::async_runtime::spawn_blocking(move || {
        forgotten(&account)?;
        let_go_held(&app, &account);
        Ok(())
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
}

/// Google takes Tisty's access away from this person on every machine at once; this machine forgets it too.
#[tauri::command(async)]
pub async fn cloud_revoke(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    account: Option<String>,
) -> Answer<()> {
    let account = whose(&session, account)?;
    tauri::async_runtime::spawn_blocking(move || {
        let kept = secrets_for(&account).map_err(|e| refused(e, "Google Drive"))?;
        // Revoked before it is forgotten, so a revoke that fails can be asked again with the token still here.
        if let Some(token) = kept.read(KEPT_AS).map_err(|e| refused(e, "Google Drive"))? {
            tisty_google::revoke_everywhere(&token, &tisty_google::Endpoints::google())
                .map_err(|e| refused(e, "Google Drive"))?;
        }
        forgotten(&account)?;
        let_go_held(&app, &account);
        Ok(())
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
}

// A connected account not chosen yet has a token too, so it can be let go of by name.
fn whose(session: &tauri::State<'_, Mutex<Session>>, account: Option<String>) -> Answer<String> {
    let session = held(session);
    account
        .or_else(|| {
            tisty_carrier::cloud_of(session.config.sync.as_ref()).map(|at| at.account.clone())
        })
        .ok_or_else(|| Refusal::of("noRemote"))
}

// Taken only once the stored token is gone, so whatever builds the cloud anew finds nothing to renew.
fn let_go_held(app: &tauri::AppHandle, account: &str) {
    let Some(session) = app.try_state::<Mutex<Session>>() else {
        return;
    };
    let (tokens, home) = {
        let session = held(&session);
        let chosen = tisty_carrier::cloud_of(session.config.sync.as_ref())
            .filter(|at| at.account == account)
            .map(|at| session.reach.home_of(at));
        let tokens = chosen.as_ref().and_then(|_| session.reach.taken());
        (tokens, chosen)
    };
    if let Some(tokens) = tokens {
        tokens.let_go();
    }
    // Settings says so at once, not only after a round finds the door shut.
    if let Some(home) = home {
        tisty_carrier::round_noted(
            &home,
            &tisty_carrier::LastRound {
                at: jiff::Timestamp::now().as_second().unsigned_abs(),
                code: "cloudLost".into(),
                name: Some("Google Drive".into()),
                left: 0,
                names: Vec::new(),
            },
        );
    }
}

pub(crate) fn has_access(account: &str) -> Answer<()> {
    let kept = secrets_for(account)
        .and_then(|kept| kept.read(KEPT_AS))
        .map_err(|e| refused(e, "Google Drive"))?;
    match kept {
        Some(_) => Ok(()),
        None => Err(Refusal::about("cloudLost", "Google Drive".to_string())),
    }
}

/// A cloud left before this machine ever carried into it keeps no access behind.
pub(crate) fn let_go_of(account: &str) {
    if let Err(why) = forgotten(account) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::CLOUD,
            "the access of a cloud never used here could not be let go of",
            &[("why", tisty_core::witness::Fact::Why(why.code.to_string()))],
        );
    }
}

fn forgotten(account: &str) -> Answer<()> {
    secrets_for(account)
        .and_then(|kept| kept.forget(KEPT_AS))
        .map_err(|e| refused(e, "Google Drive"))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudStatus {
    provider: String,
    account: String,
    email: String,
    joined: bool,
    units_spent: u64,
    units_a_day: u64,
    bytes_spent: u64,
    bytes_a_day: u64,
    paused_here: bool,
    halted_until: Option<u64>,
    owed: usize,
    last: Option<tisty_carrier::LastRound>,
}

/// What Settings and Maintenance say of the chosen cloud, read off this disk with no request.
#[tauri::command(async)]
pub async fn cloud_status(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<Option<CloudStatus>> {
    let (at, home, here) = {
        let session = held(&session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return Ok(None);
        };
        let home = session.reach.home_of(&at);
        (at, home, session.here())
    };
    tauri::async_runtime::spawn_blocking(move || {
        let joined = tisty_carrier::cloud_joined(&home, &here);
        Some(status_of(&at, &home, joined))
    })
    .await
    .map_err(|_| Refusal::of("internal"))
}

pub(crate) fn status_of(at: &CloudAt, home: &std::path::Path, joined: bool) -> CloudStatus {
    let told = tisty_carrier::standing(home);
    CloudStatus {
        provider: at.provider.clone(),
        account: at.account.clone(),
        email: at.email.clone(),
        joined,
        units_spent: told.units_spent,
        units_a_day: told.units_a_day,
        bytes_spent: told.bytes_spent,
        bytes_a_day: told.bytes_a_day,
        paused_here: told.paused_here,
        halted_until: told.halted_until,
        owed: told.owed + told.forgotten,
        last: told.last,
    }
}

#[tauri::command]
pub fn cloud_pause(session: tauri::State<'_, Mutex<Session>>, paused: bool) -> Answer<()> {
    let at = chosen_cloud(&session)?;
    let home = held(&session).reach.home_of(&at);
    tisty_carrier::policy::pause_here(&home, paused)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))
}

fn chosen_cloud(session: &tauri::State<'_, Mutex<Session>>) -> Answer<CloudAt> {
    let session = held(session);
    tisty_carrier::cloud_of(session.config.sync.as_ref())
        .cloned()
        .ok_or_else(|| Refusal::of("noRemote"))
}

#[cfg(test)]
#[path = "clouding_test.rs"]
mod tests;
