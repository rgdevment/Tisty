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
        let reference = "attachments/grande-0123456789.bin";
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
        one.store()
            .append(kept.told(tisty_core::attach::Via::Window))
            .unwrap();
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
        one.store()
            .append(kept.told(tisty_core::attach::Via::Window))
            .unwrap();
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
        one.store()
            .append(named.told(tisty_core::attach::Via::Window))
            .unwrap();
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
    let big =
        tisty_core::attach::resolve("attachments/grande-0123456789.bin", &one.here.data).unwrap();
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

#[test]
fn an_index_of_an_older_layout_starts_the_mirror_over_into_the_new_space() {
    let (setting, _) = each_kind(|fake| fake).remove(0);
    let one = desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nantes\n");
    one.wrote("de la rc.1");
    one.round(Way::Both);
    let home = home_of(&setting, "dev_a");
    let mut old: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("index.json")).unwrap()).unwrap();
    old.as_object_mut().unwrap().remove("layout");
    std::fs::write(home.join("index.json"), old.to_string()).unwrap();
    let stale = home.join("tree").join("stale.md");
    std::fs::write(&stale, "de la carpeta vieja").unwrap();
    let space = Arc::new(Fake::drive());
    let anew = Desk {
        here: one.here.clone(),
        carrier: Arc::new(Cloud::over(space.clone(), home.clone())),
    };

    let done = anew.round(Way::Both);

    assert!(done.refused.is_none(), "{done:?}");
    assert!(!stale.exists(), "the old mirror was kept");
    assert!(
        space.about(&doc_name()).unwrap().is_some(),
        "the new space was taken for already in step: {:?}",
        space.uploaded()
    );
    let now: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("index.json")).unwrap()).unwrap();
    assert_eq!(
        now["layout"],
        serde_json::json!(crate::cloud::index::LAYOUT)
    );
}

#[test]
fn a_store_deleted_whole_up_there_is_said_so_and_sending_it_all_again_writes_it_back() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        one.filed(DOC, "# Plan\n\nlo escrito\n");
        one.wrote("algo");
        one.round(Way::Both);
        for there in fake.list("").unwrap() {
            fake.delete(&there.name, None).unwrap();
        }

        let stopped = one.carry(Way::Both, Holds::Everywhere);

        assert!(
            matches!(stopped, Err(Trouble::CloudEmptied { .. })),
            "{who}: {stopped:?}"
        );
        assert_eq!(one.titles(), ["algo"], "{who}: something here changed");
        let again = one.carry(Way::Again, Holds::Everywhere);
        assert!(again.is_ok(), "{who}: {again:?}");
        assert!(
            fake.about(&doc_name()).unwrap().is_some(),
            "{who}: sending it all again left the cloud empty"
        );
    }
}

fn emptied_up_there(one: &Desk, fake: &Fake) {
    one.filed(DOC, "# Plan\n\nlo escrito\n");
    one.wrote("algo");
    one.round(Way::Both);
    for there in fake.list("").unwrap() {
        fake.delete(&there.name, None).unwrap();
    }
    assert!(matches!(
        one.carry(Way::Both, Holds::Everywhere),
        Err(Trouble::CloudEmptied { .. })
    ));
}

#[test]
fn sending_it_all_again_turned_away_by_a_busy_round_is_not_carried_out_by_a_later_look() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        emptied_up_there(&one, &fake);
        let round = crate::cloud::lock::Round::take(&home_of(&setting, "dev_a"))
            .unwrap()
            .unwrap();
        let busy = one.round(Way::Again);
        assert_eq!(
            busy.deferred.map(|one| one.reason),
            Some(Reason::Busy),
            "{who}"
        );
        drop(round);

        one.carrier.kin(&one.here);

        assert!(
            matches!(
                one.carry(Way::Both, Holds::Everywhere),
                Err(Trouble::CloudEmptied { .. })
            ),
            "{who}: the mirror started over though nobody asked again"
        );
    }
}

#[test]
fn sending_it_all_again_put_off_by_the_provider_s_wait_is_not_carried_out_by_a_later_ask() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let one = desk(&setting, "dev_a");
        emptied_up_there(&one, &fake);
        crate::cloud::resting::rest(&home_of(&setting, "dev_a"), Duration::from_secs(60));
        let resting = one.round(Way::Again);
        assert_eq!(
            resting.deferred.map(|one| one.reason),
            Some(Reason::Limit),
            "{who}"
        );
        waited_out(&setting, "dev_a");

        let _ = one.carrier.stitch(&one.here, one.key());

        assert!(
            matches!(
                one.carry(Way::Both, Holds::Everywhere),
                Err(Trouble::CloudEmptied { .. })
            ),
            "{who}: the mirror started over though nobody asked again"
        );
    }
}

#[test]
fn a_store_written_back_up_there_by_another_machine_keeps_a_later_send_again_from_starting_over() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("algo");
        one.round(Way::Both);
        two.round(Way::Both);
        for there in fake.list("").unwrap() {
            fake.delete(&there.name, None).unwrap();
        }
        assert!(one.carry(Way::Both, Holds::Everywhere).is_err(), "{who}");
        assert!(two.carry(Way::Both, Holds::Everywhere).is_err(), "{who}");
        two.round(Way::Again);
        one.round(Way::Both);
        assert!(
            !home_of(&setting, "dev_a")
                .join(crate::cloud::EMPTIED)
                .exists(),
            "{who}: the store came back and the mark of its deletion stayed"
        );
        fake.forget_counts();

        one.round(Way::Again);

        assert_eq!(
            fake.counts().fetches,
            0,
            "{who}: sending it all again started the mirror over"
        );
    }
}

#[test]
fn a_cloud_read_from_another_root_starts_the_mirror_over_and_a_plain_index_says_no_root() {
    let (setting, _) = each_kind(|fake| fake).remove(0);
    let one = desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nantes\n");
    one.round(Way::Both);
    let home = home_of(&setting, "dev_a");
    let plain: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("index.json")).unwrap()).unwrap();
    assert!(plain.get("root").is_none(), "{plain}");
    let carried = home.join(crate::cloud::standing::CARRIED);
    std::fs::write(&carried, "1").unwrap();
    let stale = home.join("tree").join("stale.md");
    std::fs::write(&stale, "de la raíz vieja").unwrap();
    let key = tisty_share_proto::cloud::CloudKey::of(&tisty_share_proto::seed::Seed::from_bytes(
        &[7; 32],
    ));
    let space = Arc::new(crate::Sealed::new(Fake::drive(), key, "store_one"));
    let sealed = Desk {
        here: one.here.clone(),
        carrier: Arc::new(Cloud::over(space.clone(), home.clone())),
    };

    let done = sealed.round(Way::Both);

    assert!(done.refused.is_none(), "{done:?}");
    assert!(!stale.exists(), "the mirror of the old root was kept");
    assert!(
        carried.exists(),
        "a new root took this machine for a newcomer"
    );
    assert!(
        space.about(&doc_name()).unwrap().is_some(),
        "the other root was taken for already in step"
    );
    let now: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("index.json")).unwrap()).unwrap();
    assert_eq!(now["root"], serde_json::json!("sealed:store_one"));
}

fn sealed_desk(setting: &Setting, device: &str) -> (Desk, Arc<crate::Sealed<Fake>>) {
    let one = desk(setting, device);
    let space = Arc::new(crate::fakes::sealed_over(Fake::drive()));
    let home = home_of(setting, device);
    let sealed = Desk {
        here: one.here.clone(),
        carrier: Arc::new(Cloud::over(space.clone(), home)),
    };
    (sealed, space)
}

#[test]
fn a_round_into_a_sealed_cloud_leaves_its_mark_and_says_the_place_is_sealed() {
    let (setting, _) = each_kind(|fake| fake).remove(0);
    let (one, space) = sealed_desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nsellado\n");

    let done = one.round(Way::Both);

    assert!(done.refused.is_none(), "{done:?}");
    assert_eq!(
        space.marked().unwrap(),
        Some(tisty_share_proto::cloud::Mark::Holds)
    );
    let mut word = Vec::new();
    space
        .inner()
        .fetch(tisty_sync::NAMED, 0, &mut word)
        .unwrap();
    assert!(
        String::from_utf8(word)
            .unwrap()
            .contains("sealed = \"store_one\""),
        "the place does not say it is sealed"
    );
}

#[test]
fn a_sealed_cloud_whose_sealed_folder_was_deleted_whole_is_said_to_be_emptied() {
    let (setting, _) = each_kind(|fake| fake).remove(0);
    let (one, space) = sealed_desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nsellado\n");
    one.wrote("lo de uno");
    one.round(Way::Both);
    for gone in space.inner().list("sealed").unwrap() {
        space.inner().delete(&gone.name, None).unwrap();
    }

    let stopped = match one.carry(Way::Both, Holds::Everywhere) {
        Ok(moved) => moved.refused,
        Err(trouble) => Some(trouble),
    };

    assert!(
        matches!(stopped, Some(Trouble::CloudEmptied { .. })),
        "{stopped:?}"
    );
}

#[test]
fn a_round_into_a_closed_place_stops_before_it_puts_up_any_mark() {
    let (setting, _) = each_kind(|fake| fake).remove(0);
    let (one, space) = sealed_desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nsellado\n");
    let dir = tempfile::tempdir().unwrap();
    let word = dir.path().join("word");
    std::fs::write(
        &word,
        "shape = 3\nneeds = \"1.35.0\"\nsealing = \"closed:dev_b:abc\"\n",
    )
    .unwrap();
    space
        .inner()
        .put(tisty_sync::NAMED, &word, crate::Expect::Any)
        .unwrap();

    let stopped = match one.carry(Way::Both, Holds::Everywhere) {
        Ok(moved) => moved.refused,
        Err(trouble) => Some(trouble),
    };

    assert!(matches!(stopped, Some(Trouble::Sealing(_))), "{stopped:?}");
    assert_eq!(
        space.marked().unwrap(),
        None,
        "a mark was put up in a closed place"
    );
}
