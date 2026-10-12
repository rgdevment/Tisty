use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;

const KEY: &str = "drive:ana";

#[derive(Default)]
struct Kept(
    Mutex<BTreeMap<String, String>>,
    std::sync::atomic::AtomicBool,
);

impl Secrets for Kept {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }

    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        if self.1.load(Ordering::SeqCst) {
            return Err(Hitch::Broke("the keychain is locked".into()));
        }
        self.0
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn forget(&self, key: &str) -> Result<(), Hitch> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

struct Rotating {
    current: Mutex<String>,
    lasts: Duration,
    renewed: AtomicUsize,
    slow: Duration,
}

impl Rotating {
    fn lasting(lasts: Duration) -> Self {
        Self {
            current: Mutex::new("r0".into()),
            lasts,
            renewed: AtomicUsize::new(0),
            slow: Duration::ZERO,
        }
    }
}

impl Renew for Rotating {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch> {
        let was = self.current.lock().unwrap().clone();
        std::thread::sleep(self.slow);
        let mut current = self.current.lock().unwrap();
        if refresh != was || *current != was {
            return Err(Hitch::Lost);
        }
        let n = self.renewed.fetch_add(1, Ordering::SeqCst) + 1;
        *current = format!("r{n}");
        Ok(Renewed {
            access: format!("a{n}"),
            lasts: self.lasts,
            refresh: Some(current.clone()),
        })
    }
}

fn kept_with(refresh: &str) -> Arc<Kept> {
    let kept = Arc::new(Kept::default());
    kept.write(KEY, refresh).unwrap();
    kept
}

#[test]
fn an_access_token_that_still_has_time_is_used_again_without_renewing() {
    let home = tempfile::tempdir().unwrap();
    let renew = Arc::new(Rotating::lasting(Duration::from_secs(3600)));
    let tokens = Tokens::new(kept_with("r0"), renew.clone(), KEY, home.path());

    assert_eq!(tokens.access().unwrap(), "a1");
    assert_eq!(tokens.access().unwrap(), "a1");
    assert_eq!(renew.renewed.load(Ordering::SeqCst), 1);
}

#[test]
fn a_rotated_refresh_token_is_kept_so_the_next_renewal_still_works() {
    let home = tempfile::tempdir().unwrap();
    let kept = kept_with("r0");
    let renew = Arc::new(Rotating::lasting(Duration::from_secs(1)));
    let tokens = Tokens::new(kept.clone(), renew.clone(), KEY, home.path());

    assert_eq!(tokens.access().unwrap(), "a1");
    assert_eq!(tokens.access().unwrap(), "a2");
    assert_eq!(kept.read(KEY).unwrap().as_deref(), Some("r2"));
}

#[test]
fn processes_renewing_at_once_take_turns_and_none_uses_a_token_already_rotated() {
    let home = tempfile::tempdir().unwrap();
    let kept = kept_with("r0");
    let renew = Arc::new(Rotating {
        slow: Duration::from_millis(40),
        ..Rotating::lasting(Duration::from_secs(1))
    });

    let answers: Vec<Result<String, Hitch>> = std::thread::scope(|scope| {
        let all: Vec<_> = (0..4)
            .map(|_| {
                let tokens = Tokens::new(kept.clone(), renew.clone(), KEY, home.path());
                scope.spawn(move || tokens.access())
            })
            .collect();
        all.into_iter().map(|one| one.join().unwrap()).collect()
    });

    assert!(
        answers.iter().all(Result::is_ok),
        "a process renewed with a refresh token another had already rotated: {answers:?}"
    );
    assert_eq!(renew.renewed.load(Ordering::SeqCst), 4);
    assert_eq!(kept.read(KEY).unwrap().as_deref(), Some("r4"));
}

#[test]
fn with_no_refresh_token_kept_the_account_is_lost_until_it_is_connected_again() {
    let home = tempfile::tempdir().unwrap();
    let tokens = Tokens::new(
        Arc::new(Kept::default()),
        Arc::new(Rotating::lasting(Duration::from_secs(3600))),
        KEY,
        home.path(),
    );

    assert!(matches!(tokens.access(), Err(Hitch::Lost)));
}

#[test]
fn a_rotated_refresh_token_the_keychain_would_not_take_is_held_until_it_does() {
    let home = tempfile::tempdir().unwrap();
    let kept = kept_with("r0");
    let renew = Arc::new(Rotating::lasting(Duration::from_secs(1)));
    let tokens = Tokens::new(kept.clone(), renew.clone(), KEY, home.path());
    kept.1.store(true, Ordering::SeqCst);

    assert_eq!(
        tokens.access().unwrap(),
        "a1",
        "a fresh access token was thrown away"
    );
    assert_eq!(kept.read(KEY).unwrap().as_deref(), Some("r0"));

    kept.1.store(false, Ordering::SeqCst);
    assert_eq!(
        tokens.access().unwrap(),
        "a2",
        "the next renewal used a refresh token already spent"
    );
    assert_eq!(kept.read(KEY).unwrap().as_deref(), Some("r2"));
}

#[test]
fn an_access_token_the_provider_turned_down_is_renewed_on_the_next_ask() {
    let home = tempfile::tempdir().unwrap();
    let renew = Arc::new(Rotating::lasting(Duration::from_secs(3600)));
    let tokens = Tokens::new(kept_with("r0"), renew.clone(), KEY, home.path());
    let first = tokens.access().unwrap();

    tokens.spoiled(&first);

    assert_eq!(tokens.access().unwrap(), "a2");
    assert_eq!(renew.renewed.load(Ordering::SeqCst), 2);
}

#[test]
fn a_lifetime_no_clock_could_hold_does_not_bring_renewal_down() {
    let home = tempfile::tempdir().unwrap();
    let renew = Arc::new(Rotating::lasting(Duration::MAX));
    let tokens = Tokens::new(kept_with("r0"), renew, KEY, home.path());

    assert_eq!(tokens.access().unwrap(), "a1");
}

struct Marked;

impl Renew for Marked {
    fn renew(&self, _: &str) -> Result<Renewed, Hitch> {
        Ok(Renewed {
            access: "acceso-secreto".into(),
            lasts: Duration::ZERO,
            refresh: Some("renovado-secreto".into()),
        })
    }
}

fn every_file(at: &std::path::Path, found: &mut Vec<Vec<u8>>) {
    for one in std::fs::read_dir(at).unwrap().flatten() {
        match one.path().is_dir() {
            true => every_file(&one.path(), found),
            false => found.push(std::fs::read(one.path()).unwrap_or_default()),
        }
    }
}

#[test]
fn no_token_ever_lands_in_a_file_even_when_the_keychain_will_not_take_it() {
    let home = tempfile::tempdir().unwrap();
    let kept = kept_with("primero-secreto");
    let tokens = Tokens::new(kept.clone(), Arc::new(Marked), KEY, home.path());
    tokens.access().unwrap();
    kept.1.store(true, Ordering::SeqCst);
    tokens.access().unwrap();

    let mut found = Vec::new();
    every_file(home.path(), &mut found);

    assert!(
        !found.is_empty(),
        "nothing was left beside the locks to look into"
    );
    let secret = b"secreto";
    assert!(
        !found
            .iter()
            .any(|body| body.windows(secret.len()).any(|one| one == secret)),
        "a token was written beside the locks"
    );
}

#[test]
fn a_machine_that_let_go_holds_no_access_in_memory_either() {
    let home = tempfile::tempdir().unwrap();
    let kept = kept_with("r0");
    let renew = Arc::new(Rotating::lasting(Duration::from_secs(3600)));
    let tokens = Tokens::new(kept.clone(), renew, KEY, home.path());
    tokens.access().unwrap();

    kept.forget(KEY).unwrap();
    tokens.let_go();

    assert_eq!(tokens.access(), Err(Hitch::Lost));
}
