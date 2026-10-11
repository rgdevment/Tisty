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
