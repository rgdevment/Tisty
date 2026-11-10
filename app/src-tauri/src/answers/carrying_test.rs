use super::{outside_tisty, shown_as, tistys_own};
use tisty_carrier::folio_limits::FolioMaker;

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

#[test]
fn a_parcel_signed_with_a_name_like_the_makers_from_another_root_names_nobody() {
    let maker = FolioMaker {
        root: "R".repeat(43),
        name: Some("rgdevment".into()),
        domain: Some("rgdevment.dev".into()),
    };
    let other = "O".repeat(43);

    assert_eq!(
        shown_as(Some("rgdevrnent".into()), &other, Some(&maker)),
        None
    );
    assert_eq!(
        shown_as(Some("rgdevment".into()), &maker.root, Some(&maker)).as_deref(),
        Some("rgdevment")
    );
    assert_eq!(
        shown_as(Some("beto".into()), &other, Some(&maker)).as_deref(),
        Some("beto")
    );
    assert_eq!(
        shown_as(Some("rgdevment".into()), &other, None).as_deref(),
        Some("rgdevment")
    );
}
