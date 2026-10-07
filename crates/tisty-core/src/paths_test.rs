use super::{as_written, aside, from_the_store, named, told_apart, under_windows_apps};
use std::path::PathBuf;

#[test]
fn a_profile_name_cannot_escape_its_own_directory() {
    for raw in ["../..", "..", "/etc", "a/../../b", r"..\..\windows"] {
        let clean = named(raw).unwrap_or_default();
        assert!(!clean.contains(".."), "{raw} survived as {clean}");
        assert!(!clean.contains('/'), "{raw} -> {clean}");
        assert!(!clean.contains('\\'), "{raw} -> {clean}");
    }
}

#[test]
fn an_ordinary_name_survives_whole() {
    assert_eq!(named("demo"), Some("demo".into()));
    assert_eq!(named("dos-maquinas_2"), Some("dos-maquinas_2".into()));
}

#[test]
fn a_name_with_nothing_usable_in_it_is_no_profile() {
    assert_eq!(named(""), None);
    assert_eq!(named("   "), None);
    assert_eq!(named("../.."), None);
}

#[test]
fn without_a_profile_nothing_moves() {
    let root = PathBuf::from("/data");

    assert_eq!(aside(root.clone(), None), root);
    assert_eq!(
        aside(root, Some("demo")),
        PathBuf::from("/data/sandboxes/demo")
    );
}
use super::*;

fn paths() -> Paths {
    Paths::new("/data/Tisty", "/config/tisty")
}

#[test]
fn nothing_private_lives_where_the_transports_look() {
    let p = Paths::resolve().unwrap();
    for kept in [p.config_file(), p.private(), p.cache().to_path_buf()] {
        assert!(!kept.starts_with(p.store()), "{kept:?}");
        assert!(!kept.starts_with(p.attachments()), "{kept:?}");
        assert!(!kept.starts_with(p.docs()), "{kept:?}");
    }
}

#[test]
fn leaving_takes_the_settings_and_not_what_proves_the_store_is_its_own() {
    let p = paths();
    let swept = p.swept_on_leaving();

    assert!(swept.contains(&p.config_file()), "{swept:?}");
    assert!(
        !swept.iter().any(|at| p.private().starts_with(at)),
        "leaving would take the key with it: {swept:?}"
    );
    assert!(
        swept.contains(&crate::witness::file(&p)),
        "leaving would keep the diary of a machine that left: {swept:?}"
    );
    assert!(
        swept.iter().any(|at| at.ends_with("tisty.log.1")),
        "leaving would keep the rolled-over diary: {swept:?}"
    );
    assert!(
        !swept
            .iter()
            .any(|at| at.ends_with(crate::store::KEEP) || at == &p.private()),
        "leaving would take the key with it: {swept:?}"
    );
}

#[test]
fn the_config_never_lands_in_a_roaming_profile() {
    let p = Paths::resolve().unwrap();
    if let Some(roaming) = std::env::var_os("APPDATA").map(PathBuf::from) {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        if local.as_deref() != Some(roaming.as_path()) {
            assert!(!p.config().starts_with(&roaming), "{:?}", p.config());
            assert!(!p.private().starts_with(&roaming), "{:?}", p.private());
        }
    }
}

#[test]
fn every_device_gets_its_own_directory() {
    let p = paths();
    let a = p.device_dir(&DeviceId("dev_a".into()));
    let b = p.device_dir(&DeviceId("dev_b".into()));

    assert_ne!(a, b);
    assert!(a.starts_with(p.store()));
    assert!(b.starts_with(p.store()));
}

#[test]
fn the_store_never_lands_in_the_documents_folder() {
    let p = Paths::resolve().unwrap();
    assert!(p.data().is_absolute(), "{:?}", p.data());

    let documents =
        directories::UserDirs::new().and_then(|d| d.document_dir().map(Path::to_path_buf));
    if let Some(documents) = documents {
        assert!(!p.data().starts_with(&documents), "{:?}", p.data());
    }
}

#[test]
fn the_cache_is_disposable_and_never_synced() {
    let p = paths();
    assert!(!p.selection_file().starts_with(p.data()));
}

#[test]
fn store_docs_and_attachments_are_all_synced() {
    let p = paths();
    for path in [p.store(), p.docs(), p.attachments()] {
        assert!(path.starts_with(p.data()), "{path:?} should be synced");
    }
}

#[test]
fn a_folder_written_down_is_the_one_the_disk_agrees_on() {
    let room = tempfile::tempdir().unwrap();
    let one = room.path().join("one");
    let two = room.path().join("two");
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&two).unwrap();

    assert_eq!(as_written(&one), as_written(&one.join("..").join("one")));
    assert_ne!(as_written(&one), as_written(&two));
}

#[test]
fn what_tells_a_folder_apart_follows_it_and_not_the_name_it_was_reached_by() {
    let room = tempfile::tempdir().unwrap();
    let one = room.path().join("one");
    let two = room.path().join("two");
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&two).unwrap();

    let mark = told_apart(&one).expect("a folder that is there can be told apart");
    assert_eq!(
        told_apart(&one.join("..").join("one")).as_deref(),
        Some(mark.as_str())
    );
    assert_ne!(told_apart(&two).as_deref(), Some(mark.as_str()));
    assert_ne!(mark, as_written(&one), "a path is not what tells it apart");
}

#[test]
fn a_folder_that_is_not_there_is_written_down_as_it_was_asked_for() {
    let room = tempfile::tempdir().unwrap();
    let nowhere = room.path().join("nowhere");

    assert_eq!(as_written(&nowhere), nowhere.display().to_string());
    assert_eq!(told_apart(&nowhere), None, "nothing there to tell apart");
}

#[test]
fn only_a_copy_kept_under_windows_apps_is_the_stores() {
    for packaged in [
        r"C:\Program Files\WindowsApps\rgdevment.Tisty_1.24.0.0_x64__kdjgfdc2rb3gc\cli\tisty.exe",
        "/c/program files/windowsapps/tisty/Tisty.exe",
    ] {
        assert!(
            under_windows_apps(std::path::Path::new(packaged)),
            "{packaged}"
        );
    }
    for loose in [
        r"C:\Users\someone\AppData\Local\Programs\Tisty\tisty.exe",
        r"C:\Tools\NotWindowsApps\tisty.exe",
    ] {
        assert!(!under_windows_apps(std::path::Path::new(loose)), "{loose}");
    }
}

#[test]
fn a_test_binary_is_no_store_copy() {
    assert!(!from_the_store());
}

#[cfg(windows)]
#[test]
fn the_key_folder_answers_to_this_user_alone_and_inherits_nothing() {
    let room = tempfile::tempdir().unwrap();
    let private = room.path().join("private");
    std::fs::create_dir_all(&private).unwrap();

    key_alone(&private).unwrap();

    let said = sddl_of(&private, room.path());
    let dacl = said
        .lines()
        .nth(1)
        .and_then(|line| line.split_once("D:"))
        .map(|(_, dacl)| dacl.to_string())
        .unwrap_or_default();
    let aces: Vec<&str> = dacl
        .split('(')
        .skip(1)
        .map(|ace| ace.trim_end_matches(|c: char| c == ')' || c.is_whitespace() || c == '\0'))
        .collect();
    let who = |ace: &str| ace.rsplit(';').next().unwrap_or_default().to_string();
    let sid = account_sid().unwrap();
    // SDDL writes the built-in administrator's own SID as LA.
    let mine = |ace: &str| who(ace) == sid || (who(ace) == "LA" && sid.ends_with("-500"));

    assert!(
        dacl.starts_with('P'),
        "something is still inherited: {said}"
    );
    assert!(
        aces.iter()
            .all(|ace| !ace.split(';').nth(1).unwrap_or_default().contains("ID")),
        "something is still inherited: {said}"
    );
    assert!(
        aces.iter()
            .any(|ace| mine(ace) && ace.split(';').nth(2) == Some("FA")),
        "the folder went to someone else: {said}"
    );
    // Windows itself and its administrators can always take any folder, so the key is kept from everyone else.
    assert!(
        aces.iter()
            .all(|ace| mine(ace) || who(ace) == "SY" || who(ace) == "BA"),
        "more than this user can reach the key: {said}"
    );
}

#[cfg(unix)]
#[test]
fn the_key_folder_answers_to_this_user_alone() {
    use std::os::unix::fs::PermissionsExt;
    let room = tempfile::tempdir().unwrap();
    let private = room.path().join("private");
    std::fs::create_dir_all(&private).unwrap();

    key_alone(&private).unwrap();

    let mode = std::fs::metadata(&private).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[cfg(windows)]
fn sddl_of(at: &Path, room: &Path) -> String {
    let saved = room.join("acl");
    let done = std::process::Command::new("icacls")
        .arg(at)
        .arg("/save")
        .arg(&saved)
        .output()
        .unwrap();
    assert!(done.status.success(), "{done:?}");
    let raw = std::fs::read(&saved).unwrap();
    let words: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|two| u16::from_le_bytes(*two))
        .collect();
    String::from_utf16_lossy(&words)
}

#[cfg(windows)]
#[test]
fn the_home_set_aside_is_hidden_inherits_nothing_and_is_looked_at_once() {
    use std::os::windows::fs::MetadataExt;
    let room = tempfile::tempdir().unwrap();
    let home = room.path().join(".tisty");
    std::fs::create_dir_all(home.join("data")).unwrap();

    tucked(&home).unwrap();
    tucked(&home).unwrap();

    let hidden = std::fs::metadata(&home).unwrap().file_attributes() & 0x2;
    assert_ne!(hidden, 0, "the home is still in plain sight");
    assert!(home.join(".kept").exists(), "a finished walk left no mark");
    let said = sddl_of(&home, room.path());
    let dacl = said
        .lines()
        .nth(1)
        .and_then(|line| line.split_once("D:"))
        .map(|(_, dacl)| dacl.to_string())
        .unwrap_or_default();
    assert!(dacl.starts_with('P'), "the home still inherits: {said}");
    assert!(
        dacl.contains(";;;SY)"),
        "a backup running as the system account lost the store: {said}"
    );
    assert!(
        !dacl.contains("ID;"),
        "something is still inherited: {said}"
    );
}

#[cfg(windows)]
#[test]
fn a_home_somebody_hid_by_hand_is_still_kept_to_this_account() {
    let room = tempfile::tempdir().unwrap();
    let home = room.path().join(".tisty");
    std::fs::create_dir_all(&home).unwrap();
    let hid = std::process::Command::new("attrib")
        .arg("+h")
        .arg(&home)
        .status()
        .unwrap();
    assert!(hid.success());

    tucked(&home).unwrap();

    let said = sddl_of(&home, room.path());
    let dacl = said
        .lines()
        .nth(1)
        .and_then(|line| line.split_once("D:"))
        .map(|(_, dacl)| dacl.to_string())
        .unwrap_or_default();
    assert!(
        dacl.starts_with('P'),
        "a hidden home was taken as kept: {said}"
    );
}

#[cfg(windows)]
#[test]
fn a_walk_cut_short_is_walked_again_though_the_home_itself_is_kept() {
    let room = tempfile::tempdir().unwrap();
    let home = room.path().join(".tisty");
    std::fs::create_dir_all(&home).unwrap();
    tucked(&home).unwrap();
    std::fs::remove_file(home.join(".kept")).unwrap();

    tucked(&home).unwrap();

    assert!(
        home.join(".kept").exists(),
        "a home with no mark of a finished walk was taken as kept"
    );
}

#[test]
fn a_store_kept_anywhere_else_leaves_the_shared_home_untouched() {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));

    home_set_aside(&paths);

    assert!(!room.path().join(".kept").exists());
    assert!(!room.path().join("data").join(".kept").exists());
}

#[test]
fn a_home_that_could_not_be_set_aside_is_said_and_not_kept_quiet() {
    let _alone = crate::witness::ALONE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let room = tempfile::tempdir().unwrap();
    let log = room.path().join("tisty.log");
    crate::witness::keeps(log.clone(), true);

    not_set_aside(
        room.path().join(".tisty"),
        std::io::Error::other("a file was locked"),
    );

    crate::witness::stops();
    let said = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(said.contains("could not be hidden"), "{said}");
    assert!(said.contains("a file was locked"), "{said}");
}

#[cfg(not(windows))]
#[test]
fn only_windows_has_a_home_to_hide() {
    let room = tempfile::tempdir().unwrap();

    assert!(tucked(room.path()).is_ok());
    assert!(!room.path().join(".kept").exists());
}
