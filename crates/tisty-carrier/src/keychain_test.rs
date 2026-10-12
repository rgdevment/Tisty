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
    at_most: Option<usize>,
    twinned: bool,
}

struct Held {
    kept: Kept,
    user: String,
    broken: bool,
    at_most: Option<usize>,
    twinned: bool,
}

fn refused() -> Error {
    Error::Invalid("store".into(), "locked".into())
}

impl CredentialApi for Held {
    fn set_secret(&self, secret: &[u8]) -> keyring_core::Result<()> {
        if self.broken {
            return Err(refused());
        }
        if let Some(most) = self.at_most.filter(|most| secret.len() > *most) {
            return Err(Error::TooLong("the secret".into(), most as u32));
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
        if self.twinned {
            let one = || {
                Entry::new_with_credential(Arc::new(Held {
                    kept: self.kept.clone(),
                    user: self.user.clone(),
                    broken: false,
                    at_most: None,
                    twinned: false,
                }))
            };
            return Err(Error::Ambiguous(vec![one(), one()]));
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
            at_most: self.at_most,
            twinned: self.twinned,
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
    round_trip(
        &Keychain::over(Arc::new(Shelf::default()), "store"),
        "drive:ana",
    );
}

#[test]
fn what_is_not_text_is_never_handed_out_as_a_token() {
    let shelf = Shelf::default();
    shelf
        .kept
        .lock()
        .unwrap()
        .insert("store/drive:ana".into(), vec![0xff, 0xfe]);
    let keychain = Keychain::over(Arc::new(shelf), "store");

    assert!(matches!(keychain.read("drive:ana"), Err(Hitch::Broke(_))));
}

#[test]
fn a_keychain_that_refuses_is_a_hitch_and_never_an_empty_answer() {
    let keychain = Keychain::over(
        Arc::new(Shelf {
            broken: true,
            ..Shelf::default()
        }),
        "store",
    );

    assert!(matches!(keychain.read("drive:ana"), Err(Hitch::Broke(_))));
    assert!(matches!(
        keychain.write("drive:ana", "r0"),
        Err(Hitch::Broke(_))
    ));
    assert!(matches!(keychain.forget("drive:ana"), Err(Hitch::Broke(_))));
}

#[test]
fn two_stores_of_one_person_never_read_or_forget_each_others_token() {
    let shelf: Arc<CredentialStore> = Arc::new(Shelf::default());
    let main = Keychain::over(shelf.clone(), "main");
    let sandbox = Keychain::over(shelf, "sandbox");
    main.write("drive:ana", "r-main").unwrap();

    assert_eq!(sandbox.read("drive:ana").unwrap(), None);
    sandbox.forget("drive:ana").unwrap();
    assert_eq!(main.read("drive:ana").unwrap().as_deref(), Some("r-main"));
}

#[test]
fn a_token_too_long_for_the_keychain_says_so() {
    let keychain = Keychain::over(
        Arc::new(Shelf {
            at_most: Some(2560),
            ..Shelf::default()
        }),
        "store",
    );

    let refused = keychain.write("onedrive:ana", &"a".repeat(3000));

    assert!(
        matches!(&refused, Err(Hitch::Broke(why)) if why.contains("at most 2560 bytes")),
        "{refused:?}"
    );
}

#[test]
fn forgetting_a_token_kept_twice_forgets_both() {
    let keychain = Keychain::over(
        Arc::new(Shelf {
            twinned: true,
            ..Shelf::default()
        }),
        "store",
    );
    keychain.write("drive:ana", "r0").unwrap();

    keychain.forget("drive:ana").unwrap();

    assert_eq!(keychain.read("drive:ana").unwrap(), None);
}

// Only Windows runners hold a keychain that answers without a person unlocking it.
#[cfg(windows)]
#[test]
fn the_system_keychain_keeps_a_token_whole() {
    struct Cleared<'a>(&'a Keychain, String);
    impl Drop for Cleared<'_> {
        fn drop(&mut self) {
            let _ = self.0.forget(&self.1);
        }
    }

    let keychain = Keychain::native(&format!("test-{}", std::process::id())).unwrap();
    let _cleared = Cleared(&keychain, "drive:ana".into());

    round_trip(&keychain, "drive:ana");
}
