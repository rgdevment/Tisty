use std::collections::BTreeMap;
use std::io::Write;

use tisty_carrier::{Costs, Expect, Hitch, Limits, Seen};
use tisty_core::Op;
use tisty_share_proto::seed::Seed;

use super::*;

#[derive(Default)]
pub(crate) struct Shelf(Mutex<BTreeMap<String, Vec<u8>>>);

fn seen(name: &str, bytes: &[u8]) -> Seen {
    Seen {
        name: name.to_string(),
        bytes: bytes.len() as u64,
        hash: format!("{:x}", bytes.len()),
        revision: format!("{name}:{}", bytes.len()),
    }
}

impl Remote for Shelf {
    fn limits(&self) -> Limits {
        Limits {
            provider: "shelf",
            units_a_day: u64::MAX,
            bytes_a_day: u64::MAX,
            poll_every: Duration::from_secs(60),
            chunk: 1 << 20,
            page: 100,
            most_per_file: u64::MAX,
            folds_case: false,
            costs: Costs {
                list: 0,
                fetch: 0,
                put: 0,
                delete: 0,
                about: 0,
                changes: 0,
                checks: 0,
            },
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let held = self.0.lock().unwrap();
        Ok(held
            .iter()
            .filter(|(name, _)| under.is_empty() || name.starts_with(&format!("{under}/")))
            .map(|(name, bytes)| seen(name, bytes))
            .collect())
    }

    fn fetch(&self, name: &str, _: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        let held = self.0.lock().unwrap();
        let bytes = held.get(name).ok_or_else(|| Hitch::Missing(name.into()))?;
        into.write_all(bytes)
            .map_err(|e| Hitch::Broke(e.to_string()))?;
        Ok(seen(name, bytes))
    }

    fn put(&self, name: &str, from: &std::path::Path, expect: Expect) -> Result<Seen, Hitch> {
        let bytes = std::fs::read(from).map_err(|e| Hitch::Broke(e.to_string()))?;
        let mut held = self.0.lock().unwrap();
        if matches!(expect, Expect::Absent) && held.contains_key(name) {
            return Err(Hitch::Changed(name.into()));
        }
        let told = seen(name, &bytes);
        held.insert(name.to_string(), bytes);
        Ok(told)
    }

    fn delete(&self, name: &str, _: Option<&str>) -> Result<(), Hitch> {
        self.0.lock().unwrap().remove(name);
        Ok(())
    }

    fn hash_of(&self, local: &std::path::Path) -> std::io::Result<String> {
        Ok(format!("{:x}", std::fs::read(local)?.len()))
    }
}

pub(crate) fn key(n: u8) -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[n; 32]))
}

fn machine(kept: &tempfile::TempDir) -> Mutex<Session> {
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
    Mutex::new(session)
}

fn here_of(
    session: &Mutex<Session>,
    plain: &Arc<Shelf>,
    home: &tempfile::TempDir,
    cloud: &str,
) -> Here {
    let session = held(session);
    Here {
        paths: session.paths.clone(),
        device: session.config.device_id.clone(),
        cloud: cloud.into(),
        plain: plain.clone(),
        home: home.path().to_path_buf(),
    }
}

pub(crate) fn knows(giver: &Mutex<Session>, asker: &Here) {
    let key = tisty_core::signing::mine(&asker.paths, &asker.device).unwrap();
    let mut giver = held(giver);
    giver.state.devices.insert(asker.device.clone());
    giver.state.keys.insert(
        asker.device.clone(),
        tisty_core::signing::shown_of(&key.verifying_key()),
    );
}

pub(crate) fn sealed_with(two: &Two, key: &CloudKey) {
    let cloud = two.cloud.as_str();
    tisty_carrier::Sealed::opened(two.plain.clone(), key.clone(), cloud, &key.named_for(cloud))
        .unwrap()
        .mark_it()
        .unwrap();
}

pub(crate) struct Two {
    pub(crate) cloud: String,
    pub(crate) plain: Arc<Shelf>,
    pub(crate) asker: Mutex<Session>,
    pub(crate) giver: Mutex<Session>,
    pub(crate) asking: Here,
    pub(crate) giving: Here,
    pub(crate) _kept: [tempfile::TempDir; 4],
}

pub(crate) fn two() -> Two {
    let kept = std::array::from_fn(|_| tempfile::tempdir().unwrap());
    let plain = Arc::new(Shelf::default());
    // The keychain of the tests is one for the whole run, so each test seals a cloud of its own.
    let cloud = format!(
        "store_{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    );
    let (asker, giver) = (machine(&kept[0]), machine(&kept[1]));
    let (asking, giving) = (
        here_of(&asker, &plain, &kept[2], &cloud),
        here_of(&giver, &plain, &kept[3], &cloud),
    );
    Two {
        cloud,
        plain,
        asker,
        giver,
        asking,
        giving,
        _kept: kept,
    }
}

#[test]
fn a_machine_on_the_list_asks_and_takes_the_key_another_hands_it() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    knows(&two.asker, &two.giving);
    let knocking = Knocking::default();

    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    answering(&two.giver, &two.giving, &key(7)).unwrap();
    assert!(asking(&two.asker, &knocking, &two.asking).unwrap());

    assert_eq!(held(&two.asker).config.key_handed, Some(two.cloud.clone()));
    assert_eq!(
        kept(&two.cloud)
            .unwrap()
            .map(|one| one.named_for(&two.cloud)),
        Some(key(7).named_for(&two.cloud))
    );
    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_key_that_does_not_open_the_cloud_s_mark_is_never_taken() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();
    asking(&two.asker, &knocking, &two.asking).unwrap();

    answering(&two.giver, &two.giving, &key(4)).unwrap();

    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    assert_eq!(held(&two.asker).config.key_handed, None);
    assert!(kept(&two.cloud).unwrap().is_none());
}

#[test]
fn only_a_seated_machine_asking_with_its_own_key_is_answered_on_its_own() {
    let two = two();
    sealed_with(&two, &key(7));
    let knocking = Knocking::default();
    held(&two.asker).state.devices.clear();
    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty(),
        "a machine off the list asked on its own"
    );

    let forged = Ask::signed(
        &two.asking.device.0,
        "Impostor",
        &SigningKey::from_bytes(&[3; 32]),
        &OneUse::minted().unwrap(),
        &two.cloud,
        now(),
    );
    knows(&two.giver, &two.asking);
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &forged).unwrap();
    answering(&two.giver, &two.giving, &key(7)).unwrap();

    assert_eq!(
        tisty_carrier::knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0)
            .unwrap(),
        None
    );
}

#[test]
fn an_ask_past_its_time_is_taken_away_rather_than_answered() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let signer = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    let old = Ask::signed(
        &two.asking.device.0,
        "PC",
        &signer,
        &OneUse::minted().unwrap(),
        &two.cloud,
        now() - JOINED_FOR - 60,
    );
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &old).unwrap();

    answering(&two.giver, &two.giving, &key(7)).unwrap();

    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_box_is_looked_at_only_once_its_time_came_again() {
    let knocking = Knocking::default();

    assert!(due(&knocking, ANSWERED_EVERY));
    assert!(!due(&knocking, ANSWERED_EVERY));
    assert!(due(&knocking, Duration::ZERO));
}

#[test]
fn an_answer_that_does_not_open_is_taken_away_so_another_can_come() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    knows(&two.asker, &two.giving);
    let knocking = Knocking::default();
    asking(&two.asker, &knocking, &two.asking).unwrap();
    answering(&two.giver, &two.giving, &key(4)).unwrap();
    asking(&two.asker, &knocking, &two.asking).unwrap();

    answering(&two.giver, &two.giving, &key(7)).unwrap();

    assert!(asking(&two.asker, &knocking, &two.asking).unwrap());
}

fn signed_at(two: &Two, at: i64, one_use: &OneUse) -> Ask {
    let signer = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    Ask::signed(&two.asking.device.0, "PC", &signer, one_use, &two.cloud, at)
}

#[test]
fn an_ask_left_unanswered_for_an_hour_is_made_again() {
    let two = two();
    let knocking = Knocking::default();
    let one_use = OneUse::minted().unwrap();
    *knocking.asked.lock().unwrap() =
        Some((signed_at(&two, now() - ASKED_AGAIN - 60, &one_use), one_use));

    asking(&two.asker, &knocking, &two.asking).unwrap();

    let up_there = tisty_carrier::knock::knocks(&*two.plain, &two.asking.home).unwrap();
    assert_eq!(up_there.len(), 1);
    assert!(now() - up_there[0].ask.as_ref().unwrap().at < 60);
}

#[test]
fn an_ask_from_a_clock_running_ahead_is_answered_and_one_from_far_ahead_is_taken_away() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let one_use = OneUse::minted().unwrap();
    let ahead = signed_at(&two, now() + 10 * 60, &one_use);
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &ahead).unwrap();

    answering(&two.giver, &two.giving, &key(7)).unwrap();
    assert!(
        tisty_carrier::knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0)
            .unwrap()
            .is_some()
    );

    let far = signed_at(&two, now() + JOINED_FOR + 60, &OneUse::minted().unwrap());
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &far).unwrap();
    answering(&two.giver, &two.giving, &key(7)).unwrap();
    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_flag_is_let_go_of_even_when_the_work_under_it_falls_over() {
    let busy = AtomicBool::new(true);

    let fell = std::panic::catch_unwind(|| {
        let _free = Free(&busy);
        panic!("the work fell over");
    });

    assert!(fell.is_err());
    assert!(!busy.load(Ordering::Acquire));
}

#[test]
fn a_key_from_a_machine_this_one_does_not_know_is_never_taken() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();
    asking(&two.asker, &knocking, &two.asking).unwrap();

    answering(&two.giver, &two.giving, &key(7)).unwrap();

    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    assert!(kept(&two.cloud).unwrap().is_none());
    assert_eq!(
        tisty_carrier::knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0)
            .unwrap(),
        None
    );
}

#[test]
fn a_machine_whose_log_is_behind_takes_the_key_from_one_its_person_certified() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let seed = tisty_share_proto::seed::Seed::from_bytes(&[7; 32]);
    crate::identity::begun(&mut held(&two.giver), &seed).unwrap();
    let persons = held(&two.giver).state.persons.clone();
    held(&two.asker).state.persons = persons;
    let knocking = Knocking::default();
    asking(&two.asker, &knocking, &two.asking).unwrap();

    answering(&two.giver, &two.giving, &key(7)).unwrap();

    assert!(asking(&two.asker, &knocking, &two.asking).unwrap());
    assert_eq!(held(&two.asker).config.key_handed, Some(two.cloud.clone()));
}

#[test]
fn letting_go_forgets_the_handed_key_and_what_was_asked() {
    let two = two();
    sealed_with(&two, &key(7));
    kept_for(&two.asker, &two.cloud, &key(7)).unwrap();
    let knocking = Knocking::default();
    ask_now(&two.asker, &mut knocking.asked.lock().unwrap(), &two.asking).unwrap();

    let_go(&two.asker, &knocking);

    assert!(kept(&two.cloud).unwrap().is_none());
    assert_eq!(held(&two.asker).config.key_handed, None);
    assert!(knocking.asked.lock().unwrap().is_none());
}

#[test]
fn a_key_handed_for_a_new_cloud_lets_the_old_one_go() {
    let two = two();
    kept_for(&two.asker, "store_old_cloud", &key(4)).unwrap();

    kept_for(&two.asker, &two.cloud, &key(7)).unwrap();

    assert!(kept("store_old_cloud").unwrap().is_none());
    assert_eq!(held(&two.asker).config.key_handed, Some(two.cloud.clone()));
}

#[test]
fn forgetting_another_account_keeps_the_key_of_the_cloud_this_machine_syncs_with() {
    let two = two();
    let at = |name: &str| tisty_core::config::CloudAt {
        provider: tisty_core::config::WEBDAV.into(),
        account: format!("https://{name}@cloud.example.com/dav/Tisty/"),
        email: format!("{name}@cloud.example.com"),
    };
    kept_for(&two.asker, &two.cloud, &key(7)).unwrap();
    held(&two.asker).config.sync = Some(tisty_core::config::Sync::Cloud(at("mario")));
    let knocking = Knocking::default();

    let_go_of(&two.asker, &knocking, &at("otra"));
    assert!(kept(&two.cloud).unwrap().is_some());

    let_go_of(&two.asker, &knocking, &at("mario"));
    assert!(kept(&two.cloud).unwrap().is_none());
}
