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

    fn titles(&self) -> Vec<String> {
        tisty_core::State::replay(&self.store().read_all().unwrap())
            .tasks
            .into_values()
            .map(|task| task.title)
            .collect()
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

    fn opened(&self) -> &Self {
        self.seated();
        tisty_core::agent::register(&self.paths).unwrap();
        self
    }

    fn confirms(&self, other: &Computer) {
        assert!(tisty_core::vouched::confirm(
            self.paths.data(),
            &other.id(),
            &other.shown()
        ));
        turned::let_through(self.paths.data(), &other.id().0);
    }

    fn lets_in(&self, other: &Computer) {
        assert!(tisty_core::vouched::asked(
            self.paths.data(),
            &other.id(),
            &other.shown()
        ));
        turned::let_through(self.paths.data(), &other.id().0);
    }

    fn waits_on(&self, shared: &Path) -> Vec<String> {
        let mut waits = self.round(shared).unwrap().unconfirmed;
        waits.sort();
        waits
    }

    fn settles(&self, shared: &Path) {
        for _ in 0..2 {
            self.round(shared).unwrap();
        }
    }

    fn takes_the_cloud_s(&self, shared: &Path, handed: Option<&str>) -> DeviceId {
        let asked = self.round(shared);
        assert!(
            matches!(asked, Err(Trouble::OtherStore { .. })),
            "the choice never came: {asked:?}"
        );
        let was = self.id();
        let aside = self.paths.config().join("aside");
        let into = self.paths.config().join("before.zip");
        tisty_core::backup::reset_as(&self.paths, &into, &aside, None, let_in(shared, handed))
            .unwrap();
        assert_ne!(
            self.id(),
            was,
            "it took the cloud's history under its old name"
        );
        was
    }
}

fn sealed(shared: &Path) {
    seal_word(shared, CLOUD).unwrap();
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
    newcomer.opened().writes("lo mío");
    giver.lets_in(&newcomer);

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    quiet_twice(shared.path(), &[&newcomer]);
    assert!(newcomer.titles().contains(&"lo del primero".to_string()));
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
}

#[test]
fn a_computer_opened_with_the_phrase_that_takes_the_cloud_s_history_asks_nothing_of_its_own() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    quiet_twice(shared.path(), &[&newcomer]);
    for one in [&giver, &third] {
        assert_eq!(one.waits_on(shared.path()), vec![newcomer.id().0]);
        one.confirms(&newcomer);
    }
    quiet_twice(shared.path(), &[&newcomer, &giver, &third]);
}

#[test]
fn a_computer_let_in_meets_the_place_it_synced_with_before_as_a_stranger() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, _third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");
    let before = tempfile::tempdir().unwrap();
    newcomer.round(before.path()).unwrap();
    giver.lets_in(&newcomer);

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));
    newcomer.round(shared.path()).unwrap();
    newcomer.writes("en la nube");
    newcomer.round(shared.path()).unwrap();

    assert!(matches!(
        kinship(&newcomer.paths.store(), before.path()),
        Kin::Strangers
    ));
}

#[test]
fn the_digits_let_in_one_new_name_and_never_a_second() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, _third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");
    giver.lets_in(&newcomer);
    let was = newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));
    newcomer.settles(shared.path());
    assert!(giver.waits_on(shared.path()).is_empty());

    let copy = Computer::new();
    let key = tisty_core::signing::kept_at(&newcomer.paths, &was).unwrap();
    let at = tisty_core::signing::kept_at(&copy.paths, &copy.id()).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::copy(key, &at).unwrap();
    copy.settles(shared.path());

    assert_eq!(giver.waits_on(shared.path()), vec![copy.id().0]);
}

#[test]
fn a_computer_the_person_took_away_is_no_pass_for_a_new_name() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, _third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");
    giver.lets_in(&newcomer);
    giver
        .store()
        .append(Op::DeviceRemove { d: newcomer.id() })
        .unwrap();

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));
    newcomer.settles(shared.path());

    assert_eq!(giver.waits_on(shared.path()), vec![newcomer.id().0]);
}

#[test]
fn a_key_confirmed_by_hand_is_no_pass_for_a_new_name() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, _third) = two_already(shared.path());
    sealed(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");
    giver.confirms(&newcomer);

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));
    newcomer.settles(shared.path());

    assert_eq!(giver.waits_on(shared.path()), vec![newcomer.id().0]);
}

#[test]
fn a_plain_cloud_still_meets_a_computer_that_takes_its_history_as_a_new_one() {
    let shared = tempfile::tempdir().unwrap();
    let (giver, third) = two_already(shared.path());
    let newcomer = Computer::new();
    newcomer.opened().writes("lo mío");
    giver.lets_in(&newcomer);
    let was = newcomer.shown();

    newcomer.takes_the_cloud_s(shared.path(), Some(CLOUD));

    assert_ne!(newcomer.shown(), was);
    let mut there = vec![giver.id().0, third.id().0];
    there.sort();
    for _ in 0..2 {
        assert_eq!(newcomer.waits_on(shared.path()), there);
    }
    for one in [&giver, &third] {
        assert_eq!(one.waits_on(shared.path()), vec![newcomer.id().0]);
    }
}

#[test]
fn only_the_cloud_whose_key_was_handed_lets_a_computer_in() {
    let shared = tempfile::tempdir().unwrap();
    Computer::new().round(shared.path()).unwrap();
    assert!(!let_in(shared.path(), Some(CLOUD)));
    sealed(shared.path());

    assert!(let_in(shared.path(), Some(CLOUD)));
    assert!(!let_in(shared.path(), None));
    assert!(!let_in(shared.path(), Some("store_other")));
}
