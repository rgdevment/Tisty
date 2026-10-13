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

#[test]
fn a_file_the_last_pull_could_not_take_is_no_news_until_it_changes() {
    let waiting = crate::Seen {
        name: "store/dev_b/000001.tisty".into(),
        bytes: 3,
        hash: "h".into(),
        revision: "r1".into(),
    };
    let mut index = super::super::index::Index::default();
    index.owed.insert(waiting.name.clone(), waiting.clone());
    let same = crate::Changes::Since {
        changed: vec![waiting.clone()],
        gone: Vec::new(),
        cursor: "c".into(),
    };
    let moved_on = crate::Changes::Since {
        changed: vec![crate::Seen {
            revision: "r2".into(),
            ..waiting
        }],
        gone: Vec::new(),
        cursor: "c".into(),
    };

    assert!(!super::super::news(&index, &same));
    assert!(super::super::news(&index, &moved_on));
}

#[test]
fn the_tasks_sent_ahead_spend_the_history_budget_not_the_attachments_one() {
    for setting in clouds() {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        let home = home_of(&setting, "dev_a");
        std::fs::create_dir_all(&home).unwrap();
        let today = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 86_400;
        let remote = counting(&setting);
        let past_attachments = remote.limits().units_a_day / 100 * 95;
        std::fs::write(
            home.join("budget.json"),
            format!(r#"{{"day":{today},"units":{past_attachments},"bytes":0}}"#),
        )
        .unwrap();
        remote.forget_counts();

        one.round(Way::Both);

        let sent = remote.uploaded();
        assert!(
            sent.iter().any(|name| name.starts_with("store/")),
            "{who}: {sent:?}"
        );
        assert!(
            !sent.iter().any(|name| name.starts_with("attachments/")),
            "{who}: {sent:?}"
        );
    }
}
