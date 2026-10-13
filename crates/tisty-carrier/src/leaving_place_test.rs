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

#[test]
fn a_round_sends_the_history_before_the_attachments() {
    for setting in clouds() {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        one.wrote("con foto");
        one.filed(DOC, "el acta");
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        let remote = counting(&setting);
        remote.forget_counts();

        one.round(Way::Both);

        let sent = remote.uploaded();
        let first_attachment = sent
            .iter()
            .position(|name| name.starts_with("attachments/"));
        let last_rest = sent.iter().rposition(|name| name.starts_with("store/"));
        assert!(
            first_attachment.is_some() && last_rest.is_some(),
            "{who}: {sent:?}"
        );
        assert!(last_rest < first_attachment, "{who}: {sent:?}");
    }
}

#[test]
fn an_attachment_gone_from_this_machine_never_holds_back_the_round() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto perdida");
        let at = tisty_core::attach::resolve(&kept.at, &one.here.data).unwrap();
        std::fs::remove_file(at).unwrap();

        let sent = one.carry(Way::Both, tisty_core::config::Holds::Everywhere);
        two.round(Way::Pull);

        assert!(sent.is_ok(), "{who}: {sent:?}");
        assert_eq!(two.titles(), ["con foto perdida"], "{who}");
    }
}

#[test]
fn a_machine_receives_tasks_and_documents_before_an_attachment_has_come_down() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        one.filed(
            DOC,
            &format!(
                "![foto]({})
",
                kept.at
            ),
        );
        one.round(Way::Both);

        two.round(Way::Pull);

        assert_eq!(two.titles(), ["con foto"], "{who}");
        assert!(two.doc(DOC).is_some(), "{who}");
        assert!(
            two.held(&kept.at).is_none(),
            "{who}: brought before it was opened"
        );
    }
}
