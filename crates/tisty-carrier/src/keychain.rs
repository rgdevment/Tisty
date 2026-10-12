use std::collections::HashMap;
use std::sync::Arc;

use keyring_core::{CredentialStore, Entry, Error};

use crate::{Hitch, Secrets};

const SERVICE: &str = "tisty";

/// The system's own keychain: Credential Manager, the macOS Keychain or the Secret Service.
pub struct Keychain {
    store: Arc<CredentialStore>,
    scope: String,
}

impl Keychain {
    /// `scope` keeps apart what two stores or two profiles of one person hold for the same account.
    pub fn native(scope: &str) -> Result<Self, Hitch> {
        native_store()
            .map(|store| Self::over(store, scope))
            .map_err(trouble)
    }

    pub(crate) fn over(store: Arc<CredentialStore>, scope: &str) -> Self {
        Self {
            store,
            scope: scope.to_string(),
        }
    }

    fn entry(&self, key: &str) -> Result<Entry, Hitch> {
        let named = format!("{}/{key}", self.scope);
        let kept = kept_here();
        let modifiers = (!kept.is_empty()).then_some(&kept);
        self.store
            .build(SERVICE, &named, modifiers)
            .map_err(trouble)
    }
}

impl Secrets for Keychain {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        match self.entry(key)?.get_secret() {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| Hitch::Broke("a token in the keychain is not text".into())),
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(trouble(e)),
        }
    }

    // Written as bytes: Credential Manager holds a password as UTF-16, which halves the room a long token has.
    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        self.entry(key)?
            .set_secret(value.as_bytes())
            .map_err(trouble)
    }

    fn forget(&self, key: &str) -> Result<(), Hitch> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(Error::Ambiguous(all)) => {
                all.iter()
                    .try_for_each(|one| match one.delete_credential() {
                        Ok(()) | Err(Error::NoEntry) => Ok(()),
                        Err(e) => Err(trouble(e)),
                    })
            }
            Err(e) => Err(trouble(e)),
        }
    }
}

// Enterprise, Windows' default, roams with a domain profile, and another machine would rotate the same token.
#[cfg(windows)]
fn kept_here() -> HashMap<&'static str, &'static str> {
    HashMap::from([("persistence", "Local")])
}

#[cfg(not(windows))]
fn kept_here() -> HashMap<&'static str, &'static str> {
    HashMap::new()
}

#[cfg(windows)]
fn native_store() -> keyring_core::Result<Arc<CredentialStore>> {
    Ok(windows_native_keyring_store::Store::new()?)
}

#[cfg(target_os = "macos")]
fn native_store() -> keyring_core::Result<Arc<CredentialStore>> {
    Ok(apple_native_keyring_store::keychain::Store::new()?)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn native_store() -> keyring_core::Result<Arc<CredentialStore>> {
    Ok(zbus_secret_service_keyring_store::Store::new()?)
}

fn trouble(e: Error) -> Hitch {
    match e {
        Error::TooLong(what, most) => Hitch::Broke(format!(
            "the keychain holds at most {most} bytes for {what}, and this token is longer"
        )),
        other => Hitch::Broke(format!("the keychain refused: {other}")),
    }
}

#[cfg(test)]
#[path = "keychain_test.rs"]
mod tests;
