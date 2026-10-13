use super::*;

#[test]
fn bytes_put_back_after_drive_s_gap_and_never_told_still_reach_the_machine_that_wrote() {
    let (setting, fake) = each_kind(|fake| fake.restoring_unheard()).remove(0);
    assert_eq!(fake.who(), "drive");
    let (one, two) = pair(&setting);
    one.filed(DOC, "# Plan\n\nlo primero\n");
    one.round(Way::Both);
    two.round(Way::Both);
    one.edited(DOC, "# Plan\n\nlo de uno\n");
    two.edited(DOC, "# Plan\n\nlo de dos\n");
    let meanwhile = Desk {
        here: two.here.clone(),
        carrier: two.carrier.clone(),
    };
    fake.in_the_gap(&doc_name(), move || {
        meanwhile.round(Way::Both);
    });

    one.round(Way::Both);
    one.round(Way::Both);

    assert_eq!(one.doc(DOC).as_deref(), Some("# Plan\n\nlo de uno\n"));
    assert!(
        one.carrier.paper_waiting(DOC),
        "the write refused in the gap was refused every round and nobody was asked"
    );
}

#[test]
fn a_file_the_provider_will_not_take_stops_the_push_and_is_named() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo primero\n");
        one.round(Way::Both);
        fake.refusing(&doc_name(), Hitch::Broke("not taken".into()));
        one.edited(DOC, "# Plan\n\nlo de después\n");

        let stopped = match one.carry(Way::Both, Holds::Everywhere) {
            Ok(moved) => moved.refused,
            Err(trouble) => Some(trouble),
        };

        assert!(
            matches!(&stopped, Some(Trouble::StuckOn { name, .. }) if *name == doc_name()),
            "{who}: the person was not told which file holds the rest back: {stopped:?}"
        );
    }
}

const PHOTO: &[u8] = b"los bytes de una foto";

impl Desk {
    fn brought_home(&self) -> Moved {
        let mut saying = |_| {};
        self.carrier
            .bring_home(
                &self.here,
                Round {
                    way: Way::Pull,
                    alive: &[],
                    holds: Holds::Everywhere,
                    saying: &mut saying,
                },
            )
            .unwrap()
    }
}

#[test]
fn an_attachment_past_what_the_cloud_takes_stays_here_and_says_so() {
    for setting in settings() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let reference = "attachments/ab/grande-0123456789.bin";
        let at = tisty_core::attach::resolve(reference, &one.here.data).unwrap();
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::File::create(&at)
            .unwrap()
            .set_len(tisty_core::attach::HELD_UP_TO + 1)
            .unwrap();
        one.wrote("con algo grande");

        let moved = one.round(Way::Both);

        let unsent = moved.deferred.expect(who);
        assert_eq!(unsent.reason, Reason::Unsent, "{who}");
        assert!(
            unsent.names.iter().any(|one| one == reference),
            "{who}: {unsent:?}"
        );
        assert!(at.is_file(), "{who}: what could not go was not kept here");
        assert_eq!(
            one.round(Way::Both).deferred.map(|one| one.reason),
            Some(Reason::Unsent),
            "{who}: said once and then forgotten"
        );
    }
}

#[test]
fn an_attachment_up_there_with_other_bytes_is_told_and_sent_over_it_next_round() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let kept = one.kept("foto", PHOTO);
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        let wrong = setting.room.path().join("wrong");
        std::fs::write(&wrong, b"bytes que no son los de la foto").unwrap();
        remote.put(&kept.at, &wrong, Expect::Absent).unwrap();

        let first = one.round(Way::Both);
        one.round(Way::Both);

        let told = first.deferred.expect(who);
        assert_eq!(told.reason, Reason::Unsent, "{who}");
        assert!(told.names.contains(&kept.at), "{who}: {told:?}");
        let mut up = Vec::new();
        remote.fetch(&kept.at, 0, &mut up).unwrap();
        assert_eq!(
            up, PHOTO,
            "{who}: the wrong copy up there was warned about forever"
        );
    }
}

#[test]
fn a_full_cloud_while_an_attachment_is_owed_still_lets_the_documents_through_and_says_so() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo primero");
        one.round(Way::Both);
        two.round(Way::Both);
        two.filed(DOC, "# Plan\n\nlo de dos\n");
        two.round(Way::Both);
        let kept = one.kept("foto", PHOTO);
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        fake.refusing("attachments/", Hitch::Full);

        let moved = one.carry(Way::Both, Holds::Everywhere).expect(who);

        assert_eq!(
            one.doc(DOC).as_deref(),
            Some("# Plan\n\nlo de dos\n"),
            "{who}: a full drive held back what the others wrote"
        );
        assert!(
            matches!(moved.refused, Some(Trouble::CloudFull { .. })),
            "{who}: {:?}",
            moved.refused
        );
    }
}

#[test]
fn bringing_home_takes_everything_but_writes_down_only_what_the_log_answers_for() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let named = one.kept("foto", PHOTO);
        one.store().append(named.told()).unwrap();
        let stray = one.kept("suelta", b"nadie la nombra");
        one.wrote("con foto");
        one.round(Way::Both);

        let moved = two.brought_home();

        assert_eq!(two.held(&named.at).as_deref(), Some(PHOTO), "{who}");
        assert_eq!(
            two.held(&stray.at).as_deref(),
            Some(b"nadie la nombra".as_slice()),
            "{who}: leaving the cloud left a file behind in it"
        );
        assert!(
            moved.took_in.iter().any(|(at, _, _)| *at == named.at),
            "{who}: {:?}",
            moved.took_in
        );
        assert!(
            moved.took_in.iter().all(|(at, _, _)| *at != stray.at),
            "{who}: a name alone was written down as held: {:?}",
            moved.took_in
        );
    }
}

#[test]
fn a_wait_the_provider_asked_for_is_kept_by_the_next_rounds_without_asking_it_again() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        fake.fail_next(Hitch::Limited {
            wait: Duration::from_secs(600),
        });
        assert_eq!(
            one.round(Way::Both).deferred.map(|one| one.reason),
            Some(Reason::Limit),
            "{who}"
        );
        fake.forget_counts();

        let again = one.round(Way::Both);
        let polled = one.carrier.stirring();

        let deferred = again.deferred.expect(who);
        assert_eq!(deferred.reason, Reason::Limit, "{who}");
        assert!(deferred.retry_after.is_some(), "{who}");
        assert_eq!(polled, 0, "{who}");
        assert_eq!(
            fake.counts().requests,
            0,
            "{who}: the provider was asked again before its wait"
        );
        let _ = one.carrier.both_papers(&one.here, DOC);
        assert!(
            fake.counts().requests > 0,
            "{who}: what the person asked for waited on the window's own pace"
        );
        waited_out(&setting, "dev_a");
        assert_eq!(one.round(Way::Both).deferred, None, "{who}");
    }
}

#[test]
fn what_the_person_asks_for_on_a_spent_day_says_it_is_today_s_budget() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo primero\n");
        one.round(Way::Both);
        spent_today(&setting, "dev_a", fake.limits().units_a_day);

        let refused = one.carrier.both_papers(&one.here, DOC);

        assert!(
            matches!(refused, Err(Trouble::CloudSpent { .. })),
            "{who}: {refused:?}"
        );
    }
}

#[test]
fn a_forgetting_written_aside_while_its_list_was_held_is_still_forgotten() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        std::fs::remove_file(one.here.data.join("docs").join(format!("{DOC}.md"))).unwrap();
        let home = home_of(&setting, "dev_a");
        let aside = home.join(format!(
            "{}9-9{}",
            super::super::FORGOTTEN_ASIDE,
            super::super::ASIDE_LIST
        ));
        std::fs::write(&aside, format!("{DOC}\n")).unwrap();

        one.round(Way::Both);

        assert_eq!(remote.about(&doc_name()).unwrap(), None, "{who}");
        assert!(!aside.exists(), "{who}: read and kept");
    }
}

#[test]
fn a_machine_reading_between_a_segment_and_its_signature_comes_in_on_the_next_round() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        let mut written: Vec<String> = std::fs::read_dir(one.here.data.join(STORE).join("dev_a"))
            .unwrap()
            .filter_map(|one| one.ok()?.file_name().into_string().ok())
            .map(|leaf| format!("{STORE}/dev_a/{leaf}"))
            .filter(|name| super::super::mirror::mirrored(name))
            .collect();
        written.sort();
        let last = written.last().expect(who).clone();
        let meanwhile = Desk {
            here: two.here.clone(),
            carrier: two.carrier.clone(),
        };
        fake.in_the_gap(&last, move || {
            let _ = meanwhile.carry(Way::Both, Holds::Everywhere);
        });

        one.round(Way::Both);
        two.round(Way::Both);

        assert_eq!(
            two.titles(),
            ["lo de uno"],
            "{who}: what was read half way never came in"
        );
    }
}

#[test]
fn a_move_with_an_attachment_too_big_and_a_document_changed_meanwhile_is_not_ready() {
    let (setting, fake) = each_kind(|fake| fake).remove(0);
    let (one, _) = pair(&setting);
    one.filed(DOC, "# Plan\n\nlo primero\n");
    one.round(Way::Both);
    let big = tisty_core::attach::resolve("attachments/ab/grande-0123456789.bin", &one.here.data)
        .unwrap();
    std::fs::create_dir_all(big.parent().unwrap()).unwrap();
    std::fs::File::create(&big)
        .unwrap()
        .set_len(tisty_core::attach::HELD_UP_TO + 1)
        .unwrap();
    one.edited(DOC, "# Plan\n\nlo de uno\n");
    fake.refusing(&doc_name(), Hitch::Changed(doc_name()));
    let mut saying = |_| {};

    let shifted = crate::shift(
        None,
        Some(&*one.carrier),
        &one.here,
        &[DOC.to_string()],
        &mut saying,
    )
    .unwrap();

    assert!(
        matches!(shifted, crate::Shift::Filling { .. }),
        "a document left behind was hidden by an attachment that can never go: {shifted:?}"
    );
}
