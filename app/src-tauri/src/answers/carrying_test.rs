use super::outside_tisty;

#[test]
fn a_file_tisty_already_keeps_is_never_brought_in_as_new() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let kept = paths.docs().join("de-ana.md");
    std::fs::create_dir_all(paths.docs()).unwrap();
    std::fs::write(&kept, "# De Ana").unwrap();
    let outside = tmp.path().join("mio.md");
    std::fs::write(&outside, "# Mío").unwrap();

    assert_eq!(
        outside_tisty(&paths, &kept).unwrap_err().code,
        "importedFromTisty"
    );
    assert!(outside_tisty(&paths, &outside).is_ok());
}
