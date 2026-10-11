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
    held: Mutex<Option<(String, Instant)>>,
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

    pub fn access(&self) -> Result<String, Hitch> {
        let mut held = self
            .held
            .lock()
            .map_err(|_| Hitch::Broke("the access token was left half written".into()))?;
        if let Some((access, until)) = held.as_ref()
            && Instant::now() + RENEWED_AHEAD < *until
        {
            return Ok(access.clone());
        }
        let _turn =
            Held::wait_for(&self.home, RENEWING, RENEWING_WAITED_OUT)?.ok_or(Hitch::Limited {
                wait: RENEWING_WAITED_OUT,
            })?;
        let refresh = self.secrets.read(&self.key)?.ok_or(Hitch::Lost)?;
        let renewed = self.renew.renew(&refresh)?;
        if let Some(next) = renewed.refresh.as_deref() {
            self.secrets.write(&self.key, next)?;
        }
        *held = Some((renewed.access.clone(), Instant::now() + renewed.lasts));
        Ok(renewed.access)
    }
}

#[cfg(test)]
#[path = "tokens_test.rs"]
mod tests;
