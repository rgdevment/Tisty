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

fn wrote_as(root: &Path, device: &str, title: &str) {
    let mut store = Store::open(root.join("data").join("store"), DeviceId(device.into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        })
        .unwrap();
}

fn wrote(root: &Path, title: &str) {
    wrote_as(root, "dev_a", title);
    put(root, "config/config.toml", "device_id = \"dev_a\"\n");
}

fn put(root: &Path, at: &str, body: &str) {
    let at = root.join(at);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, body).unwrap();
}

fn aged(root: &Path, device: &str, by: Duration) {
    let active = root.join(format!("data/store/{device}/active.tisty"));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(active)
        .unwrap();
    file.set_modified(SystemTime::now() - by).unwrap();
}

fn read(root: &Path, at: &str) -> Option<String> {
    std::fs::read_to_string(root.join(at)).ok()
}

fn readable(root: &Path) -> bool {
    crate::store::read_all(root.join("data/store")).is_ok()
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
fn only_the_data_and_the_settings_move_never_the_cache_nor_a_program_beside_them() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    put(&home.roots.real, "cache/read.db", "rebuilt anyway");
    put(&home.roots.real, "Tisty.exe", "an old install lived here");

    settle(&home.roots);

    assert!(!home.roots.new.join("cache").exists());
    assert!(!home.roots.new.join("Tisty.exe").exists());
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
    aged(&home.roots.real, "dev_a", Duration::from_secs(3600));
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
    aged(&private, "dev_a", Duration::from_secs(3600));
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
fn another_machine_just_carried_in_does_not_decide_which_copy_leads() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&home.roots.real, "this machine, a while ago");
    aged(&home.roots.real, "dev_a", Duration::from_secs(3600));
    wrote_as(&home.roots.real, "dev_laptop", "carried in a moment ago");
    wrote(&private, "this machine, in the store app");
    aged(&private, "dev_a", Duration::from_secs(60));

    settle(&home.roots);

    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("this machine, in the store app")
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
    assert!(readable(&home.roots.new));
}

#[test]
fn a_root_that_was_made_without_moving_fences_nothing_it_never_copied() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    std::fs::create_dir_all(&home.roots.new).unwrap();

    assert_eq!(settle(&home.roots), Settled::AlreadyThere);

    assert!(
        readable(&home.roots.real),
        "a store nobody copied was fenced off"
    );
    assert!(!home.roots.real.join("MOVED.txt").exists());
}

#[test]
fn the_fence_is_written_once_however_often_it_is_asked_for() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    settle(&home.roots);
    let fenced = read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap();
    std::fs::remove_file(home.roots.real.join("MOVED.txt")).unwrap();

    fence_moved(&home.roots.new);

    assert_eq!(
        read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap(),
        fenced
    );
}

#[test]
fn a_store_moved_again_after_its_new_home_was_lost_comes_back_readable() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    settle(&home.roots);
    std::fs::remove_dir_all(&home.roots.new).unwrap();
    std::fs::remove_file(home.roots.real.join("MOVED.txt")).unwrap();

    assert!(matches!(settle(&home.roots), Settled::Moved { .. }));

    assert!(
        readable(&home.roots.new),
        "the fence came along and locked the copy"
    );
}

#[test]
fn a_store_still_being_written_waits_for_the_next_start() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    let _writing = crate::store::alone(&home.roots.real.join("data/store/dev_a")).unwrap();

    assert!(matches!(settle(&home.roots), Settled::Failed(_)));

    assert!(!home.roots.new.exists());
    assert!(readable(&home.roots.real));
}

#[test]
fn what_a_cut_short_move_left_is_swept_and_never_copied() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    put(&home.roots.real, "data/docs/half.md.tmp", "half");
    let parent = home.roots.new.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(parent.join(".keep.part-1")).unwrap();

    settle(&home.roots);

    assert!(!parent.join(".keep.part-1").exists());
    assert!(!home.roots.new.join("data/store/dev_a/.lock").exists());
    assert!(!home.roots.new.join("data/docs/half.md.tmp").exists());
}

#[test]
fn an_attachment_keeps_its_own_extension_whatever_it_is() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    put(
        &home.roots.real,
        "data/attachments/ab/0123-report.tmp",
        "a real file",
    );
    put(
        &home.roots.real,
        "data/attachments/ab/.0123-half.part",
        "half",
    );

    settle(&home.roots);

    assert!(
        home.roots
            .new
            .join("data/attachments/ab/0123-report.tmp")
            .is_file()
    );
    assert!(
        !home
            .roots
            .new
            .join("data/attachments/ab/.0123-half.part")
            .exists()
    );
}

#[test]
fn a_store_app_updated_in_place_carries_its_own_copy_out_of_the_package() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&private, "only ever in the store app");

    assert_eq!(settle(&home.roots), Settled::Moved { aside: None });

    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("only ever in the store app")
    );
    assert!(readable(&home.roots.new));
}

#[test]
fn a_loose_install_put_in_before_the_store_app_is_removed_takes_its_copy_first() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&private, "written in the store app");
    put(&private, "config/private/key", "the store app's key");

    assert!(matches!(settle(&home.roots), Settled::Moved { .. }));
    std::fs::remove_dir_all(private.parent().unwrap()).unwrap();

    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("written in the store app")
    );
    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("the store app's key")
    );
}

#[test]
fn the_store_app_put_in_over_a_loose_install_takes_what_the_loose_one_wrote() {
    let home = home(true);
    wrote(&home.roots.real, "written by the loose install");

    assert_eq!(settle(&home.roots), Settled::Moved { aside: None });

    assert!(
        read(&home.roots.new, "data/store/dev_a/active.tisty")
            .unwrap()
            .contains("written by the loose install")
    );
}

#[cfg(windows)]
#[test]
fn leaving_one_install_keeps_the_settings_the_others_share() {
    let root = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".tisty");
    let shared = crate::Paths::new(root.join("data"), root.join("config"));

    assert!(!shared.swept_on_leaving().contains(&shared.config_file()));
}
