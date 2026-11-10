use super::{outside_tisty, tistys_own};

#[test]
fn a_file_tisty_already_keeps_is_never_brought_in_as_new() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let mut session = crate::Session::at(paths.clone()).unwrap();
    let synced = tmp.path().join("OneDrive").join("Tisty");
    std::fs::create_dir_all(synced.join("docs")).unwrap();
    session
        .keep(|config| {
            config.sync = Some(tisty_core::config::Sync::Folder(synced.clone()));
        })
        .unwrap();
    let kept = paths.docs().join("de-ana.md");
    std::fs::create_dir_all(paths.docs()).unwrap();
    std::fs::write(&kept, "# De Ana").unwrap();
    let mirrored = synced.join("docs").join("de-ana.md");
    std::fs::write(&mirrored, "# De Ana").unwrap();
    let outside = tmp.path().join("mio.md");
    std::fs::write(&outside, "# Mío").unwrap();
    let own = tistys_own(&session);

    for at in [&kept, &mirrored] {
        assert_eq!(
            outside_tisty(&own, at).unwrap_err().code,
            "importedFromTisty"
        );
    }
    assert!(outside_tisty(&own, &outside).is_ok());
}
