use std::collections::BTreeMap;
use std::io::Write;

use tisty_carrier::{Costs, Expect, Hitch, Limits, Seen};
use tisty_core::Op;
use tisty_share_proto::seed::Seed;

use super::*;

const CLOUD: &str = "store_one";

#[derive(Default)]
struct Shelf(Mutex<BTreeMap<String, Vec<u8>>>);

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

fn key(n: u8) -> CloudKey {
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

fn here_of(session: &Mutex<Session>, plain: &Arc<Shelf>, home: &tempfile::TempDir) -> Here {
    let session = held(session);
    Here {
        paths: session.paths.clone(),
        device: session.config.device_id.clone(),
        cloud: CLOUD.into(),
        plain: plain.clone(),
        home: home.path().to_path_buf(),
    }
}

// The giver's log knows the asker as a seated machine with the key it signs with.
fn knows(giver: &Mutex<Session>, asker: &Here) {
    let key = tisty_core::signing::mine(&asker.paths, &asker.device).unwrap();
    let mut giver = held(giver);
    giver.state.devices.insert(asker.device.clone());
    giver.state.keys.insert(
        asker.device.clone(),
        tisty_core::signing::shown_of(&key.verifying_key()),
    );
}

fn sealed_with(plain: &Arc<Shelf>, key: &CloudKey) {
    tisty_carrier::Sealed::opened(plain.clone(), key.clone(), CLOUD, &key.named_for(CLOUD))
        .unwrap()
        .mark_it()
        .unwrap();
}

struct Two {
    plain: Arc<Shelf>,
    asker: Mutex<Session>,
    giver: Mutex<Session>,
    asking: Here,
    giving: Here,
    _kept: [tempfile::TempDir; 4],
}

fn two() -> Two {
    let kept = std::array::from_fn(|_| tempfile::tempdir().unwrap());
    let plain = Arc::new(Shelf::default());
    let (asker, giver) = (machine(&kept[0]), machine(&kept[1]));
    let (asking, giving) = (
        here_of(&asker, &plain, &kept[2]),
        here_of(&giver, &plain, &kept[3]),
    );
    Two {
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
    sealed_with(&two.plain, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();

    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    answering(&two.giver, &Knocking::default(), &two.giving, &key(7)).unwrap();
    assert!(asking(&two.asker, &knocking, &two.asking).unwrap());

    assert_eq!(
        held(&two.asker).config.key_handed,
        Some(key(7).named_for(CLOUD))
    );
    assert_eq!(
        kept(&two.asking.paths)
            .unwrap()
            .map(|one| one.named_for(CLOUD)),
        Some(key(7).named_for(CLOUD))
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
    sealed_with(&two.plain, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();
    asking(&two.asker, &knocking, &two.asking).unwrap();

    answering(&two.giver, &Knocking::default(), &two.giving, &key(4)).unwrap();

    assert!(!asking(&two.asker, &knocking, &two.asking).unwrap());
    assert_eq!(held(&two.asker).config.key_handed, None);
    assert!(kept(&two.asking.paths).unwrap().is_none());
}

#[test]
fn only_a_seated_machine_asking_with_its_own_key_is_answered_on_its_own() {
    let two = two();
    sealed_with(&two.plain, &key(7));
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
        CLOUD,
        now(),
    );
    knows(&two.giver, &two.asking);
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &forged).unwrap();
    answering(&two.giver, &Knocking::default(), &two.giving, &key(7)).unwrap();

    assert_eq!(
        tisty_carrier::knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0)
            .unwrap(),
        None
    );
}

#[test]
fn an_ask_past_its_time_is_taken_away_rather_than_answered() {
    let two = two();
    sealed_with(&two.plain, &key(7));
    knows(&two.giver, &two.asking);
    let signer = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    let old = Ask::signed(
        &two.asking.device.0,
        "PC",
        &signer,
        &OneUse::minted().unwrap(),
        CLOUD,
        now() - JOINED_FOR - 60,
    );
    tisty_carrier::knock::knocked(&*two.plain, &two.asking.home, &old).unwrap();

    answering(&two.giver, &Knocking::default(), &two.giving, &key(7)).unwrap();

    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_machine_with_the_key_looks_at_the_box_at_most_every_five_minutes() {
    let two = two();
    sealed_with(&two.plain, &key(7));
    knows(&two.giver, &two.asking);
    let looking = Knocking::default();
    answering(&two.giver, &looking, &two.giving, &key(7)).unwrap();
    asking(&two.asker, &Knocking::default(), &two.asking).unwrap();

    answering(&two.giver, &looking, &two.giving, &key(7)).unwrap();

    assert_eq!(
        tisty_carrier::knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0)
            .unwrap(),
        None
    );
}
