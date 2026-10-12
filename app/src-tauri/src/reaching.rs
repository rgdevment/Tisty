use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tisty_carrier::{Cloud, Hitch, Reach, Secrets, Shared, Tokens, policy};
use tisty_core::Paths;
use tisty_core::config::CloudAt;
use tisty_core::witness::{self, Fact, channel};

pub const GOOGLE: &str = "google";
/// The name the refresh token is kept under, inside the account's own scope.
pub const KEPT_AS: &str = "refresh";

const POLICY: &str = "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json";
const POLICY_SIGNED: &str =
    "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json.sig";
const POLICY_ASKED: &str = "cloud-policy-asked";
const POLICY_EVERY: Duration = Duration::from_secs(60 * 60);
const PATIENCE: Duration = Duration::from_secs(10);

/// Builds the cloud carrier once per account and hands the same one back, so Drive's listing is kept between calls.
pub struct Reaching {
    paths: Paths,
    built: Mutex<Option<(CloudAt, Shared)>>,
}

impl Reaching {
    pub fn of(paths: &Paths) -> Self {
        Self {
            paths: paths.clone(),
            built: Mutex::new(None),
        }
    }

    pub fn home_of(&self, at: &CloudAt) -> PathBuf {
        self.paths.cloud_home(&at.provider, &at.account)
    }
}

impl Reach for Reaching {
    fn cloud(&self, at: &CloudAt) -> Option<Shared> {
        let mut built = self.built.lock().ok()?;
        if let Some((was, carrier)) = built.as_ref()
            && was == at
        {
            return Some(carrier.clone());
        }
        let carrier = cloud_of(&self.home_of(at), at)?;
        *built = Some((at.clone(), carrier.clone()));
        Some(carrier)
    }
}

/// Whether this build can reach any cloud at all: one made without Google's ids offers none.
pub fn offered() -> bool {
    tisty_google::client().is_some()
}

fn cloud_of(home: &Path, at: &CloudAt) -> Option<Shared> {
    if at.provider != GOOGLE {
        return None;
    }
    let client = tisty_google::client()?;
    let secrets = secrets_for(&at.account).ok()?;
    let renew = tisty_google::GoogleRenew::new(client, tisty_google::Endpoints::google()).ok()?;
    let tokens = Arc::new(Tokens::new(secrets, Arc::new(renew), KEPT_AS, home));
    let drive = tisty_google::Drive::new(tokens, tisty_google::DriveAt::google())
        .ok()?
        .for_account(&at.account);
    Some(Arc::new(Cloud::over(Arc::new(drive), home)))
}

#[cfg(windows)]
pub fn secrets_for(account: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    let home = crate::sealed::home()
        .ok_or_else(|| Hitch::Broke("Windows named no local folder for this person".into()))?;
    Ok(Arc::new(crate::sealed::Sealed::under(&home, account)))
}

#[cfg(not(windows))]
pub fn secrets_for(account: &str) -> Result<Arc<dyn Secrets>, Hitch> {
    Ok(Arc::new(tisty_carrier::Keychain::native(account)?))
}

// Fixed when the release is built, like Google's ids: a build without it keeps the limits it ships with.
fn policy_key() -> Option<&'static str> {
    option_env!("TISTY_CLOUD_POLICY_KEY").filter(|key| !key.is_empty())
}

/// The maintainer's signed word on the cloud, asked at most once an hour and only while a cloud is chosen.
pub fn heed(home: &Path) {
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
    if let Err(why) = policy::accepted(home, &said, &signature, key) {
        witness::warn(
            channel::CLOUD,
            "the signed word on the cloud was not taken, so the last one taken still stands",
            &[("why", Fact::Why(format!("{why:?}")))],
        );
    }
}

// No redirect is followed and no more is read than a word can weigh, whatever the answer says it holds.
fn brought(url: &str) -> Option<Vec<u8>> {
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
