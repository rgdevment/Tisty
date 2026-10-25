use super::*;

fn a_place() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    let dest = room.path().join("shared");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(dest.join(STORE)).unwrap();
    (room, data, dest)
}

#[test]
fn a_folder_from_before_this_file_existed_is_adopted_rather_than_refused() {
    let (_room, data, dest) = a_place();
    std::fs::create_dir_all(dest.join(STORE).join("dev_b")).unwrap();

    before_reading(&data, &dest).expect("a folder written by an older build was turned away");

    stamp(&data, &dest);
    assert!(
        dest.join(NAMED).is_file(),
        "it was adopted without saying so"
    );
    before_reading(&data, &dest).expect("what it stamped itself it then refused");
}

#[test]
fn a_shape_this_build_does_not_know_stops_it_before_a_byte_is_written() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), format!("shape = {}\n", OURS + 1)).unwrap();

    assert!(
        matches!(before_reading(&data, &dest), Err(Trouble::Shape(_))),
        "a folder arranged by a build that knows more was read anyway"
    );
}

#[test]
fn a_folder_that_said_its_shape_and_went_quiet_is_not_read_as_empty() {
    let (_room, data, dest) = a_place();
    std::fs::create_dir_all(dest.join(STORE).join("dev_b")).unwrap();
    stamp(&data, &dest);
    before_reading(&data, &dest).unwrap();

    std::fs::remove_file(dest.join(NAMED)).unwrap();

    assert!(
        matches!(before_reading(&data, &dest), Err(Trouble::Unshaped(_))),
        "a folder whose shape went missing read as one with nothing in it"
    );
}

#[test]
fn another_folder_from_before_this_file_is_still_one_to_take_up() {
    let (room, data, dest) = a_place();
    stamp(&data, &dest);
    before_reading(&data, &dest).unwrap();

    let older = room.path().join("older");
    std::fs::create_dir_all(older.join(STORE).join("dev_b")).unwrap();

    before_reading(&data, &older).expect(
        "a machine that had met one folder saying its shape could never take up an older one",
    );
}

#[test]
fn what_says_a_shape_and_cannot_be_read_is_not_taken_for_nothing() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "folders = []\n").unwrap();

    assert!(matches!(
        before_reading(&data, &dest),
        Err(Trouble::Unshaped(_))
    ));
}

fn said_in(dest: &Path) -> String {
    std::fs::read_to_string(dest.join(NAMED)).unwrap()
}

#[test]
fn a_folder_that_already_says_this_shape_is_not_told_again() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        format!("shape = {OURS}\n# as it was left\n"),
    )
    .unwrap();

    stamp(&data, &dest);

    assert_eq!(
        said_in(&dest),
        format!("shape = {OURS}\n# as it was left\n")
    );
}

#[test]
fn a_folder_that_says_more_than_this_build_knows_is_not_told_over() {
    let (_room, data, dest) = a_place();
    let more = format!("shape = {}\n", OURS + 1);
    std::fs::write(dest.join(NAMED), &more).unwrap();

    stamp(&data, &dest);

    assert_eq!(said_in(&dest), more);
}

#[test]
fn a_folder_that_says_something_unreadable_is_not_told_over() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "folders = []\n").unwrap();

    stamp(&data, &dest);

    assert_eq!(said_in(&dest), "folders = []\n");
}

#[test]
fn a_folder_that_says_less_than_this_build_knows_is_told_what_shape_it_is_in() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 0\n").unwrap();

    stamp(&data, &dest);

    assert!(said_in(&dest).contains(&format!("shape = {OURS}")));
}

#[test]
fn a_folder_that_was_not_told_but_already_says_its_shape_is_still_remembered() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), format!("shape = {OURS}\n")).unwrap();

    stamp(&data, &dest);
    std::fs::remove_file(dest.join(NAMED)).unwrap();

    assert!(matches!(
        before_reading(&data, &dest),
        Err(Trouble::Unshaped(_))
    ));
}

#[test]
fn a_release_is_later_only_when_its_numbers_say_so_and_one_unread_is_taken_for_later() {
    assert!(!needs_later(NEEDS));
    assert!(!needs_later("1.34.1"));
    assert!(!needs_later("1.35.0-rc.1"));
    assert!(!needs_later(SEALS));
    assert!(!needs_later(TURNS));
    assert!(needs_later("1.36.1"));
    assert!(needs_later("1.100.0"));
    assert!(needs_later("2.0.0"));
    assert!(needs_later("pronto"));
}

#[test]
fn a_place_that_asks_for_this_release_is_read_and_one_that_asks_for_a_later_one_is_not() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        format!("shape = {OURS}\nneeds = \"{NEEDS}\"\n"),
    )
    .unwrap();
    before_reading(&data, &dest).expect("a place this build arranged turned it away");

    std::fs::write(dest.join(NAMED), "shape = 9\nneeds = \"7.1.0\"\n").unwrap();

    assert_eq!(
        before_reading(&data, &dest),
        Err(Trouble::Needs("7.1.0".into())),
        "the later release was not named, only a shape nobody can act on"
    );
}

#[test]
fn a_place_older_builds_arranged_is_raised_and_one_already_raised_is_not() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 1\n").unwrap();

    assert!(stamp(&data, &dest));
    assert!(said_in(&dest).contains(&format!("needs = \"{NEEDS}\"")));
    assert!(!stamp(&data, &dest));
}

#[test]
fn a_place_left_is_raised_for_the_older_builds_and_a_newer_or_empty_one_is_not() {
    let (_room, _data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 1\n").unwrap();
    raise_place(&dest);
    assert!(said_in(&dest).contains(&format!("shape = {OURS}")));

    let more = format!("shape = {}\n", OURS + 1);
    std::fs::write(dest.join(NAMED), &more).unwrap();
    raise_place(&dest);
    assert_eq!(said_in(&dest), more);

    let room = tempfile::tempdir().unwrap();
    raise_place(room.path());
    assert!(
        !room.path().join(NAMED).exists(),
        "an empty folder was arranged"
    );
}

#[test]
fn only_another_machine_s_history_in_the_place_is_someone_left_behind() {
    let (_room, _data, dest) = a_place();
    std::fs::create_dir_all(dest.join(STORE).join("dev_a")).unwrap();
    assert!(!shared_with_others(&dest, "dev_a"));

    std::fs::create_dir_all(dest.join(STORE).join("dev_b")).unwrap();
    assert!(shared_with_others(&dest, "dev_a"));
}

#[test]
fn a_new_place_is_arranged_without_warning_anyone() {
    let (_room, data, dest) = a_place();

    assert!(!stamp(&data, &dest));
    assert!(said_in(&dest).contains(&format!("shape = {OURS}")));
}

#[test]
fn a_place_is_raised_first_only_when_it_already_says_an_older_shape() {
    let (_room, data, dest) = a_place();
    assert!(!raise_first(&data, &dest));
    assert!(
        !dest.join(NAMED).exists(),
        "a new place was named before its round finished"
    );

    std::fs::write(dest.join(NAMED), "shape = 2\n").unwrap();
    assert!(raise_first(&data, &dest));
    assert!(said_in(&dest).contains(&format!("needs = \"{NEEDS}\"")));
    assert!(
        !raise_first(&data, &dest),
        "a place already raised was raised again"
    );
}

#[test]
fn what_a_later_tisty_added_to_the_place_survives_a_round_here() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        "shape = 2\nneeds = \"1.30.0\"\nsealed = true\n# said by a later build\n",
    )
    .unwrap();

    assert!(raise_first(&data, &dest));

    let said = std::fs::read_to_string(dest.join(NAMED)).unwrap();
    assert!(said.contains("sealed = true\n"), "{said}");
    assert!(said.contains("# said by a later build\n"), "{said}");
    assert!(said.contains(&format!("needs = \"{NEEDS}\"")), "{said}");
    assert_eq!(said.matches("shape =").count(), 1, "{said}");
}

#[test]
fn a_place_asking_for_a_later_tisty_is_refused_outside_a_round_too() {
    let (_room, _data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        format!("shape = {OURS}\nneeds = \"999.0.0\"\n"),
    )
    .unwrap();

    assert_eq!(refuses(&dest), Err(Trouble::Needs("999.0.0".to_string())));
}

#[test]
fn a_place_that_says_nothing_or_our_shape_is_not_refused() {
    let (_room, data, dest) = a_place();
    assert_eq!(refuses(&dest), Ok(()));

    stamp(&data, &dest);

    assert_eq!(refuses(&dest), Ok(()));
}

#[test]
fn a_later_builds_tables_and_spread_arrays_survive_a_round_here() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        "shape = 2\nfolders = [\n  \"store\",\n  \"attachments\",\n]\n[seal]\nneeds = \"1.40.0\"\nshape = 1\n",
    )
    .unwrap();

    assert!(raise_first(&data, &dest));

    let said = std::fs::read_to_string(dest.join(NAMED)).unwrap();
    assert!(
        said.ends_with("[seal]\nneeds = \"1.40.0\"\nshape = 1\n"),
        "{said}"
    );
    assert!(!said.contains("  \"store\",\n"), "{said}");
    assert_eq!(said.matches("folders =").count(), 1, "{said}");
    assert!(
        matches!(told(&dest), Told::Shape { shape: OURS, needs: Some(needs), .. } if needs == NEEDS)
    );
}

#[test]
fn a_place_whose_word_cannot_be_opened_just_now_is_not_there_rather_than_unshaped() {
    let (_room, _data, dest) = a_place();
    std::fs::create_dir_all(dest.join(NAMED)).unwrap();

    assert!(matches!(refuses(&dest), Err(Trouble::NotThere(_))));
}

#[test]
fn a_word_that_is_not_text_is_unshaped_rather_than_not_there() {
    let (_room, _data, dest) = a_place();
    std::fs::write(dest.join(NAMED), b"shape = 3\n\xff\xfe\n").unwrap();

    assert!(matches!(refuses(&dest), Err(Trouble::Unshaped(_))));
}

#[test]
fn a_later_builds_nested_arrays_do_not_hide_what_it_needs() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        "shape = 2\nmachines = [\n  [\"x\", 1],\n  [\"y\", 2],\n]\nneeds = \"1.30.0\"\n",
    )
    .unwrap();

    assert!(raise_first(&data, &dest));

    let said = std::fs::read_to_string(dest.join(NAMED)).unwrap();
    assert!(
        said.contains("machines = [\n  [\"x\", 1],\n  [\"y\", 2],\n]\n"),
        "{said}"
    );
    assert_eq!(said.matches("needs =").count(), 1, "{said}");

    std::fs::write(
        dest.join(NAMED),
        "shape = 3\nmachines = [\n  [\"x\", 1],\n]\nneeds = \"999.0.0\"\n",
    )
    .unwrap();
    assert_eq!(refuses(&dest), Err(Trouble::Needs("999.0.0".to_string())));
}

#[test]
fn brackets_inside_a_later_builds_strings_do_not_hide_what_it_needs() {
    let (_room, _data, dest) = a_place();
    for said in [
        "shape = 3\nnote = 'see [docs'\nneeds = \"999.0.0\"\n",
        "shape = 3\ndesc = \"\"\"\n[note] text\n\"\"\"\nneeds = \"999.0.0\"\n",
    ] {
        std::fs::write(dest.join(NAMED), said).unwrap();

        assert_eq!(
            refuses(&dest),
            Err(Trouble::Needs("999.0.0".to_string())),
            "{said}"
        );
    }
}

#[test]
fn a_needs_this_build_cannot_read_is_taken_for_a_later_one() {
    let (_room, _data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 3\nneeds = 5\n").unwrap();

    assert!(matches!(refuses(&dest), Err(Trouble::Needs(_))));
}

#[test]
fn a_later_word_this_parser_cannot_read_still_says_what_it_needs() {
    let (_room, data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        "shape = 3\nneeds = \"999.0.0\"\nseal = { a = 1,\n b = 2 }\n",
    )
    .unwrap();
    let needs = Some(Trouble::Needs("999.0.0".to_string()));

    assert_eq!(later(&dest), needs);
    assert_eq!(refuses(&dest).err(), needs);
    assert_eq!(before_reading(&data, &dest).err(), needs);
}

#[test]
fn a_word_that_cannot_be_read_and_asks_nothing_later_is_unshaped_not_later() {
    let (_room, _data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = = 3\n").unwrap();

    assert_eq!(later(&dest), None);
    assert!(matches!(refuses(&dest), Err(Trouble::Unshaped(_))));
}

#[test]
fn a_needs_a_later_build_wrote_is_never_lowered_when_the_place_is_raised() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 2\nneeds = \"999.0.0\"\n").unwrap();

    assert!(raise_first(&data, &dest));

    let said = std::fs::read_to_string(dest.join(NAMED)).unwrap();
    assert!(said.contains("needs = \"999.0.0\""), "{said}");
    assert!(said.contains(&format!("shape = {OURS}")), "{said}");
}

#[test]
fn only_an_older_needs_is_raised() {
    assert!(needs_earlier("1.30.0"));
    assert!(!needs_earlier(NEEDS));
    assert!(!needs_earlier("999.0.0"));
    assert!(!needs_earlier("not a release"));
}

#[test]
fn a_word_1_33_read_that_this_parser_does_not_is_still_read_as_it_was() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "shape = 03\nneeds = \"1.31.0\"\n").unwrap();
    assert_eq!(refuses(&dest), Ok(()));
    assert_eq!(before_reading(&data, &dest), Ok(()));

    std::fs::write(dest.join(NAMED), "shape = 2\nneeds = 1.30.0\n").unwrap();
    assert!(raise_first(&data, &dest));
    assert!(matches!(
        told(&dest),
        Told::Shape {
            shape: OURS,
            said: Some(_),
            ..
        }
    ));
}

#[test]
fn what_writes_outside_a_round_stops_at_a_place_a_later_build_raised() {
    let (_room, data, dest) = a_place();
    std::fs::create_dir_all(dest.join(PAPERS)).unwrap();
    let said = format!("shape = {OURS}\nneeds = \"999.0.0\"\n");
    std::fs::write(dest.join(NAMED), &said).unwrap();
    let needs = Some(Trouble::Needs("999.0.0".to_string()));

    assert_eq!(crate::stitch(&data, "dev_a", &dest, None).err(), needs);
    assert_eq!(crate::both_papers(&data, &dest, "dev_a-0001").err(), needs);
    assert_eq!(
        crate::settle(&data, &dest, "dev_a-0001", crate::Keep::Both, None).err(),
        needs
    );
    assert_eq!(
        crate::let_go_telling(&data, &dest, 0, &|_| false, &mut |_| true).err(),
        needs
    );
    assert_eq!(std::fs::read_to_string(dest.join(NAMED)).unwrap(), said);
}

#[test]
fn a_beta_comes_before_its_release_and_build_marks_change_nothing() {
    assert!(tisty_core::released("1.35.0-beta.1") < tisty_core::released("1.35.0"));
    assert_eq!(
        tisty_core::released("1.35.0-rc.2"),
        tisty_core::released("1.35.0-rc.1")
    );
    assert!(tisty_core::released("1.34.9") < tisty_core::released("1.35.0-beta.1"));
    assert_eq!(
        tisty_core::released("1.35.0+abc"),
        tisty_core::released("1.35.0")
    );
}

#[test]
fn a_place_says_whether_it_is_plain_being_sealed_closed_or_sealed() {
    let (_room, _data, dest) = a_place();
    let says = |word: &str| {
        std::fs::write(
            dest.join(NAMED),
            format!("shape = 3\nneeds = \"1.34.0\"\n{word}"),
        )
        .unwrap();
        sealing_of(&dest)
    };

    assert_eq!(says(""), Sealing::Plain);
    assert_eq!(
        says("sealed = \"store_one\"\nbroken = = =\n"),
        Sealing::Sealed("store_one".into())
    );
    assert_eq!(
        says("sealing = \"copying:seal:dev_a:n1\"\n"),
        Sealing::Copying(Turn::On)
    );
    assert_eq!(
        says("sealed = \"store_one\"\nsealing = \"copying:unseal:dev_a:n1\"\n"),
        Sealing::Copying(Turn::Off)
    );
    assert_eq!(
        says("sealing = \"copying:dev_a:n1\"\n"),
        Sealing::Closed,
        "a copy whose way is not said stops everyone"
    );
    assert_eq!(says("sealing = \"closed:dev_a:n1\"\n"), Sealing::Closed);
    assert_eq!(
        says("sealed = \"store_one\"\n"),
        Sealing::Sealed("store_one".into())
    );
    assert_eq!(
        says("sealed = \"store_one\"\nsealing = \"closed:dev_a:n2\"\n"),
        Sealing::Closed
    );
}

#[test]
fn a_reader_stops_where_the_place_is_sealed_another_way_and_never_writes_in_the_clear_on_its_word()
{
    let (_room, _data, dest) = a_place();
    let against = |word: &str, reading: Option<&str>| {
        std::fs::write(dest.join(NAMED), format!("shape = 3\n{word}")).unwrap();
        sealed_against(&dest, reading).map(|trouble| trouble.code())
    };

    assert_eq!(sealed_against(&dest, Some("store_one")), None);
    assert_eq!(against("", None), None);
    assert_eq!(against("", Some("store_one")), Some("unsealed"));
    assert_eq!(against("sealed = \"store_one\"\n", None), Some("sealed"));
    assert_eq!(against("sealed = \"store_one\"\n", Some("store_one")), None);
    assert_eq!(
        against("sealed = \"store_one\"\n", Some("store_two")),
        Some("sealed")
    );
    assert_eq!(against("sealing = \"copying:seal:dev_a:n1\"\n", None), None);
    assert_eq!(
        against("sealing = \"copying:seal:dev_a:n1\"\n", Some("store_one")),
        Some("sealing"),
        "a sealed reader never writes into a copy still being filled"
    );
    assert_eq!(
        against(
            "sealed = \"store_one\"\nsealing = \"copying:unseal:dev_a:n1\"\n",
            Some("store_one")
        ),
        None
    );
    assert_eq!(
        against(
            "sealed = \"store_one\"\nsealing = \"copying:unseal:dev_a:n1\"\n",
            None
        ),
        Some("sealing"),
        "a plain reader never writes into the plain copy being filled"
    );
    assert_eq!(
        against("sealing = \"closed:dev_a:n1\"\n", None),
        Some("sealing")
    );
    assert_eq!(
        against("sealing = \"closed:dev_a:n1\"\n", Some("store_one")),
        Some("sealing")
    );
}

#[test]
fn a_word_written_for_a_sealed_place_says_so_once_raises_needs_and_keeps_the_rest() {
    let (_room, _data, dest) = a_place();
    seal_word(&dest, "store_one").unwrap();
    assert!(
        !dest.join(NAMED).exists(),
        "a word was made up where none was said"
    );
    std::fs::write(dest.join(NAMED), "shape = 03\nneeds = 1.30.0\n").unwrap();
    seal_word(&dest, "store_one").unwrap();
    assert_eq!(sealing_of(&dest), Sealing::Sealed("store_one".into()));
    assert!(matches!(
        told(&dest),
        Told::Shape {
            shape: OURS,
            said: Some(_),
            ..
        }
    ));
    std::fs::write(
        dest.join(NAMED),
        "shape = 3\nneeds = \"1.34.0\"\nkept = 1\n",
    )
    .unwrap();

    seal_word(&dest, "store_one").unwrap();
    seal_word(&dest, "store_two").unwrap();

    let said = std::fs::read_to_string(dest.join(NAMED)).unwrap();
    assert!(said.contains("kept = 1"), "{said}");
    assert!(said.contains(&format!("needs = \"{SEALS}\"")), "{said}");
    assert_eq!(sealing_of(&dest), Sealing::Sealed("store_one".into()));
}

#[test]
fn a_sealing_phase_this_build_cannot_name_stops_everyone_rather_than_no_one() {
    let (_room, _data, dest) = a_place();
    std::fs::write(
        dest.join(NAMED),
        "shape = 3\nsealed = \"store_one\"\nsealing = \"\"\n",
    )
    .unwrap();

    assert_eq!(sealing_of(&dest), Sealing::Closed);
    assert_eq!(
        sealed_against(&dest, None).map(|one| one.code()),
        Some("sealing")
    );
}

#[test]
fn a_word_read_only_line_by_line_that_already_seals_is_left_as_it_is() {
    let (_room, _data, dest) = a_place();
    let said = "shape = 3\nsealing = \"copying:dev_b:n1\"\nbroken = = =\n";
    std::fs::write(dest.join(NAMED), said).unwrap();

    seal_word(&dest, "store_one").unwrap();

    assert_eq!(std::fs::read_to_string(dest.join(NAMED)).unwrap(), said);
}

#[test]
fn a_turn_is_claimed_closed_and_switched_only_by_the_machine_that_claimed_it() {
    let word = "shape = 3\nneeds = \"1.34.0\"\nunknown = \"kept\"\n";
    let ours = phase(Turn::On, "dev_a", "n1");

    let claimed_word = claimed(word, &ours).unwrap();
    assert_eq!(sealing_in(&claimed_word), Sealing::Copying(Turn::On));
    assert!(claimed(&claimed_word, &phase(Turn::On, "dev_b", "n2")).is_none());
    assert!(closed(&claimed_word, "dev_b:n2").is_none());

    let closed_word = closed(&claimed_word, "dev_a:n1").unwrap();
    assert_eq!(sealing_in(&closed_word), Sealing::Closed);
    assert!(closed_word.contains("needs = \"1.36.0\""));

    let sealed_word = switched(&closed_word, "dev_a:n1", Some("store_one")).unwrap();
    assert_eq!(
        sealing_in(&sealed_word),
        Sealing::Sealed("store_one".into())
    );
    assert!(sealed_word.contains("unknown = \"kept\""));
    assert!(
        switched(&claimed_word, "dev_a:n1", Some("store_one")).is_none(),
        "switched before it closed"
    );
}

#[test]
fn turning_off_takes_the_sealed_line_away_and_never_lowers_needs() {
    let word = "shape = 3\nneeds = \"1.36.0\"\nsealed = \"store_one\"\n";
    let ours = phase(Turn::Off, "dev_a", "n1");

    let claimed_word = claimed(word, &ours).unwrap();
    assert_eq!(sealing_in(&claimed_word), Sealing::Copying(Turn::Off));
    let closed_word = closed(&claimed_word, "dev_a:n1").unwrap();
    assert!(closed_word.contains("needs = \"1.36.0\""));
    let plain = switched(&closed_word, "dev_a:n1", None).unwrap();

    assert_eq!(sealing_in(&plain), Sealing::Plain);
}

#[test]
fn a_claim_is_dropped_only_by_its_own_machine_and_leaves_the_rest_of_the_word() {
    let word = "shape = 3\nneeds = \"1.34.0\"\n";
    let claimed_word = claimed(word, &phase(Turn::On, "dev_a", "n1")).unwrap();

    assert!(dropped(&claimed_word, "dev_b:n1").is_none());
    assert!(dropped(&claimed_word, "dev_a:n2").is_none());
    let back = dropped(&claimed_word, "dev_a:n1").unwrap();

    assert_eq!(sealing_in(&back), Sealing::Plain);
    assert!(
        back.contains("needs = \"1.36.0\""),
        "needs never comes back down"
    );
    assert!(claimed("not = = toml", &phase(Turn::On, "dev_a", "n1")).is_none());
}

#[test]
fn a_turn_stirs_only_its_own_claim_and_the_stir_goes_with_it() {
    let claimed_word = claimed("shape = 3\n", &phase(Turn::On, "dev_a", "n1")).unwrap();

    assert!(stirred(&claimed_word, "dev_b:n1", 40).is_none());
    let stirred_word = stirred(&claimed_word, "dev_a:n1", 40).unwrap();
    assert_eq!(stirred_in(&stirred_word), Some(40));
    assert_eq!(sealing_in(&stirred_word), Sealing::Copying(Turn::On));

    let back = dropped(&stirred_word, "dev_a:n1").unwrap();
    assert_eq!(stirred_in(&back), None);
    let closed_word = closed(&stirred_word, "dev_a:n1").unwrap();
    let sealed_word = switched(&closed_word, "dev_a:n1", Some("store_one")).unwrap();
    assert_eq!(stirred_in(&sealed_word), None);
}
