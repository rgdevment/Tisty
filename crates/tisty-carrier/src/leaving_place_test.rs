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
fn a_machine_leaving_a_place_knows_who_else_still_writes_there() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("de uno");
        one.round(Way::Both);
        assert!(one.carrier.others(&one.here).is_empty(), "{who}");

        two.round(Way::Pull);
        two.wrote("de dos");
        two.round(Way::Both);
        one.round(Way::Pull);

        assert_eq!(one.carrier.others(&one.here), ["dev_b"], "{who}");
        assert_eq!(two.carrier.others(&two.here), ["dev_a"], "{who}");
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
        assert_eq!(left, [tisty_core::moved::NAMED], "{who}");
    }
}

/// Hands every request to the provider it wraps, except the word, which the provider will not take.
struct WordRefused {
    inner: Arc<dyn Counting>,
    hitch: Hitch,
}

impl Remote for WordRefused {
    fn limits(&self) -> crate::Limits {
        self.inner.limits()
    }
    fn list(&self, under: &str) -> Result<Vec<crate::Seen>, Hitch> {
        self.inner.list(under)
    }
    fn fetch(
        &self,
        name: &str,
        from: u64,
        into: &mut dyn std::io::Write,
    ) -> Result<crate::Seen, Hitch> {
        self.inner.fetch(name, from, into)
    }
    fn put(
        &self,
        name: &str,
        from: &std::path::Path,
        expect: Expect,
    ) -> Result<crate::Seen, Hitch> {
        match name == tisty_core::moved::NAMED {
            true => Err(self.hitch.clone()),
            false => self.inner.put(name, from, expect),
        }
    }
    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.inner.delete(name, expect)
    }
    fn hash_of(&self, local: &std::path::Path) -> std::io::Result<String> {
        self.inner.hash_of(local)
    }
    fn about(&self, name: &str) -> Result<Option<crate::Seen>, Hitch> {
        self.inner.about(name)
    }
    fn changes(&self, since: Option<&str>) -> Result<crate::Changes, Hitch> {
        self.inner.changes(since)
    }
}

#[test]
fn a_word_the_cloud_would_not_take_is_never_taken_for_delivered_nor_lets_the_copies_go() {
    let refusals = [
        Hitch::Full,
        Hitch::Limited {
            wait: Duration::from_secs(9),
        },
        Hitch::Unreachable("sin red".into()),
    ];
    for hitch in &refusals {
        for setting in clouds() {
            let who = format!("{} · {hitch:?}", setting.name);
            let one = desk(&setting, "dev_a");
            one.wrote("algo");
            one.round(Way::Both);
            let refusing: Arc<dyn Remote> = Arc::new(WordRefused {
                inner: counting(&setting).clone(),
                hitch: hitch.clone(),
            });
            let leaving = Cloud::over(refusing, home_of(&setting, "dev_a"));

            let said = leaving.leave_word("se fue");
            let wiped = one.carrier.wipe();

            assert!(
                said.is_err(),
                "{who}: a word that never went up was said to"
            );
            assert_eq!(
                wiped,
                Err(Trouble::NotThere(tisty_core::moved::NAMED.to_string())),
                "{who}"
            );
            let left = counting(&setting).list("").unwrap();
            assert!(
                left.iter().any(|seen| seen.name.starts_with("store/")),
                "{who}: the copies went before the others were told"
            );
        }
    }
}

#[test]
fn a_word_that_went_up_is_delivered_and_its_copies_can_go() {
    for setting in clouds() {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        one.wrote("algo");
        one.round(Way::Both);

        one.carrier.leave_word("se fue").unwrap();

        let there = counting(&setting).about(tisty_core::moved::NAMED).unwrap();
        assert!(there.is_some(), "{who}");
        assert!(one.carrier.wipe().is_ok(), "{who}");
    }
}

#[test]
fn a_folder_has_nothing_to_wipe() {
    let room = tempfile::tempdir().unwrap();
    assert_eq!(Folder::at(room.path().to_path_buf()).wipe().unwrap(), 0);
}

#[test]
fn leaving_a_place_older_builds_still_use_raises_it_so_they_stop_too() {
    for setting in settings() {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        one.wrote("algo");
        one.round(Way::Both);
        let older = "shape = 1\n";
        let shaped = match &setting.place {
            Place::Folder(shared) => shared.join(tisty_sync::NAMED),
            Place::Cloud(_) => home_of(&setting, "dev_a")
                .join("tree")
                .join(tisty_sync::NAMED),
        };
        std::fs::write(&shaped, older).unwrap();
        if let Place::Cloud(remote) = &setting.place {
            remote.put(tisty_sync::NAMED, &shaped, Expect::Any).unwrap();
        }

        one.carrier.leave_word("se fue").unwrap();

        let said = std::fs::read_to_string(&shaped).unwrap();
        assert!(
            said.contains(&format!("shape = {}", crate::SHAPE)),
            "{who}: {said}"
        );
        if let Place::Cloud(remote) = &setting.place {
            let mut there = Vec::new();
            remote.fetch(tisty_sync::NAMED, 0, &mut there).unwrap();
            assert_ne!(there, older.as_bytes(), "{who}: the raise never went up");
        }
    }
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

#[test]
fn a_folder_left_keeps_a_note_for_whoever_opens_it_until_this_machine_chooses_it_again() {
    let room = tempfile::tempdir().unwrap();
    let folder = Folder::at(room.path().to_path_buf());
    let said = tisty_core::moved::said(
        &tisty_core::signing::SigningKey::from_bytes(&[3; 32]),
        tisty_core::moved::Leaving {
            by: &tisty_core::DeviceId("dev_a".into()),
            to: "cloud:ana@example.com",
            at: 1_760_000_000,
            store: "store",
            place: "folder",
        },
    );
    let note = room.path().join(tisty_core::left_behind::NOTICE);

    folder.leave_word(&said).unwrap();
    assert!(
        std::fs::read_to_string(&note)
            .unwrap()
            .contains("Google Drive"),
        "the folder left behind says nothing of where things went"
    );

    folder.forget_word().unwrap();
    assert!(
        !note.exists(),
        "the note stayed once the folder was chosen again"
    );
}
