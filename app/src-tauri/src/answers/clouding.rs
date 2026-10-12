use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::Emitter;
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
    let tokens = Tokens::new(unfiled, Arc::new(renew), KEPT_AS, aside);
    let account = tisty_google::Drive::new(Arc::new(tokens), tisty_google::DriveAt::google())
        .and_then(|drive| drive.account())
        .map_err(|e| refused(e, "Google Drive"))?;
    secrets_for(&account.id)
        .and_then(|kept| kept.write(KEPT_AS, &granted.refresh))
        .map_err(|e| refused(e, "Google Drive"))?;
    Ok(Connected {
        provider: GOOGLE,
        account: account.id,
        email: account.email,
    })
}

/// Only a copy that shares with nobody starts on the cloud here; any other way has things to bring over first.
#[tauri::command]
pub fn cloud_choose(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    account: String,
    email: String,
) -> Answer<()> {
    let has_token = secrets_for(&account)
        .and_then(|kept| kept.read(KEPT_AS))
        .map_err(|e| refused(e, "Google Drive"))?
        .is_some();
    if !has_token {
        return Err(Refusal::about("cloudLost", "Google Drive".to_string()));
    }
    let chosen = Sync::cloud(CloudAt {
        provider: GOOGLE.to_string(),
        account,
        email,
    });
    let mut session = held(&session);
    session.refresh_settings();
    session.keep_unless(choosing_cloud(chosen))?;
    drop(session);
    let _ = app.emit("unstuck", ());
    Ok(())
}

pub(crate) fn choosing_cloud(
    chosen: Sync,
) -> impl FnOnce(&mut tisty_core::Config) -> Option<Refusal> {
    let same =
        |was: &CloudAt, now: &CloudAt| was.provider == now.provider && was.account == now.account;
    move |config| match config.sync.as_ref().map(Sync::leaving) {
        None | Some(Leaving::Free) => {
            config.sync = Some(chosen);
            None
        }
        // The account outlives a change of address, so the new address is only written down.
        Some(Leaving::Cloud)
            if tisty_carrier::cloud_of(config.sync.as_ref())
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

#[tauri::command]
pub fn cloud_forget(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    let at = chosen_cloud(&session)?;
    held(&session).reach.let_go();
    secrets_for(&at.account)
        .and_then(|kept| kept.forget(KEPT_AS))
        .map_err(|e| refused(e, "Google Drive"))
}

/// Google takes Tisty's access away from this person on every machine at once; this machine forgets it too.
#[tauri::command(async)]
pub async fn cloud_revoke(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    let at = chosen_cloud(&session)?;
    held(&session).reach.let_go();
    tauri::async_runtime::spawn_blocking(move || {
        let kept = secrets_for(&at.account).map_err(|e| refused(e, "Google Drive"))?;
        if let Some(token) = kept.read(KEPT_AS).map_err(|e| refused(e, "Google Drive"))? {
            tisty_google::revoke_everywhere(&token, &tisty_google::Endpoints::google())
                .map_err(|e| refused(e, "Google Drive"))?;
        }
        kept.forget(KEPT_AS).map_err(|e| refused(e, "Google Drive"))
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
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
