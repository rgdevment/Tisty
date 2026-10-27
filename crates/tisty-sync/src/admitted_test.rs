use super::*;
use std::path::Path;
use tisty_core::event::{DeviceId, TaskAdd};
use tisty_core::{Config, Op, Paths, Store};
use ulid::Ulid;

const CLOUD: &str = "store_cloud";

struct Computer {
    _dir: tempfile::TempDir,
    paths: Paths,
}

impl Computer {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().join("data"), dir.path().join("config"));
        std::fs::create_dir_all(paths.data()).unwrap();
        Self { _dir: dir, paths }
    }

    fn id(&self) -> DeviceId {
        Config::load_or_init(&self.paths).unwrap().device_id
    }

    fn shown(&self) -> String {
        tisty_core::signing::shown(&tisty_core::signing::mine(&self.paths, &self.id()).unwrap())
    }

    fn store(&self) -> Store {
        let id = self.id();
        Store::open(self.paths.store(), id.clone())
            .unwrap()
            .signing_with(tisty_core::signing::mine(&self.paths, &id))
    }

    fn writes(&self, title: &str) {
        self.store()
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(title, "a0"),
            })
            .unwrap();
    }

    fn round(&self, shared: &Path) -> Result<Moved, Trouble> {
        let moved = carry_telling(
            self.paths.data(),
            Some(self.paths.cache()),
            &self.id().0,
            shared,
            Way::Both,
            &[],
            Holds::Everywhere,
            &mut |_| {},
        )?;
        self.seated();
        Ok(moved)
    }

    fn seated(&self) {
        let who = self.id();
        let shown = self.shown();
        tisty_core::vouched::confirm(self.paths.data(), &who, &shown);
        let state = tisty_core::State::replay(&self.store().read_all().unwrap());
        if !state.keys.contains_key(&who) {
            tisty_core::vouched::confirm_each(self.paths.data(), &state.keys);
        }
        let op = match state.devices.contains(&who) {
            true if state.keys.contains_key(&who) => return,
            true => Op::DeviceKey { d: who, p: shown },
            false => Op::DeviceJoin {
                d: who,
                k: Some(tisty_core::DeviceKind::Machine),
                p: Some(shown),
            },
        };
        self.store().append(op).unwrap();
    }

    fn confirms(&self, other: &Computer) {
        assert!(tisty_core::vouched::confirm(
            self.paths.data(),
            &other.id(),
            &other.shown()
        ));
        turned::let_through(self.paths.data(), &other.id().0);
    }

    fn waits_on(&self, shared: &Path) -> Vec<String> {
        self.round(shared).unwrap().unconfirmed
    }

    fn takes_the_cloud_s(&self, shared: &Path, handed: Option<&str>) {
        let asked = self.round(shared);
        assert!(
            matches!(asked, Err(Trouble::OtherStore { .. })),
            "the choice never came: {asked:?}"
        );
        let renamed = !keeps_its_name(shared, handed, &self.id().0);
        let aside = self.paths.config().join("aside");
        let into = self.paths.config().join("before.zip");
        tisty_core::backup::reset_as(&self.paths, &into, &aside, None, renamed).unwrap();
    }
}

fn sealed(shared: &Path) {
    std::fs::write(
        shared.join(NAMED),
        format!("shape = 3\nneeds = \"1.35.0\"\nsealed = \"{CLOUD}\"\n"),
    )
    .unwrap();
}

fn two_already(shared: &Path) -> (Computer, Computer) {
    let giver = Computer::new();
    giver.writes("lo del primero");
    giver.round(shared).unwrap();
    let third = Computer::new();
    third.round(shared).unwrap();
    giver.confirms(&third);
    for _ in 0..2 {
        assert!(giver.waits_on(shared).is_empty());
        assert!(third.waits_on(shared).is_empty());
    }
    (giver, third)
}

fn quiet_twice(shared: &Path, all: &[&Computer]) {
    for _ in 0..2 {
        for one in all {
            assert_eq!(one.waits_on(shared), Vec::<String>::new());
        }
    }
}

#[test]
fn a_computer_let_in_by_its_digits_that_takes_the_cloud_s_history_is_not_asked_for_again() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.writes("lo mío");
    giver.confirms(&newcomer);
    let let_in = newcomer.id();

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    quiet_twice(shared.path(), &[&newcomer]);
    assert!(
        giver.waits_on(shared.path()).is_empty(),
        "the computer that took its digits asked for it again"
    );
    assert_eq!(
        third.waits_on(shared.path()),
        vec![newcomer.id().0],
        "a computer that never saw its digits answers for its key once"
    );
    third.confirms(&newcomer);
    quiet_twice(shared.path(), &[&newcomer, &giver, &third]);
    assert_eq!(newcomer.id(), let_in);
}

#[test]
fn a_computer_opened_with_the_phrase_that_takes_the_cloud_s_history_asks_nothing_of_its_own() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.writes("lo mío");
    let before = tempfile::tempdir().unwrap();
    newcomer.round(before.path()).unwrap();
    let typed = newcomer.id();

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    for _ in 0..2 {
        assert!(
            newcomer.waits_on(shared.path()).is_empty(),
            "the computer that typed the phrase asked to confirm the ones already there"
        );
    }
    for one in [&giver, &third] {
        assert_eq!(one.waits_on(shared.path()), vec![newcomer.id().0]);
        one.confirms(&newcomer);
    }
    quiet_twice(shared.path(), &[&newcomer, &giver, &third]);
    assert_eq!(newcomer.id(), typed);
}

#[test]
fn a_plain_cloud_still_meets_a_computer_that_takes_its_history_under_a_new_name() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, third) = two_already(shared.path());
    let newcomer = Computer::new();
    newcomer.writes("lo mío");
    let was = newcomer.id();

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    let now = newcomer.id();
    assert_ne!(now, was);
    quiet_twice(shared.path(), &[&newcomer]);
    for one in [&giver, &third] {
        assert_eq!(one.waits_on(shared.path()), vec![now.0.clone()]);
        one.confirms(&newcomer);
    }
    quiet_twice(shared.path(), &[&newcomer, &giver, &third]);
}

#[test]
fn a_name_the_encrypted_cloud_already_knows_is_never_kept() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, _third) = two_already(shared.path());
    sealed(shared.path());
    let gone = giver.id().0;

    assert!(!keeps_its_name(shared.path(), Some(CLOUD), &gone));
    assert!(!keeps_its_name(
        shared.path(),
        Some(CLOUD),
        &gone.to_uppercase()
    ));
    assert!(!keeps_its_name(shared.path(), None, "dev_new"));
    assert!(!keeps_its_name(
        shared.path(),
        Some("store_other"),
        "dev_new"
    ));
    assert!(keeps_its_name(shared.path(), Some(CLOUD), "dev_new"));
}
