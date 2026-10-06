use super::*;
use crate::event::{DeviceId, TaskAdd};
use crate::{Error, Op, Store};
use std::time::{Duration, SystemTime};

struct Home {
    _dir: tempfile::TempDir,
    roots: Roots,
}

fn home(private: bool) -> Home {
    let dir = tempfile::tempdir().unwrap();
    let roots = Roots {
        new: dir.path().join("profile").join(".keep"),
        real: dir.path().join("local").join("kept"),
        private: private.then(|| dir.path().join("package").join("kept")),
    };
    Home { _dir: dir, roots }
}

fn wrote(root: &Path, title: &str) {
    let mut store = Store::open(root.join("data").join("store"), DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        })
        .unwrap();
}

fn put(root: &Path, at: &str, body: &str) {
    let at = root.join(at);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, body).unwrap();
}

fn aged(root: &Path, by: Duration) {
    let active = root.join("data/store/dev_a/active.tisty");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(active)
        .unwrap();
    file.set_modified(SystemTime::now() - by).unwrap();
}

fn read(root: &Path, at: &str) -> Option<String> {
    std::fs::read_to_string(root.join(at)).ok()
}

#[test]
fn a_machine_that_never_ran_tisty_has_nothing_to_move() {
    let home = home(true);

    assert_eq!(settle(&home.roots), Settled::Fresh);
    assert!(!home.roots.new.exists());
}

#[test]
fn a_loose_install_moves_whole_and_leaves_its_folder_as_it_was() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    put(&home.roots.real, "config/private/key", "secret");

    assert_eq!(settle(&home.roots), Settled::Moved { aside: None });

    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("secret")
    );
    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("pay the bill")
    );
    assert!(read(&home.roots.real, "config/private/key").is_some());
    assert!(home.roots.real.join("MOVED.txt").is_file());
}

#[test]
fn the_store_copy_is_laid_over_the_real_one_file_by_file() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&home.roots.real, "only in the real folder");
    put(
        &home.roots.real,
        "config/private/key",
        "the key nobody rewrote",
    );
    put(&home.roots.real, "data/docs/both.md", "old body");
    aged(&home.roots.real, Duration::from_secs(3600));
    wrote(&private, "written by the store app");
    put(&private, "data/docs/both.md", "new body");

    let Settled::Moved { aside: Some(aside) } = settle(&home.roots) else {
        panic!("both copies were not kept");
    };

    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("the key nobody rewrote"),
        "what only the real folder held was lost"
    );
    assert_eq!(
        read(&home.roots.new, "data/docs/both.md").as_deref(),
        Some("new body")
    );
    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("written by the store app")
    );
    assert_eq!(
        read(&aside, "data/docs/both.md").as_deref(),
        Some("old body")
    );
}

#[test]
fn a_real_folder_written_after_the_store_one_leads_and_the_store_copy_is_kept_aside() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&private, "from the store app");
    aged(&private, Duration::from_secs(3600));
    wrote(&home.roots.real, "from the loose install, later");

    let Settled::Moved { aside: Some(aside) } = settle(&home.roots) else {
        panic!("the store copy was not kept aside");
    };

    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("from the loose install, later")
    );
    assert!(
        read(&aside, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("from the store app")
    );
}

#[test]
fn an_older_tisty_refuses_the_folder_that_was_left_behind() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");

    settle(&home.roots);

    assert!(matches!(
        crate::store::read_all(home.roots.real.join("data/store")),
        Err(Error::UnsupportedVersion { .. })
    ));
    assert!(crate::store::read_all(home.roots.new.join("data/store")).is_ok());
}

#[test]
fn a_second_start_finds_the_move_done_and_fences_what_was_missed() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    std::fs::create_dir_all(&home.roots.new).unwrap();

    assert_eq!(settle(&home.roots), Settled::AlreadyThere);
    assert!(home.roots.real.join("MOVED.txt").is_file());
    assert!(read(&home.roots.new, "data/store/dev_a/active.tisty").is_none());

    let fenced = read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap();
    settle(&home.roots);
    assert_eq!(
        read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap(),
        fenced,
        "the fence was written twice"
    );
}

#[test]
fn what_a_cut_short_move_left_is_swept_and_never_copied() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    put(&home.roots.real, "data/store/dev_a/.lock", "");
    put(&home.roots.real, "data/docs/half.md.tmp", "half");
    let parent = home.roots.new.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(parent.join(".keep.part-1")).unwrap();

    settle(&home.roots);

    assert!(!parent.join(".keep.part-1").exists());
    assert!(!home.roots.new.join("data/store/dev_a/.lock").exists());
    assert!(!home.roots.new.join("data/docs/half.md.tmp").exists());
}
