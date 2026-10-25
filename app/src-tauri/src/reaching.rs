use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tisty_carrier::{Cloud, Hitch, Reach, Secrets, Shared, Tokens, policy};
use tisty_core::Paths;
use tisty_core::config::CloudAt;
pub use tisty_core::config::{GOOGLE, ONEDRIVE, WEBDAV};
use tisty_core::witness::{self, Fact, channel};

pub const KEPT_AS: &str = "refresh";

const POLICY: &str = "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json";
const POLICY_SIGNED: &str =
    "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json.sig";
const POLICY_ASKED: &str = "cloud-policy-asked";
const POLICY_EVERY: Duration = Duration::from_secs(60 * 60);
const PATIENCE: Duration = Duration::from_secs(10);

// Built once per account and handed back, so Drive's listing outlives a command.
pub struct Reaching {
    paths: Paths,
    built: Mutex<Option<Built>>,
    why_not: Mutex<Option<String>>,
    keyed: Mutex<Option<Keyed>>,
    tried: Mutex<Option<(Locked, usize)>>,
    seed_of: Mutex<Option<(Option<tisty_share_proto::identity::Person>, usize)>>,
}

#[derive(Debug, Clone)]
pub struct Locked {
    pub at: CloudAt,
    pub cloud: String,
    pub named: String,
}

impl PartialEq for Locked {
    fn eq(&self, other: &Self) -> bool {
        self.at.same_account(&other.at) && self.cloud == other.cloud && self.named == other.named
    }
}

impl Eq for Locked {}

#[derive(Clone)]
pub struct Keyed {
    pub locked: Locked,
    pub key: tisty_share_proto::cloud::CloudKey,
}

struct Built {
    at: CloudAt,
    sealed: Option<Locked>,
    carrier: Shared,
    tokens: Arc<Tokens>,
}

impl Reaching {
    pub fn of(paths: &Paths) -> Self {
        Self {
            paths: paths.clone(),
            built: Mutex::new(None),
            why_not: Mutex::new(None),
            keyed: Mutex::new(None),
            tried: Mutex::new(None),
            seed_of: Mutex::new(None),
        }
    }

    pub fn tried(&self) -> Option<(Locked, usize)> {
        self.tried.lock().ok()?.clone()
    }

    /// Whose seed this machine keeps, as last read with so many stores taken in; asked again once that changes.
    pub fn seed_of(&self, forebears: usize) -> Option<Option<tisty_share_proto::identity::Person>> {
        self.seed_of
            .lock()
            .ok()?
            .filter(|(_, known)| *known == forebears)
            .map(|(person, _)| person)
    }

    pub fn seed_is(&self, person: Option<(Option<tisty_share_proto::identity::Person>, usize)>) {
        if let Ok(mut held) = self.seed_of.lock() {
            *held = person;
        }
    }

    pub fn tried_for(&self, locked: Option<(Locked, usize)>) {
        if let Ok(mut held) = self.tried.lock() {
            *held = locked;
        }
    }

    pub fn sealed(&self) -> Option<Locked> {
        self.keyed().map(|one| one.locked)
    }

    fn keyed(&self) -> Option<Keyed> {
        self.keyed.lock().ok()?.clone()
    }

    pub fn seal_with(&self, keyed: Option<Keyed>) {
        if let Ok(mut held) = self.keyed.lock() {
            *held = keyed;
        }
    }

    pub fn home_of(&self, at: &CloudAt) -> PathBuf {
        self.paths.cloud_home(&at.provider, &at.account)
    }

    pub fn why_not(&self) -> Option<String> {
        self.why_not.lock().ok().and_then(|held| held.clone())
    }

    pub fn apart(&self, at: &CloudAt) -> Option<Shared> {
        let home = self.home_of(at);
        tisty_carrier::off::away(|| cloud_of(&home, at, None))
            .ok()
            .map(|(carrier, _)| carrier)
    }

    pub fn plain(&self, at: &CloudAt) -> Result<Arc<dyn tisty_carrier::Remote>, String> {
        let home = self.home_of(at);
        tisty_carrier::off::away(|| remote_of(&home, at)).map(|(remote, _)| remote)
    }

    // Handed out rather than let go here, so the caller can do it without holding the session.
    pub fn taken(&self) -> Option<Arc<Tokens>> {
        self.built.lock().ok()?.take().map(|was| was.tokens)
    }
}

impl Reach for Reaching {
    fn cloud(&self, at: &CloudAt) -> Option<Shared> {
        let keyed = self.keyed().filter(|one| one.locked.at.same_account(at));
        let sealed = keyed.as_ref().map(|one| one.locked.clone());
        let mut built = self.built.lock().ok()?;
        if let Some(was) = built.as_ref()
            && was.at.same_account(at)
            && was.sealed == sealed
        {
            return Some(was.carrier.clone());
        }
        let home = self.home_of(at);
        let made = tisty_carrier::off::away(|| cloud_of(&home, at, keyed.as_ref()));
        if let Ok(mut why) = self.why_not.lock() {
            *why = made.as_ref().err().cloned();
        }
        let (carrier, tokens) = made.ok()?;
        *built = Some(Built {
            at: at.clone(),
            sealed,
            carrier: carrier.clone(),
            tokens,
        });
        Some(carrier)
    }
}

pub fn offered() -> bool {
    !clouds().is_empty()
}

pub fn clouds() -> Vec<&'static str> {
    [GOOGLE, ONEDRIVE, WEBDAV]
        .into_iter()
        .filter(|one| offers(one))
        .collect()
}

pub fn offers(provider: &str) -> bool {
    match provider {
        GOOGLE => tisty_google::client().is_some(),
        ONEDRIVE => tisty_onedrive::client().is_some(),
        // The person brings the server and the password, so no build lacks what it needs.
        WEBDAV => true,
        _ => false,
    }
}

type Reached = (Arc<dyn tisty_carrier::Remote>, Arc<Tokens>);

fn cloud_of(
    home: &Path,
    at: &CloudAt,
    keyed: Option<&Keyed>,
) -> Result<(Shared, Arc<Tokens>), String> {
    let (inner, tokens) = remote_of(home, at)?;
    let remote: Arc<dyn tisty_carrier::Remote> = match keyed {
        Some(keyed) => match tisty_carrier::Sealed::opened(
            inner,
            keyed.key.clone(),
            &keyed.locked.cloud,
            &keyed.locked.named,
        ) {
            Some(sealed) => Arc::new(sealed),
            None => return Err("the key does not open the cloud the log names".to_string()),
        },
        None => inner,
    };
    Ok((Arc::new(Cloud::over(remote, home)), tokens))
}

fn remote_of(home: &Path, at: &CloudAt) -> Result<Reached, String> {
    match at.provider.as_str() {
        GOOGLE => google_of(home, at),
        ONEDRIVE => onedrive_of(home, at),
        WEBDAV => webdav_of(home, at),
        other => Err(format!("this build does not know the cloud {other}")),
    }
}

fn off_runtime<R: tisty_carrier::Remote + 'static>(inner: R) -> Arc<dyn tisty_carrier::Remote> {
    Arc::new(tisty_carrier::off::OffRuntime(inner))
}

fn onedrive_of(home: &Path, at: &CloudAt) -> Result<Reached, String> {
    let client = tisty_onedrive::client().ok_or("this build has no access to OneDrive")?;
    let secrets = secrets_for(at).map_err(|e| format!("{e:?}"))?;
    let renew = tisty_onedrive::MicrosoftRenew::new(client, tisty_onedrive::Endpoints::microsoft())
        .map_err(|e| format!("{e:?}"))?;
    let tokens = Arc::new(Tokens::new(secrets, Arc::new(renew), KEPT_AS, home));
    let drive = tisty_onedrive::OneDrive::new(tokens.clone(), tisty_onedrive::GraphAt::microsoft())
        .map_err(|e| format!("{e:?}"))?
        .for_account(&at.account)
        .keeping_at(home);
    Ok((off_runtime(drive), tokens))
}

fn webdav_of(home: &Path, at: &CloudAt) -> Result<Reached, String> {
    let dav = tisty_webdav::DavAt::of_account(&at.account).map_err(|e| format!("{e:?}"))?;
    let secrets = secrets_for(at).map_err(|e| format!("{e:?}"))?;
    let tokens = Arc::new(Tokens::new(
        secrets,
        Arc::new(tisty_webdav::Password),
        KEPT_AS,
        home,
    ));
    let server = tisty_webdav::WebDav::new(tokens.clone(), dav).map_err(|e| format!("{e:?}"))?;
    Ok((off_runtime(server), tokens))
}

fn google_of(home: &Path, at: &CloudAt) -> Result<Reached, String> {
    let client = tisty_google::client().ok_or("this build has no access to Google Drive")?;
    let secrets = secrets_for(at).map_err(|e| format!("{e:?}"))?;
    let renew = tisty_google::GoogleRenew::new(client, tisty_google::Endpoints::google())
        .map_err(|e| format!("{e:?}"))?;
    let tokens = Arc::new(Tokens::new(secrets, Arc::new(renew), KEPT_AS, home));
    let drive = tisty_google::Drive::new(tokens.clone(), tisty_google::DriveAt::google())
        .map_err(|e| format!("{e:?}"))?
        .for_account(&at.account)
        .owing_at(home);
    Ok((off_runtime(drive), tokens))
}

// Google's tokens stay where 1.30 kept them, under the account alone; every later provider is named first.
fn scope_of(at: &CloudAt) -> String {
    match at.provider.as_str() {
        GOOGLE => at.account.clone(),
        // A WebDAV account is a whole address, too long and too loose to name a file or a keychain entry.
        WEBDAV => format!(
            "{WEBDAV}-{}",
            tisty_core::attach::printed(at.account.as_bytes())
        ),
        other => format!("{other}-{}", at.account),
    }
}

pub fn secrets_for(at: &CloudAt) -> Result<Arc<dyn Secrets>, Hitch> {
    secrets_in(Shelf::Tokens, &scope_of(at))
}

pub fn seed_secrets(scope: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    secrets_in(Shelf::Seed, scope)
}

/// On Windows the seed sits apart from the tokens, so leaving takes every token and no seed nobody wrote down.
#[derive(Clone, Copy)]
pub enum Shelf {
    Tokens,
    Seed,
}

#[cfg(all(windows, not(test)))]
fn secrets_in(shelf: Shelf, scope: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    let home = match shelf {
        Shelf::Tokens => crate::sealed::home(),
        Shelf::Seed => crate::sealed::seed_home(),
    }
    .ok_or_else(|| Hitch::Broke("Windows named no local folder for this person".into()))?;
    Ok(Arc::new(crate::sealed::Sealed::under(&home, scope)))
}

#[cfg(all(not(windows), not(test)))]
fn secrets_in(_: Shelf, scope: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    Ok(Arc::new(tisty_carrier::Keychain::native(scope)?))
}

// Tests keep tokens in memory, so none ever reaches the keychain of whoever runs them.
#[cfg(test)]
fn secrets_in(_: Shelf, scope: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    Ok(Arc::new(InMemory(scope.to_string())))
}

#[cfg(test)]
static IN_MEMORY: Mutex<std::collections::BTreeMap<String, String>> =
    Mutex::new(std::collections::BTreeMap::new());

#[cfg(test)]
struct InMemory(String);

#[cfg(test)]
impl InMemory {
    fn kept() -> std::sync::MutexGuard<'static, std::collections::BTreeMap<String, String>> {
        IN_MEMORY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
static READS: Mutex<std::collections::BTreeMap<String, usize>> =
    Mutex::new(std::collections::BTreeMap::new());

#[cfg(test)]
pub(crate) fn reads_of(scope: &str) -> usize {
    READS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(scope)
        .copied()
        .unwrap_or(0)
}

#[cfg(test)]
impl Secrets for InMemory {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        *READS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(self.0.clone())
            .or_default() += 1;
        Ok(Self::kept().get(&format!("{}/{key}", self.0)).cloned())
    }
    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        Self::kept().insert(format!("{}/{key}", self.0), value.to_string());
        Ok(())
    }
    fn forget(&self, key: &str) -> Result<(), Hitch> {
        Self::kept().remove(&format!("{}/{key}", self.0));
        Ok(())
    }
}

// Fixed when the release is built, like Google's ids: a build without it keeps the limits it ships with.
fn policy_key() -> Option<&'static str> {
    option_env!("TISTY_CLOUD_POLICY_KEY").filter(|key| !key.is_empty())
}

pub fn heed(home: &Path, provider: &str) {
    let Some(key) = policy_key() else {
        return;
    };
    let asked = home.join(POLICY_ASKED);
    let recently = std::fs::metadata(&asked)
        .and_then(|one| one.modified())
        .ok()
        .and_then(|at| at.elapsed().ok())
        .is_some_and(|since| since < POLICY_EVERY);
    if recently {
        return;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(&asked, b"");
    let (Some(said), Some(signed)) = (brought(POLICY), brought(POLICY_SIGNED)) else {
        return;
    };
    let signature = String::from_utf8_lossy(&signed);
    if let Err(why) = policy::accepted(home, &said, &signature, key, provider) {
        witness::warn(
            channel::CLOUD,
            "the signed word on the cloud was not taken, so the last one taken still stands",
            &[("why", Fact::Why(format!("{why:?}")))],
        );
    }
}

// No redirect is followed and no more is read than a word can weigh, whatever the answer says it holds.
pub(crate) fn brought(url: &str) -> Option<Vec<u8>> {
    let answer = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?
        .get(url)
        .send()
        .ok()?;
    if !answer.status().is_success() {
        return None;
    }
    let mut held = Vec::new();
    answer
        .take(policy::READ_AT_MOST as u64 + 1)
        .read_to_end(&mut held)
        .ok()?;
    Some(held)
}

#[cfg(test)]
#[path = "reaching_test.rs"]
mod tests;
