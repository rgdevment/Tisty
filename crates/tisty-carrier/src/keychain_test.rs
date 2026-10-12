use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

use keyring_core::api::{CredentialApi, CredentialStoreApi};

use super::*;

type Kept = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// A keychain in memory, so what the system's own does with an answer is checked on every system.
#[derive(Default)]
struct Shelf {
    kept: Kept,
    broken: bool,
}

struct Held {
    kept: Kept,
    user: String,
    broken: bool,
}

fn refused() -> Error {
    Error::Invalid("store".into(), "locked".into())
}

impl CredentialApi for Held {
    fn set_secret(&self, secret: &[u8]) -> keyring_core::Result<()> {
        if self.broken {
            return Err(refused());
        }
        self.kept
            .lock()
            .unwrap()
            .insert(self.user.clone(), secret.to_vec());
        Ok(())
    }

    fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
        if self.broken {
            return Err(refused());
        }
        self.kept
            .lock()
            .unwrap()
            .get(&self.user)
            .cloned()
            .ok_or(Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring_core::Result<()> {
        if self.broken {
            return Err(refused());
        }
        self.kept
            .lock()
            .unwrap()
            .remove(&self.user)
            .map(|_| ())
            .ok_or(Error::NoEntry)
    }

    fn get_credential(&self) -> keyring_core::Result<Option<Arc<keyring_core::Credential>>> {
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some((SERVICE.into(), self.user.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CredentialStoreApi for Shelf {
    fn vendor(&self) -> String {
        "tisty test".into()
    }

    fn id(&self) -> String {
        "memory".into()
    }

    fn build(
        &self,
        _: &str,
        user: &str,
        _: Option<&HashMap<&str, &str>>,
    ) -> keyring_core::Result<Entry> {
        Ok(Entry::new_with_credential(Arc::new(Held {
            kept: self.kept.clone(),
            user: user.to_string(),
            broken: self.broken,
        })))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn round_trip(keychain: &Keychain, key: &str) {
    let long = "1//0".to_string() + &"a".repeat(2000);

    assert_eq!(keychain.read(key).unwrap(), None);
    keychain.write(key, &long).unwrap();
    assert_eq!(keychain.read(key).unwrap().as_deref(), Some(long.as_str()));
    keychain.write(key, "otro").unwrap();
    assert_eq!(keychain.read(key).unwrap().as_deref(), Some("otro"));

    keychain.forget(key).unwrap();
    keychain.forget(key).unwrap();
    assert_eq!(keychain.read(key).unwrap(), None);
}

#[test]
fn a_token_written_is_read_back_whole_and_forgetting_it_twice_is_no_error() {
    round_trip(&Keychain::over(Arc::new(Shelf::default())), "drive:ana");
}

#[test]
fn what_is_not_text_is_never_handed_out_as_a_token() {
    let shelf = Shelf::default();
    shelf
        .kept
        .lock()
        .unwrap()
        .insert("drive:ana".into(), vec![0xff, 0xfe]);
    let keychain = Keychain::over(Arc::new(shelf));

    assert!(matches!(keychain.read("drive:ana"), Err(Hitch::Broke(_))));
}

#[test]
fn a_keychain_that_refuses_is_a_hitch_and_never_an_empty_answer() {
    let keychain = Keychain::over(Arc::new(Shelf {
        broken: true,
        ..Shelf::default()
    }));

    assert!(matches!(keychain.read("drive:ana"), Err(Hitch::Broke(_))));
    assert!(matches!(
        keychain.write("drive:ana", "r0"),
        Err(Hitch::Broke(_))
    ));
    assert!(matches!(keychain.forget("drive:ana"), Err(Hitch::Broke(_))));
}

// Only Windows runners hold a keychain that answers without a person unlocking it.
#[cfg(windows)]
#[test]
fn the_system_keychain_keeps_a_token_whole() {
    round_trip(
        &Keychain::native().unwrap(),
        &format!("test-{}", std::process::id()),
    );
}
