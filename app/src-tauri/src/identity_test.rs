use super::*;
use tisty_core::config::{CloudAt, Sync};

fn seed(byte: u8) -> Seed {
    Seed::from_bytes(&[byte; 32])
}

fn account(name: &str) -> CloudAt {
    CloudAt {
        provider: tisty_core::config::WEBDAV.into(),
        account: format!("https://{name}@cloud.example.com/dav/Tisty/"),
        email: format!("{name}@cloud.example.com"),
    }
}

fn store_name() -> String {
    format!(
        "store_{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    )
}

fn opened(kept: &tempfile::TempDir) -> Mutex<Session> {
    let paths = Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let here = session.config.device_id.clone();
    session
        .commit(Op::DeviceJoin {
            d: here,
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        })
        .unwrap();
    session.config.sync = Some(Sync::Cloud(account("mario")));
    Mutex::new(session)
}

fn on(session: &Mutex<Session>, at: CloudAt) {
    held(session).config.sync = Some(Sync::Cloud(at));
}

fn sealed_up_there(session: &Mutex<Session>, cloud: &str) {
    let session = held(session);
    let at = tisty_carrier::cloud_of(session.config.sync.as_ref())
        .unwrap()
        .clone();
    let tree = tisty_carrier::mirrored_at(&session.reach.home_of(&at));
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(
        tree.join("tisty.toml"),
        format!("shape = 3\nneeds = \"1.35.0\"\nsealed = \"{cloud}\"\n"),
    )
    .unwrap();
}

fn said_sealed(session: &Mutex<Session>, cloud: &str, by: &Seed) -> String {
    let named = CloudKey::of(by).named_for(cloud);
    held(session)
        .commit(Op::CloudSealed {
            s: cloud.to_string(),
            k: Some(named.clone()),
        })
        .unwrap();
    named
}

fn sealed_now(session: &Mutex<Session>) -> Option<Locked> {
    held(session).reach.sealed()
}

#[test]
fn a_sealed_cloud_is_keyed_when_this_machine_holds_its_phrase() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    let named = said_sealed(&session, "cloud_one", &seed(7));
    sealed_up_there(&session, "cloud_one");

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(
        sealed_now(&session),
        Some(Locked {
            at: account("mario"),
            cloud: "cloud_one".into(),
            named,
        })
    );
    assert!(held(&session).reach.tried().is_none());
}

#[test]
fn without_the_phrase_the_cloud_is_read_as_it_stands_and_the_try_is_remembered() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    said_sealed(&session, "cloud_one", &seed(7));
    sealed_up_there(&session, "cloud_one");

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(sealed_now(&session), None);
    let (tried, _) = held(&session).reach.tried().unwrap();
    assert_eq!(tried.cloud, "cloud_one");
}

#[test]
fn another_account_is_worked_out_again_rather_than_read_with_the_last_one_key() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    said_sealed(&session, "cloud_one", &seed(7));
    sealed_up_there(&session, "cloud_one");
    tauri::async_runtime::block_on(seal_the_cloud(&session));

    on(&session, account("otra"));
    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(
        sealed_now(&session).map(|one| one.at),
        Some(account("otra"))
    );
}

#[test]
fn a_cloud_the_log_no_longer_names_sealed_is_read_plain_again() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    said_sealed(&session, "cloud_one", &seed(7));
    sealed_up_there(&session, "cloud_one");
    tauri::async_runtime::block_on(seal_the_cloud(&session));

    held(&session)
        .commit(Op::CloudSealed {
            s: "cloud_one".into(),
            k: None,
        })
        .unwrap();
    let tree = {
        let session = held(&session);
        tisty_carrier::mirrored_at(&session.reach.home_of(&account("mario")))
    };
    std::fs::remove_file(tree.join("tisty.toml")).unwrap();
    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(sealed_now(&session), None);
}

#[test]
fn a_seed_kept_under_a_store_this_one_took_in_is_brought_under_its_name() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    let paths = held(&session).paths.clone();
    let old = store_name();
    keep_at(&format!("identity-{old}"), &seed(7)).unwrap();

    let found = kept_seed(&paths, &old, &|_| true);

    assert_eq!(*found.unwrap().to_bytes(), *seed(7).to_bytes());
    assert_eq!(
        *kept_at(&scope(&paths).unwrap())
            .unwrap()
            .unwrap()
            .to_bytes(),
        *seed(7).to_bytes()
    );
}

#[test]
fn a_seed_under_a_store_taken_in_that_does_not_fit_is_left_where_it_is() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    let paths = held(&session).paths.clone();
    let old = store_name();
    keep_at(&format!("identity-{old}"), &seed(8)).unwrap();

    assert!(kept_seed(&paths, &old, &|_| false).is_none());
    assert!(kept_at(&scope(&paths).unwrap()).unwrap().is_none());
}

#[test]
fn this_store_own_seed_comes_first_when_it_fits() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    let paths = held(&session).paths.clone();
    keep(&paths, &seed(7)).unwrap();
    let old = store_name();
    keep_at(&format!("identity-{old}"), &seed(8)).unwrap();

    let found = kept_seed(&paths, &old, &|_| true);

    assert_eq!(*found.unwrap().to_bytes(), *seed(7).to_bytes());
}

fn kept_seed(paths: &Paths, forebear: &str, fits: &dyn Fn(&Seed) -> bool) -> Option<Seed> {
    let forebears = BTreeSet::from([forebear.to_string()]);
    kept(paths, &forebears, fits).unwrap()
}

#[test]
fn a_new_address_on_the_same_account_is_the_same_sealed_cloud() {
    let locked = |email: &str| Locked {
        at: CloudAt {
            email: email.into(),
            ..account("mario")
        },
        cloud: "cloud_one".into(),
        named: "key_a".into(),
    };

    assert_eq!(
        locked("mario@cloud.example.com"),
        locked("otro@cloud.example.com")
    );
    assert_ne!(
        locked("mario@cloud.example.com"),
        Locked {
            at: account("otra"),
            ..locked("mario@cloud.example.com")
        }
    );
}

#[test]
fn the_keychain_is_never_read_before_an_identity_exists() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    let asked = scope(&held(&session).paths).unwrap();
    let before = crate::reaching::reads_of(&asked);

    assert!(seed_here(&session).unwrap().is_none());
    assert!(seed_person(&session).unwrap().is_none());

    assert_eq!(crate::reaching::reads_of(&asked), before);
}

#[test]
fn once_an_identity_exists_its_seed_is_read_once_until_it_is_begun_or_typed_again() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    begun(&mut held(&session), &seed(7)).unwrap();
    let asked = scope(&held(&session).paths).unwrap();
    let before = crate::reaching::reads_of(&asked);

    let first = seed_person(&session).unwrap();
    let again = seed_person(&session).unwrap();

    assert_eq!(first, Some(seed(7).root().person()));
    assert_eq!(again, first);
    assert_eq!(crate::reaching::reads_of(&asked) - before, 1);
    held(&session).reach.seed_is(None);
    seed_person(&session).unwrap();
    assert_eq!(crate::reaching::reads_of(&asked) - before, 2);
}

#[test]
fn a_machine_that_never_read_the_event_keys_the_cloud_with_its_own_phrase() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    begun(&mut held(&session), &seed(7)).unwrap();
    sealed_up_there(&session, "cloud_one");

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(
        sealed_now(&session).map(|one| one.named),
        Some(CloudKey::of(&seed(7)).named_for("cloud_one"))
    );
}

#[test]
fn without_an_identity_a_sealed_cloud_never_asks_the_keychain_for_a_phrase() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    sealed_up_there(&session, "cloud_one");
    let asked = scope(&held(&session).paths).unwrap();
    let before = crate::reaching::reads_of(&asked);

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(sealed_now(&session), None);
    assert_eq!(crate::reaching::reads_of(&asked), before);
}

fn word_here(session: &Mutex<Session>, word: &str) {
    let tree = {
        let session = held(session);
        tisty_carrier::mirrored_at(&session.reach.home_of(&account("mario")))
    };
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(tree.join("tisty.toml"), word).unwrap();
}

#[test]
fn a_place_that_says_plain_while_the_log_names_it_sealed_is_still_read_sealed() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    said_sealed(&session, "cloud_one", &seed(7));
    word_here(&session, "shape = 3\n");

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(
        sealed_now(&session).map(|one| one.cloud),
        Some("cloud_one".into())
    );
}

#[test]
fn a_cloud_still_being_sealed_is_read_plain_even_once_the_log_names_its_key() {
    let kept = tempfile::tempdir().unwrap();
    let session = opened(&kept);
    keep(&held(&session).paths, &seed(7)).unwrap();
    said_sealed(&session, "cloud_one", &seed(7));
    word_here(&session, "shape = 3\nsealing = \"copying:seal:dev_a:1\"\n");

    tauri::async_runtime::block_on(seal_the_cloud(&session));

    assert_eq!(sealed_now(&session), None);
}
