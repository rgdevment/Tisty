use super::{aside, named};
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
