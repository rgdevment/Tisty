use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::Hitch;
use crate::cloud::lock::{Held, RENEWING};

const RENEWED_AHEAD: Duration = Duration::from_secs(60);
const RENEWING_WAITED_OUT: Duration = Duration::from_secs(30);

pub trait Secrets: Send + Sync {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch>;
    fn write(&self, key: &str, value: &str) -> Result<(), Hitch>;
    fn forget(&self, key: &str) -> Result<(), Hitch>;
}

pub struct Renewed {
    pub access: String,
    pub lasts: Duration,
    pub refresh: Option<String>,
}

pub trait Renew: Send + Sync {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch>;
}

pub struct Tokens {
    secrets: Arc<dyn Secrets>,
    renew: Arc<dyn Renew>,
    key: String,
    home: PathBuf,
    held: Mutex<InMemory>,
}

#[derive(Default)]
struct InMemory {
    access: Option<(String, Instant)>,
    unsaved: Option<String>,
}

impl Tokens {
    pub fn new(
        secrets: Arc<dyn Secrets>,
        renew: Arc<dyn Renew>,
        key: impl Into<String>,
        home: impl Into<PathBuf>,
    ) -> Self {
        Self {
            secrets,
            renew,
            key: key.into(),
            home: home.into(),
            held: Mutex::default(),
        }
    }

    /// What is held in memory goes too, so a machine that let go stops reaching the cloud at once.
    pub fn let_go(&self) {
        if let Ok(mut held) = self.held.lock() {
            *held = InMemory::default();
        }
    }

    pub fn spoiled(&self, access: &str) {
        if let Ok(mut held) = self.held.lock()
            && held.access.as_ref().is_some_and(|(kept, _)| kept == access)
        {
            held.access = None;
        }
    }

    pub fn access(&self) -> Result<String, Hitch> {
        let mut held = self
            .held
            .lock()
            .map_err(|_| Hitch::Broke("the access token was left half written".into()))?;
        if let Some(next) = held.unsaved.clone()
            && self.secrets.write(&self.key, &next).is_ok()
        {
            held.unsaved = None;
        }
        if let Some((access, until)) = held.access.as_ref()
            && Instant::now() + RENEWED_AHEAD < *until
        {
            return Ok(access.clone());
        }
        let _turn =
            Held::wait_for(&self.home, RENEWING, RENEWING_WAITED_OUT)?.ok_or(Hitch::Busy {
                wait: RENEWING_WAITED_OUT,
            })?;
        let refresh = match held.unsaved.clone() {
            Some(unsaved) => unsaved,
            None => self.secrets.read(&self.key)?.ok_or(Hitch::Lost)?,
        };
        let renewed = self.renew.renew(&refresh)?;
        let until = Instant::now()
            .checked_add(renewed.lasts)
            .unwrap_or_else(|| Instant::now() + RENEWED_AHEAD * 2);
        held.access = Some((renewed.access.clone(), until));
        if let Some(next) = renewed.refresh
            && let Err(hitch) = self.secrets.write(&self.key, &next)
        {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "the keychain would not keep the renewed token, so it lasts only while Tisty stays open",
                &[("why", tisty_core::witness::Fact::Why(format!("{hitch:?}")))],
            );
            held.unsaved = Some(next);
        }
        Ok(renewed.access)
    }
}

#[cfg(test)]
#[path = "tokens_test.rs"]
mod tests;
