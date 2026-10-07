use super::*;
use crate::event::{DeviceId, TaskAdd};
use crate::{Error, Op, Store};

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

fn read(root: &Path, at: &str) -> Option<String> {
    std::fs::read_to_string(root.join(at)).ok()
}

fn readable(root: &Path) -> bool {
    crate::store::read_all(root.join("data/store")).is_ok()
}

fn holds(root: &Path, title: &str) -> bool {
    read(root, "data/store/dev_a/active.tisty").is_some_and(|body| body.contains(title))
}

#[test]
fn a_machine_that_never_ran_tisty_has_nothing_to_move() {
    let home = home(true);

    assert_eq!(settle(&home.roots), Settled::Fresh);
    assert!(!home.roots.new.exists());
}

#[test]
fn a_store_app_updated_in_place_carries_its_own_copy_out_of_the_package() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    wrote(&private, "only ever in the store app");
    put(&private, "config/private/key", "the store app's key");

    assert_eq!(settle(&home.roots), Settled::Moved);

    assert!(holds(&home.roots.new, "only ever in the store app"));
    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("the store app's key")
    );
    assert!(readable(&home.roots.new));
    assert!(
        !home.roots.new.join("aside").exists(),
        "nothing was copied twice"
    );
}

#[test]
fn a_loose_install_put_in_before_the_store_app_is_removed_takes_the_store_view() {
    let home = home(true);
    let private = home.roots.private.clone().unwrap();
    put(
        &home.roots.real,
        "config/private/key",
        "the key nobody rewrote",
    );
    put(
        &home.roots.real,
        "data/docs/both.md",
        "what the store app read first",
    );
    wrote(&private, "written in the store app");
    put(
        &private,
        "data/docs/both.md",
        "what the store app wrote over it",
    );

    assert_eq!(settle(&home.roots), Settled::Moved);
    std::fs::remove_dir_all(private.parent().unwrap()).unwrap();

    assert!(holds(&home.roots.new, "written in the store app"));
    assert_eq!(
        read(&home.roots.new, "data/docs/both.md").as_deref(),
        Some("what the store app wrote over it")
    );
    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("the key nobody rewrote"),
        "what only the real folder held was lost"
    );
}

#[test]
fn the_store_app_put_in_over_a_loose_install_takes_what_the_loose_one_wrote() {
    let home = home(true);
    wrote(&home.roots.real, "written by the loose install");
    put(&home.roots.real, "config/private/key", "secret");

    assert_eq!(settle(&home.roots), Settled::Moved);

    assert!(holds(&home.roots.new, "written by the loose install"));
    assert_eq!(
        read(&home.roots.new, "config/private/key").as_deref(),
        Some("secret")
    );
    assert!(
        read(&home.roots.real, "config/private/key").is_some(),
        "the old folder was emptied"
    );
    assert!(home.roots.real.join("MOVED.txt").is_file());
}

#[test]
fn a_root_already_there_is_left_alone() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    std::fs::create_dir_all(&home.roots.new).unwrap();

    assert_eq!(settle(&home.roots), Settled::AlreadyThere);

    assert!(
        readable(&home.roots.real),
        "a store nobody copied was fenced off"
    );
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
fn a_store_moved_again_after_its_new_home_was_lost_comes_back_readable() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    settle(&home.roots);
    let fenced = read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap();
    std::fs::remove_dir_all(&home.roots.new).unwrap();

    assert_eq!(settle(&home.roots), Settled::Moved);

    assert!(
        readable(&home.roots.new),
        "the fence came along and locked the copy"
    );
    assert_eq!(
        read(&home.roots.real, "data/store/dev_a/active.tisty").unwrap(),
        fenced,
        "the fence was written twice"
    );
}

#[test]
fn a_store_still_being_written_is_not_moved_from_under_the_writer() {
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

#[test]
fn a_fence_that_did_not_hold_is_written_at_the_next_start() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    settle(&home.roots);
    let active = home.roots.real.join("data/store/dev_a/active.tisty");
    std::fs::remove_file(home.roots.real.join("MOVED.txt")).unwrap();
    let body = std::fs::read_to_string(&active).unwrap();
    let unfenced: String = body
        .lines()
        .take(body.lines().count() - 1)
        .map(|one| format!("{one}\n"))
        .collect();
    std::fs::write(&active, unfenced).unwrap();
    assert!(readable(&home.roots.real));

    assert_eq!(settle(&home.roots), Settled::AlreadyThere);

    assert!(!readable(&home.roots.real), "the old store was left open");
    assert!(home.roots.real.join("MOVED.txt").is_file());
}

#[cfg(unix)]
#[test]
fn a_link_that_points_back_up_the_tree_is_followed_once() {
    let home = home(false);
    wrote(&home.roots.real, "pay the bill");
    std::os::unix::fs::symlink(
        home.roots.real.join("data"),
        home.roots.real.join("data/docs"),
    )
    .unwrap();

    assert_eq!(settle(&home.roots), Settled::Moved);
}

#[test]
fn the_room_a_move_needs_counts_only_what_moves() {
    let home = home(false);
    put(&home.roots.real, "data/docs/one.md", "12345");
    put(&home.roots.real, "config/config.toml", "123");
    put(
        &home.roots.real,
        "cache/read.db",
        "a cache that stays behind",
    );

    assert_eq!(weighed(&home.roots.real), 8);
}

#[test]
fn a_move_tells_how_far_it_has_come_until_it_has_copied_everything() {
    let home = home(false);
    wrote(&home.roots.real, "a task to carry");
    put(
        &home.roots.real,
        "data/attachments/aa/una-foto.png",
        &"x".repeat(4096),
    );
    assert!(moves(&home.roots));
    let mut told = Vec::new();

    let settled = settle_telling(&home.roots, &mut |done, whole| told.push((done, whole)));

    assert_eq!(settled, Settled::Moved);
    assert!(
        !moves(&home.roots),
        "a store already moved still says it moves"
    );
    let whole = told.last().unwrap().1;
    assert!(whole >= 4096, "{told:?}");
    assert_eq!(told.first(), Some(&(0, whole)));
    assert_eq!(told.last(), Some(&(whole, whole)));
    assert!(told.windows(2).all(|two| two[0].0 <= two[1].0), "{told:?}");
}

#[test]
fn a_machine_with_nothing_to_move_says_so_before_it_starts() {
    let home = home(true);

    assert!(!moves(&home.roots));
}
