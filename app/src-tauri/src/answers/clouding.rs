use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager};
use tisty_carrier::{Hitch, Secrets, Tokens};
use tisty_core::config::{CloudAt, GOOGLE_SHOWN, Leaving, ONEDRIVE_SHOWN, Sync, WEBDAV_SHOWN};

use crate::reaching::{GOOGLE, KEPT_AS, ONEDRIVE, WEBDAV, secrets_for};
use crate::{Answer, Refusal, Session, held};

const CONSENT_WAITED: Duration = Duration::from_secs(5 * 60);
// Microsoft has no way to revoke a personal account's grant from here, so the person is taken where they can.
const ONEDRIVE_CONSENTS: &str = "https://account.live.com/consent/Manage";

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
        Hitch::Emptied => Refusal::about("cloudEmptied", provider.to_string()),
        Hitch::Broke(why) => {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "the cloud refused what was asked",
                &[("why", tisty_core::witness::Fact::Why(why))],
            );
            Refusal::of("syncFailed")
        }
        other => {
            tisty_core::witness::standing(
                tisty_core::witness::channel::CLOUD,
                "the cloud could not be asked now",
                &[
                    ("kind", tisty_core::witness::Fact::Code(other.code())),
                    ("why", tisty_core::witness::Fact::Why(format!("{other:?}"))),
                ],
            );
            Refusal::of("cloudNotNow")
        }
    }
}

/// Asked only while the person chooses the cloud: one request to its API host with no account,
/// no token and nothing of theirs, through whatever proxy the system names.
#[tauri::command]
pub async fn cloud_health(provider: Option<String>) -> Answer<&'static str> {
    let onedrive = match provider.as_deref().unwrap_or(GOOGLE) {
        GOOGLE => false,
        ONEDRIVE => true,
        _ => return Err(Refusal::of("cloudNotOffered")),
    };
    let health = tauri::async_runtime::spawn_blocking(move || match onedrive {
        true => tisty_onedrive::probe(),
        false => tisty_google::health::probe(),
    })
    .await
    .map_err(|_| Refusal::of("internal"))?;
    if health != tisty_carrier::net::Health::Reachable {
        tisty_core::witness::note(
            tisty_core::witness::channel::CLOUD,
            "the way to the cloud's API host did not answer as it should",
            &[
                ("health", tisty_core::witness::Fact::Code(health.code())),
                (
                    "provider",
                    tisty_core::witness::Fact::Word(if onedrive { ONEDRIVE } else { GOOGLE }),
                ),
            ],
        );
    }
    Ok(health.code())
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

/// The one sign-in waiting for the browser; a newer one lets go of it, so two never race for one account.
#[derive(Default)]
pub struct Consenting(Mutex<Option<Arc<AtomicBool>>>);

impl Consenting {
    fn waits_on(&self, dropping: Arc<AtomicBool>) {
        if let Ok(mut held) = self.0.lock()
            && let Some(older) = held.replace(dropping)
        {
            older.store(true, Ordering::Relaxed);
        }
    }

    fn done_with(&self, dropping: &Arc<AtomicBool>) {
        if let Ok(mut held) = self.0.lock()
            && held.as_ref().is_some_and(|now| Arc::ptr_eq(now, dropping))
        {
            *held = None;
        }
    }

    fn let_go(&self) {
        if let Some(waiting) = self.0.lock().ok().and_then(|mut held| held.take()) {
            waiting.store(true, Ordering::Relaxed);
        }
    }
}

enum Asking {
    Google(tisty_google::Client, tisty_google::Consent),
    OneDrive(tisty_onedrive::Client, tisty_onedrive::Consent),
}

impl Asking {
    fn begin(provider: &str) -> Answer<Self> {
        let offered = || Refusal::of("cloudNotOffered");
        match provider {
            GOOGLE => {
                let client = tisty_google::client().ok_or_else(offered)?;
                let consent =
                    tisty_google::Consent::begin(client).map_err(|e| refused(e, GOOGLE_SHOWN))?;
                Ok(Self::Google(client, consent))
            }
            ONEDRIVE => {
                let client = tisty_onedrive::client().ok_or_else(offered)?;
                let consent = tisty_onedrive::Consent::begin(client)
                    .map_err(|e| refused(e, ONEDRIVE_SHOWN))?;
                Ok(Self::OneDrive(client, consent))
            }
            _ => Err(offered()),
        }
    }

    fn url(&self) -> &str {
        match self {
            Self::Google(_, consent) => &consent.url,
            Self::OneDrive(_, consent) => &consent.url,
        }
    }

    fn dropping(&self) -> Arc<AtomicBool> {
        match self {
            Self::Google(_, consent) => consent.dropping(),
            Self::OneDrive(_, consent) => consent.dropping(),
        }
    }

    fn connected(self, aside: &std::path::Path) -> Answer<Connected> {
        match self {
            Self::Google(client, consent) => connected(client, consent, aside),
            Self::OneDrive(client, consent) => onedrive_connected(client, consent, aside),
        }
    }
}

#[tauri::command(async)]
pub async fn cloud_connect(
    session: tauri::State<'_, Mutex<Session>>,
    consenting: tauri::State<'_, Consenting>,
    erasing: tauri::State<'_, crate::Erasing>,
    provider: Option<String>,
) -> Answer<Connected> {
    // Which account a sign-in brings is known only after it, and an erase lets go of its keys when it ends.
    if erasing.going() {
        return Err(Refusal::of("wipeGoing"));
    }
    let provider = provider.unwrap_or_else(|| GOOGLE.to_string());
    let done = connecting(&session, &consenting, &provider).await;
    told_connecting(&provider, &done);
    done
}

fn told_connecting(provider: &str, done: &Answer<Connected>) {
    use tisty_core::witness::{self, Fact, channel};
    let provider = match provider {
        GOOGLE => GOOGLE,
        ONEDRIVE => ONEDRIVE,
        WEBDAV => WEBDAV,
        _ => "other",
    };
    match done {
        Ok(_) => witness::note(
            channel::CLOUD,
            "a cloud sign-in finished",
            &[("provider", Fact::Word(provider))],
        ),
        Err(refusal) => witness::warn(
            channel::CLOUD,
            "a cloud sign-in did not finish",
            &[
                ("provider", Fact::Word(provider)),
                ("how", Fact::Word(how_it_ended(refusal.code))),
                ("code", Fact::Code(refusal.code)),
            ],
        ),
    }
}

fn how_it_ended(code: &str) -> &'static str {
    match code {
        "cloudNoAnswer" => "no-answer",
        "cloudDropped" => "dropped",
        "cloudDenied" | "cloudLost" | "davDenied" => "denied",
        "cloudNotNow" => "unreachable",
        _ => "refused",
    }
}

async fn connecting(
    session: &tauri::State<'_, Mutex<Session>>,
    consenting: &tauri::State<'_, Consenting>,
    provider: &str,
) -> Answer<Connected> {
    let asking = Asking::begin(provider)?;
    let aside = held(session).paths.cache().to_path_buf();
    let dropping = asking.dropping();
    consenting.waits_on(dropping.clone());
    tauri_plugin_opener::open_url(asking.url(), None::<&str>)
        .map_err(|_| Refusal::of("cannotOpen"))?;
    let done = tauri::async_runtime::spawn_blocking(move || asking.connected(&aside))
        .await
        .map_err(|_| Refusal::of("internal"))?;
    consenting.done_with(&dropping);
    let_go_replaced(session, &done);
    done
}

// A token kept from before the new grant must not be written back over it.
fn let_go_replaced(session: &tauri::State<'_, Mutex<Session>>, done: &Answer<Connected>) {
    let in_use = |now: &Connected| {
        let session = held(session);
        tisty_carrier::cloud_of(session.config.sync.as_ref())
            .is_some_and(|at| at.provider == now.provider && at.account == now.account)
            .then(|| session.reach.taken())
            .flatten()
    };
    if let Some(tokens) = done.as_ref().ok().and_then(in_use) {
        tokens.let_go();
    }
}

/// A WebDAV server is signed in to with what the person types, named anew or, to reconnect, by the account it was kept under.
#[tauri::command(async)]
pub async fn cloud_connect_dav(
    session: tauri::State<'_, Mutex<Session>>,
    erasing: tauri::State<'_, crate::Erasing>,
    address: Option<String>,
    user: Option<String>,
    account: Option<String>,
    password: String,
) -> Answer<Connected> {
    if erasing.going() {
        return Err(Refusal::of("wipeGoing"));
    }
    let dav = match account {
        Some(account) => tisty_webdav::DavAt::of_account(&account),
        None => tisty_webdav::DavAt::under(
            address.as_deref().unwrap_or_default(),
            user.as_deref().unwrap_or_default(),
        ),
    }
    .map_err(|_| Refusal::of("davNotAnAddress"))?;
    if password.is_empty() {
        return Err(Refusal::of("davDenied"));
    }
    let aside = held(&session).paths.cache().to_path_buf();
    let done = tauri::async_runtime::spawn_blocking(move || dav_connected(dav, password, &aside))
        .await
        .map_err(|_| Refusal::of("internal"))?;
    told_connecting(WEBDAV, &done);
    let_go_replaced(&session, &done);
    done
}

// Kept only once the server took the password and showed it refuses a write over another copy.
fn dav_connected(
    dav: tisty_webdav::DavAt,
    password: String,
    aside: &std::path::Path,
) -> Answer<Connected> {
    let unfiled = Arc::new(Unfiled(Mutex::new(Some(password.clone()))));
    let tokens = Tokens::new(unfiled, Arc::new(tisty_webdav::Password), KEPT_AS, aside);
    let guards = tisty_webdav::WebDav::new(Arc::new(tokens), dav.clone())
        .and_then(|server| server.guards_writes())
        .map_err(dav_refused)?;
    if !guards {
        return Err(Refusal::of("davUnguarded"));
    }
    let at = CloudAt {
        provider: WEBDAV.to_string(),
        account: dav.account(),
        email: dav.shown(),
    };
    secrets_for(&at)
        .and_then(|kept| kept.write(KEPT_AS, &password))
        .map_err(|e| refused(e, WEBDAV_SHOWN))?;
    Ok(Connected {
        provider: WEBDAV,
        account: at.account,
        email: at.email,
    })
}

fn dav_refused(hitch: Hitch) -> Refusal {
    match hitch {
        Hitch::Lost => Refusal::of("davDenied"),
        Hitch::Broke(_) => Refusal::of("davRefused"),
        other => refused(other, WEBDAV_SHOWN),
    }
}

#[tauri::command]
pub fn cloud_connect_cancel(consenting: tauri::State<'_, Consenting>) {
    consenting.let_go();
}

fn connected(
    client: tisty_google::Client,
    consent: tisty_google::Consent,
    aside: &std::path::Path,
) -> Answer<Connected> {
    let at = tisty_google::Endpoints::google();
    let granted = consent
        .finish(CONSENT_WAITED, &at)
        .map_err(|how| ended(how, GOOGLE_SHOWN))?;
    let renew = tisty_google::GoogleRenew::new(client, at).map_err(|e| refused(e, GOOGLE_SHOWN))?;
    filed(GOOGLE, granted.refresh, Arc::new(renew), aside, |tokens| {
        let account =
            tisty_google::Drive::new(tokens, tisty_google::DriveAt::google())?.account()?;
        Ok((account.id, account.email))
    })
}

fn onedrive_connected(
    client: tisty_onedrive::Client,
    consent: tisty_onedrive::Consent,
    aside: &std::path::Path,
) -> Answer<Connected> {
    let at = tisty_onedrive::Endpoints::microsoft();
    let granted = consent
        .finish(CONSENT_WAITED, &at)
        .map_err(|how| ended(how, ONEDRIVE_SHOWN))?;
    let renew =
        tisty_onedrive::MicrosoftRenew::new(client, at).map_err(|e| refused(e, ONEDRIVE_SHOWN))?;
    filed(
        ONEDRIVE,
        granted.refresh,
        Arc::new(renew),
        aside,
        |tokens| {
            let account =
                tisty_onedrive::OneDrive::new(tokens, tisty_onedrive::GraphAt::microsoft())?
                    .account()?;
            Ok((account.id, account.email))
        },
    )
}

fn filed(
    provider: &'static str,
    refresh: String,
    renew: Arc<dyn tisty_carrier::Renew>,
    aside: &std::path::Path,
    account_of: impl FnOnce(Arc<Tokens>) -> Result<(String, String), Hitch>,
) -> Answer<Connected> {
    let shown = tisty_core::config::shown_of(provider);
    let unfiled = Arc::new(Unfiled(Mutex::new(Some(refresh.clone()))));
    let tokens = Tokens::new(unfiled.clone(), renew, KEPT_AS, aside);
    let (account, email) = account_of(Arc::new(tokens)).map_err(|e| refused(e, shown))?;
    // Learning the account may have renewed it, and the provider can hand back a new one then.
    let refresh = unfiled.read(KEPT_AS).ok().flatten().unwrap_or(refresh);
    let at = CloudAt {
        provider: provider.to_string(),
        account,
        email,
    };
    secrets_for(&at)
        .and_then(|kept| kept.write(KEPT_AS, &refresh))
        .map_err(|e| refused(e, at.shown()))?;
    Ok(Connected {
        provider,
        account: at.account,
        email: at.email,
    })
}

pub(crate) fn ended(how: tisty_carrier::loopback::Ended, shown: &str) -> Refusal {
    use tisty_carrier::loopback::Ended;
    match how {
        Ended::NoAnswer => Refusal::about("cloudNoAnswer", shown.to_string()),
        Ended::Dropped => Refusal::of("cloudDropped"),
        Ended::Failed(Hitch::Lost) => Refusal::about("cloudDenied", shown.to_string()),
        Ended::Failed(other) => refused(other, shown),
    }
}

// An erase going on deletes with this account's keys and lets go of them itself when it ends.
fn unerased(erasing: &crate::Erasing, at: &CloudAt) -> Answer<()> {
    match erasing.of(&at.account) {
        true => Err(Refusal::of("wipeGoing")),
        false => Ok(()),
    }
}

/// Only a copy that shares with nobody starts on the cloud here; any other way has things to bring over first.
#[tauri::command(async)]
pub async fn cloud_choose(
    app: tauri::AppHandle,
    erasing: tauri::State<'_, crate::Erasing>,
    account: String,
    email: String,
    provider: Option<String>,
) -> Answer<()> {
    let at = CloudAt {
        provider: provider.unwrap_or_else(|| GOOGLE.to_string()),
        account,
        email,
    };
    unerased(&erasing, &at)?;
    // Choosing may reach the cloud to clear a word left there, so it runs away from the window's thread.
    tauri::async_runtime::spawn_blocking(move || chosen_cloud_now(&app, at))
        .await
        .map_err(|_| Refusal::of("internal"))?
}

fn chosen_cloud_now(app: &tauri::AppHandle, at: CloudAt) -> Answer<()> {
    has_access(&at)?;
    let chosen = Sync::cloud(at);
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
    erasing: tauri::State<'_, crate::Erasing>,
    account: Option<String>,
    provider: Option<String>,
) -> Answer<()> {
    let at = whose(&session, account, provider)?;
    unerased(&erasing, &at)?;
    forget_here(app, &session, at).await
}

async fn forget_here(
    app: tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    at: CloudAt,
) -> Answer<()> {
    let (paths, home) = {
        let session = held(session);
        (session.paths.clone(), session.reach.home_of(&at))
    };
    tauri::async_runtime::spawn_blocking(move || {
        if crate::leaving::owed_to(&paths, &at.account) {
            if !lost_for_good_at(&home, &at) {
                return Err(Refusal::of("forgetUntold"));
            }
            crate::leaving::owed_no_more_to(&paths, &at.account);
        }
        forgotten(&at)?;
        let_go_held(&app, &at);
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
    erasing: tauri::State<'_, crate::Erasing>,
    account: Option<String>,
    provider: Option<String>,
) -> Answer<()> {
    let at = whose(&session, account, provider)?;
    unerased(&erasing, &at)?;
    if at.provider == ONEDRIVE {
        // Forgotten here first, so the page Microsoft keeps never opens over an access this machine still holds.
        forget_here(app, &session, at).await?;
        return tauri_plugin_opener::open_url(ONEDRIVE_CONSENTS, None::<&str>)
            .map_err(|_| Refusal::of("cannotOpen"));
    }
    if at.provider != GOOGLE {
        return Err(Refusal::of("cloudNotOffered"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let kept = secrets_for(&at).map_err(|e| refused(e, at.shown()))?;
        // Revoked before it is forgotten, so a revoke that fails can be asked again with the token still here.
        if let Some(token) = kept.read(KEPT_AS).map_err(|e| refused(e, at.shown()))? {
            tisty_google::revoke_everywhere(&token, &tisty_google::Endpoints::google())
                .map_err(|e| refused(e, at.shown()))?;
        }
        forgotten(&at)?;
        let_go_held(&app, &at);
        Ok(())
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
}

// A connected account not chosen yet has a token too, so it can be let go of by name.
fn whose(
    session: &tauri::State<'_, Mutex<Session>>,
    account: Option<String>,
    provider: Option<String>,
) -> Answer<CloudAt> {
    let session = held(session);
    match account {
        Some(account) => Ok(CloudAt {
            provider: provider.unwrap_or_else(|| GOOGLE.to_string()),
            account,
            email: String::new(),
        }),
        None => tisty_carrier::cloud_of(session.config.sync.as_ref())
            .cloned()
            .ok_or_else(|| Refusal::of("noRemote")),
    }
}

fn same_account(one: &CloudAt, other: &CloudAt) -> bool {
    one.provider == other.provider && one.account == other.account
}

// Taken only once the stored token is gone, so whatever builds the cloud anew finds nothing to renew.
fn let_go_held(app: &tauri::AppHandle, at: &CloudAt) {
    let Some(session) = app.try_state::<Mutex<Session>>() else {
        return;
    };
    let (tokens, home) = {
        let session = held(&session);
        let chosen = tisty_carrier::cloud_of(session.config.sync.as_ref())
            .filter(|now| same_account(now, at))
            .map(|now| session.reach.home_of(now));
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
                name: Some(at.shown().to_string()),
                ..Default::default()
            },
        );
    }
}

/// Only a keychain that answers it holds nothing counts: one that cannot be read now may answer later.
fn access_gone(at: &CloudAt) -> bool {
    matches!(
        secrets_for(at).and_then(|kept| kept.read(KEPT_AS)),
        Ok(None)
    )
}

/// Read off this disk and the keychain alone, so it can be asked of the chosen cloud or one left behind.
pub(crate) fn lost_for_good_at(home: &std::path::Path, at: &CloudAt) -> bool {
    let last = tisty_carrier::standing(home).last;
    lost_for_good(
        crate::reaching::offers(&at.provider),
        access_gone(at),
        last.as_ref().map(|one| one.code.as_str()),
    )
}

pub(crate) fn lost_for_good(offered: bool, gone: bool, last: Option<&str>) -> bool {
    !offered || gone || last == Some("cloudLost")
}

pub(crate) fn has_access(at: &CloudAt) -> Answer<()> {
    let kept = secrets_for(at)
        .and_then(|kept| kept.read(KEPT_AS))
        .map_err(|e| refused(e, at.shown()))?;
    match kept {
        Some(_) => Ok(()),
        None => Err(Refusal::about("cloudLost", at.shown().to_string())),
    }
}

pub(crate) fn let_go_of(at: &CloudAt) {
    if let Err(why) = forgotten(at) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::CLOUD,
            "the access of a cloud never used here could not be let go of",
            &[("why", tisty_core::witness::Fact::Why(why.code.to_string()))],
        );
    }
}

fn forgotten(at: &CloudAt) -> Answer<()> {
    secrets_for(at)
        .and_then(|kept| kept.forget(KEPT_AS))
        .map_err(|e| refused(e, at.shown()))
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
    lost: bool,
    /// The place holds another history, and the next round takes it on its own instead of asking.
    adopts: bool,
}

/// What Settings and Maintenance say of the chosen cloud, read off this disk with no request.
#[tauri::command(async)]
pub async fn cloud_status(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<Option<CloudStatus>> {
    let (at, home, here, paths, took) = {
        let session = held(&session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return Ok(None);
        };
        let home = session.reach.home_of(&at);
        (
            at,
            home,
            session.here(),
            session.paths.clone(),
            session.config.took_quietly.clone(),
        )
    };
    tauri::async_runtime::spawn_blocking(move || {
        let joined = tisty_carrier::cloud_joined(&home, &here);
        let mut status = status_of(&at, &home, joined);
        status.lost = lost_for_good_at(&home, &at);
        status.adopts = status.last.as_ref().is_some_and(|last| {
            last.code == "otherStore"
                && super::joining::new_here(&paths)
                && super::joining::may_adopt(
                    &last.code,
                    last.name.as_deref(),
                    &tisty_carrier::kin_mirrored(&home, &here),
                    took.as_deref(),
                )
        });
        Some(status)
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
        lost: false,
        adopts: false,
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
