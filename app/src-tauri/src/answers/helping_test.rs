use super::*;

#[test]
fn every_page_names_its_twin_in_the_other_language() {
    let english = help_pages(false);
    let spanish = help_pages(true);
    assert_eq!(english.len(), spanish.len());
    for one in &english {
        assert!(
            spanish.iter().any(|other| other.slug == one.twin),
            "{} has no twin",
            one.slug
        );
    }
    assert_eq!(english[0].slug, "");
    assert!(help_page(true, "sincronizar-y-respaldar".into()).is_some());
    assert!(help_page(true, "nowhere".into()).is_none());
}

#[test]
fn pictures_are_missing_until_kept_and_each_one_is_counted_once() {
    let cache = tempfile::tempdir().unwrap();
    let at = folder(cache.path());
    let before = standing(&at, true);
    assert!(before.weighs > 0);
    assert_eq!(before.missing, before.weighs);
    assert!(before.kept.is_empty());

    let first = tisty_help::pictures(true).remove(0);
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(kept_at(&at, &first), b"png").unwrap();
    let after = standing(&at, true);
    assert_eq!(after.kept.len(), 1);
    assert_eq!(
        after.missing,
        before.weighs - tisty_help::weight(&first).unwrap()
    );
}
