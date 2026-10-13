use super::*;

#[test]
fn a_fresh_machine_sees_that_a_place_already_holds_a_tisty() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let _ = two.carrier.kin(&two.here);
        assert!(!two.carrier.holds_a_store(), "{who}");

        one.wrote("ya estaba");
        one.round(Way::Both);

        let _ = two.carrier.kin(&two.here);
        assert!(two.carrier.holds_a_store(), "{who}");
    }
}

#[test]
fn wiping_a_cloud_bins_every_copy_and_keeps_only_the_word_for_the_others() {
    for setting in clouds() {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        one.wrote("algo");
        one.filed(DOC, "contenido");
        one.round(Way::Both);
        one.carrier.leave_word("se fue").unwrap();

        let gone = one.carrier.wipe().unwrap();

        let left: Vec<String> = counting(&setting)
            .list("")
            .unwrap()
            .into_iter()
            .map(|seen| seen.name)
            .collect();
        assert!(gone > 0, "{who}");
        assert!(
            left.iter().all(|name| name == tisty_core::moved::NAMED),
            "{who}: {left:?}"
        );
    }
}

#[test]
fn a_folder_has_nothing_to_wipe() {
    let room = tempfile::tempdir().unwrap();
    assert_eq!(Folder::at(room.path().to_path_buf()).wipe().unwrap(), 0);
}
