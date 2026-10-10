struct Machine {
    _dir: tempfile::TempDir,
    data: PathBuf,
    store: PathBuf,
    device: String,
}

fn blank(named: &str) -> Machine {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    let store = data.join("store");
    Machine {
        _dir: dir,
        data,
        store,
        device: named.into(),
    }
}

fn machine(named: &str) -> Machine {
    let one = blank(named);
    wrote(&one, format!("lo de {named}"));
    one
}

fn filed(who: &Machine, file: &str, body: &str) {
    let mut held = signing(who);
    held.append(Op::DocAdd {
        id: Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: None,
            said: None,
            file: file.to_string(),
            order: "a0".into(),
            folder: None,
            page_of: None,
        },
    })
    .unwrap();
    tisty_core::docs::write(&who.data.join(PAPERS), file, body).unwrap();
}

fn planted(root: &Path, called: &str, body: &[u8]) -> String {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join(called);
    std::fs::write(&at, body).unwrap();
    tisty_core::attach::keep(&at, root, tisty_core::attach::COPIED_IN_DOC)
        .unwrap()
        .at
}

fn rotated(who: &Machine) {
    let at = who.store.join(&who.device).join("active.tisty");
    let held = tisty_core::store::read_tail(&at, 0).map_or(0, |all| all.len());
    let mut store = signing(who);
    store
        .append_batch(
            (held..5_000)
                .map(|n| Op::TaskAdd {
                    id: Ulid::generate(),
                    d: TaskAdd::new(format!("relleno {n}"), "a0"),
                })
                .collect(),
        )
        .unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo primero tras rotar", "a0"),
        })
        .unwrap();
    assert!(
        at.with_file_name("000001.tisty").is_file(),
        "it did not rotate"
    );
}

fn said_about(paths: &tisty_core::paths::Paths, who: &str) -> Vec<String> {
    tisty_core::witness::recent(paths, 200)
        .into_iter()
        .filter(|line| line.contains("a shorter history") && line.contains(who))
        .collect()
}

fn says(who: &Machine, op: Op) {
    let mut held = signing(who);
    held.append(op).unwrap();
}

fn signing(who: &Machine) -> Store {
    let whose = DeviceId(who.device.clone());
    let paths = tisty_core::Paths::new(who.data.clone(), who.data.join("config"));
    let key = tisty_core::signing::mine(&paths, &whose);
    if let Some(shown) = key.as_ref().map(tisty_core::signing::shown) {
        std::fs::create_dir_all(&who.data).unwrap();
        tisty_core::vouched::confirm(&who.data, &whose, &shown);
    }
    Store::open(&who.store, whose.clone())
        .unwrap()
        .signing_with(key)
}

fn signs(who: &Machine, alias: &str) {
    says(
        who,
        Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some(alias.to_string()),
                name: None,
                email: None,
            },
        },
    );
}

fn titles(store: &Path) -> Vec<String> {
    tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
        .tasks
        .values()
        .map(|task| task.title.clone())
        .collect()
}

fn trailing(theirs: &Path) {
    let at = theirs.join("active.tisty");
    let whole = std::fs::read_to_string(&at).unwrap();
    let first = whole.lines().next().unwrap();
    std::fs::write(
        &at,
        format!(
            "{first}
"
        ),
    )
    .unwrap();
}

fn watching(at: &Path) -> tisty_core::paths::Paths {
    let paths = tisty_core::paths::Paths::new(at.join("data"), at.join("config"));
    witness::keeps(tisty_core::witness::file(&paths), false);
    paths
}

fn wrote(who: &Machine, title: String) {
    let mut held = signing(who);
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new(title, "a0"),
    })
    .unwrap();
}

fn a_shorter_history_arriving_first_never_replaces_the_longer_one_we_hold(shared: &Shared) {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();
    let held = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();
    assert!(held >= 4);

    let theirs = shared.path().join(STORE).join(&one.device);
    let at = theirs.join("active.tisty");
    let whole = std::fs::read_to_string(&at).unwrap();
    let first = whole.lines().next().unwrap();
    std::fs::write(&at, format!("{first}\n")).unwrap();

    shared.carry(&two, Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
        held,
        "una historia mas corta piso la que ya teniamos"
    );
}

fn a_shared_folder_merely_behind_ours_is_pushed_to_without_a_word(shared: &Shared) {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();
    trailing(&shared.path().join(STORE).join(&one.device));

    let kept = tempfile::tempdir().unwrap();
    let paths = watching(kept.path());
    shared.carry(&two, Way::Both, &[]).unwrap();

    assert!(said_about(&paths, &one.device).is_empty());
    assert_eq!(
        tisty_core::store::check_device(&shared.path().join(STORE).join(&one.device)).unwrap(),
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
    );
}

fn a_shared_folder_that_walked_off_on_its_own_is_still_reported(shared: &Shared) {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();

    let apart = blank("uno");
    wrote(&apart, "otra vida".into());
    let theirs = shared.path().join(STORE).join(&one.device);
    std::fs::copy(
        apart.store.join(&apart.device).join("active.tisty"),
        theirs.join("active.tisty"),
    )
    .unwrap();

    let kept = tempfile::tempdir().unwrap();
    let paths = watching(kept.path());
    shared.carry(&two, Way::Pull, &[]).unwrap();

    assert!(!said_about(&paths, &one.device).is_empty());
}

fn a_history_that_grew_on_the_other_side_still_comes_across(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();
    let before = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();

    wrote(&one, "algo mas".into());
    shared.carry(&one, Way::Push, &[]).unwrap();
    shared.carry(&two, Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
        before + 1
    );
}

fn a_retired_attachment_is_never_pushed_back_up_to_the_folder(shared: &Shared) {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia retirada");

    says(&one, Op::AttachRetire { d: kept.clone() });
    shared.carry(&one, Way::Both, &[]).unwrap();

    assert!(
        !shared.path().join(&kept).exists(),
        "lo retirado se subio a la carpeta compartida"
    );
}

fn an_attachment_nobody_retired_still_goes_up(shared: &Shared) {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia cualquiera");

    shared.carry(&one, Way::Both, &[]).unwrap();

    assert!(shared.path().join(&kept).is_file());
}

fn a_pull_cut_between_two_segments_does_not_wedge_the_next_one(shared: &Shared) {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();

    rotated(&one);
    let theirs = shared.path().join(STORE).join(&one.device);
    let mine = two.store.join(&one.device);
    shared.carry(&one, Way::Push, &[]).unwrap();
    std::fs::copy(theirs.join("000001.tisty"), mine.join("000001.tisty")).unwrap();

    wrote(&one, "lo que vino despues de rotar".into());
    shared.carry(&one, Way::Push, &[]).unwrap();
    shared.carry(&two, Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::distinct_in(&mine).unwrap(),
        tisty_core::store::distinct_in(&theirs).unwrap(),
        "la maquina quedo atascada y no recibe nada mas"
    );
}

fn a_torn_local_copy_is_never_replaced_by_a_shorter_history(shared: &Shared) {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    shared.carry(&one, Way::Push, &[]).unwrap();

    let two = blank("dos");
    shared.carry(&two, Way::Pull, &[]).unwrap();

    let mine = two.store.join(&one.device).join("active.tisty");
    let whole = std::fs::read_to_string(&mine).unwrap();
    std::fs::write(&mine, format!("{whole}{{\"v\":3,\"ts\"")).unwrap();

    let theirs = shared
        .path()
        .join(STORE)
        .join(&one.device)
        .join("active.tisty");
    let short = std::fs::read_to_string(&theirs).unwrap();
    let first = short.lines().next().unwrap();
    std::fs::write(&theirs, format!("{first}\n")).unwrap();

    shared.carry(&two, Way::Pull, &[]).unwrap();

    let after = std::fs::read_to_string(&mine).unwrap();
    assert!(
        after.starts_with(&whole),
        "una historia corta piso la copia local rota"
    );
}

fn a_clash_is_found_even_when_another_machine_already_proved_the_lineage(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let both = "compartida";
    for at in [one.store.join(both), shared.path().join(STORE).join(both)] {
        std::fs::create_dir_all(&at).unwrap();
    }
    std::fs::write(one.store.join(both).join("active.tisty"), b"lo de aqui\n").unwrap();
    std::fs::write(
        shared.path().join(STORE).join(both).join("active.tisty"),
        b"lo de alli\n",
    )
    .unwrap();

    assert_eq!(
        shared.kin(&one),
        Kin::Clash(both.to_string()),
        "un choque real se perdio porque otra maquina ya habia probado el linaje"
    );
}

fn the_base_never_travels_to_the_shared_folder(shared: &Shared) {
    let one = machine("uno");
    filed(&one, "uno-0001", "# Kit\n\ncuerpo\n");

    shared.carry(&one, Way::Both, &[]).unwrap();

    assert!(!shared.path().join("carried").exists());
    assert!(one.data.join("carried").is_dir());
}

fn two_histories_that_never_met_are_strangers(shared: &Shared) {
    let one = machine("uno");
    let two = machine("dos");
    shared.carry(&two, Way::Push, &[]).unwrap();

    assert_eq!(shared.kin(&one), Kin::Strangers);
}

fn a_folder_that_already_holds_everything_of_ours_is_the_same_lineage(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    assert_eq!(shared.kin(&one), Kin::SameLineage);
}

fn a_tail_we_never_sent_is_still_the_same_lineage_not_a_clash(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();
    wrote(&one, "algo que se quedo aqui".into());

    assert_eq!(shared.kin(&one), Kin::SameLineage);
}

fn the_same_name_writing_two_different_things_is_the_clash_that_is_refused(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
    let mut said = std::fs::read(&file).unwrap();
    said[0] ^= 0xff;
    std::fs::write(&file, said).unwrap();

    assert_eq!(shared.kin(&one), Kin::Clash(one.device.clone()));
}

fn a_folder_ahead_of_us_is_the_same_lineage_because_only_the_end_grows(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
    let mut said = std::fs::read(&file).unwrap();
    said.extend_from_slice(b"{\"v\":3}\n");
    std::fs::write(&file, said).unwrap();

    assert_eq!(shared.kin(&one), Kin::SameLineage);
}

fn a_document_written_on_the_other_machine_lands_on_the_first_sync_not_the_second(shared: &Shared) {
    let one = machine("uno");
    let two = blank("dos");

    filed(&one, "uno-0001", "# Ortografia\n\nla n con virgulilla\n");
    shared.carry(&one, Way::Both, &["uno-0001".into()]).unwrap();

    shared.carry(&two, Way::Both, &[]).unwrap();

    let landed = two.data.join(PAPERS).join("uno-0001.md");
    assert!(
        landed.is_file(),
        "el documento no llego en la primera vuelta"
    );
    assert_eq!(
        std::fs::read_to_string(landed).unwrap(),
        "# Ortografia\n\nla n con virgulilla\n"
    );
}

fn a_document_the_other_machine_deleted_is_never_brought_back_by_the_new_reckoning(
    shared: &Shared,
) {
    let one = machine("uno");
    let two = blank("dos");

    filed(&one, "uno-0001", "# Algo\n");
    shared.carry(&one, Way::Both, &["uno-0001".into()]).unwrap();

    let mut held = signing(&one);
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .next()
        .unwrap()
        .id;
    held.append(Op::DocDelete { id }).unwrap();
    drop(held);
    shared.carry(&one, Way::Both, &[]).unwrap();

    shared.carry(&two, Way::Both, &[]).unwrap();

    assert!(!two.data.join(PAPERS).join("uno-0001.md").exists());
}

fn a_folder_says_it_stirred_without_being_read(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let still = shared.stirring();
    assert_eq!(still, shared.stirring(), "a folder at rest changed");

    wrote(&one, "lo que vino despues".into());
    shared.carry(&one, Way::Push, &[]).unwrap();

    assert_ne!(still, shared.stirring(), "the folder grew and said nothing");
}

fn a_seat_nobody_ever_wrote_in_is_not_taken_home(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Both, &[]).unwrap();
    std::fs::create_dir_all(shared.path().join(STORE).join("dev_ghost")).unwrap();

    let fresh = blank("dos");
    shared.carry(&fresh, Way::Pull, &[]).unwrap();

    assert!(
        !fresh.store.join("dev_ghost").exists(),
        "an empty device directory travelled as though it were a machine"
    );
}

fn a_machine_that_wrote_nothing_takes_up_the_folder_it_is_pointed_at(shared: &Shared) {
    let one = machine("uno");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let fresh = blank("dos");
    shared.carry(&fresh, Way::Both, &[]).unwrap();

    assert_eq!(titles(&fresh.store), vec!["lo de uno".to_string()]);
    assert_eq!(
        shared.signed_at(),
        None,
        "nobody signed, so there is no name to take"
    );
}

fn signing_before_the_folder_is_chosen_makes_a_new_machine_look_like_another_history(
    shared: &Shared,
) {
    let one = machine("uno");
    signs(&one, "mario");
    shared.carry(&one, Way::Push, &[]).unwrap();

    let fresh = blank("dos");
    signs(&fresh, "mario");
    let outcome = shared.carry(&fresh, Way::Both, &[]);

    assert!(
        matches!(
            outcome,
            Err(Trouble::OtherStore { .. }) | Err(Trouble::WouldReset { .. })
        ),
        "a name of its own is a history of its own: {outcome:?}"
    );
    assert_eq!(
        shared.signed_at().as_deref(),
        Some("mario"),
        "the folder carries the name, so asking for one first is asking twice"
    );
}

fn a_store_with_no_list_yet_lets_everyone_write(shared: &Shared) {
    let one = machine("dev_a");

    let moved = shared.carry(&one, Way::Both, &[]).unwrap();

    assert!(moved.sent > 0, "an older store must not be locked out");
}

fn a_machine_on_the_list_writes_as_it_always_did(shared: &Shared) {
    let one = machine("dev_a");
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_a".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );

    let moved = shared.carry(&one, Way::Both, &[]).unwrap();

    assert!(moved.sent > 0);
}

both_sides! {
    a_shorter_history_arriving_first_never_replaces_the_longer_one_we_hold,
    a_shared_folder_merely_behind_ours_is_pushed_to_without_a_word,
    a_shared_folder_that_walked_off_on_its_own_is_still_reported,
    a_history_that_grew_on_the_other_side_still_comes_across,
    a_retired_attachment_is_never_pushed_back_up_to_the_folder,
    an_attachment_nobody_retired_still_goes_up,
    a_pull_cut_between_two_segments_does_not_wedge_the_next_one,
    a_torn_local_copy_is_never_replaced_by_a_shorter_history,
    the_base_never_travels_to_the_shared_folder,
    two_histories_that_never_met_are_strangers,
    a_folder_that_already_holds_everything_of_ours_is_the_same_lineage,
    a_tail_we_never_sent_is_still_the_same_lineage_not_a_clash,
    a_folder_ahead_of_us_is_the_same_lineage_because_only_the_end_grows,
    a_document_written_on_the_other_machine_lands_on_the_first_sync_not_the_second,
    a_document_the_other_machine_deleted_is_never_brought_back_by_the_new_reckoning,
    a_seat_nobody_ever_wrote_in_is_not_taken_home,
    a_machine_that_wrote_nothing_takes_up_the_folder_it_is_pointed_at,
    signing_before_the_folder_is_chosen_makes_a_new_machine_look_like_another_history,
    a_store_with_no_list_yet_lets_everyone_write,
    a_machine_on_the_list_writes_as_it_always_did,
}

// The cloud answers these from its mirror until it learns to look again between rounds.
cloud_owes! {
    a_clash_is_found_even_when_another_machine_already_proved_the_lineage,
    a_folder_says_it_stirred_without_being_read,
    the_same_name_writing_two_different_things_is_the_clash_that_is_refused,
}
