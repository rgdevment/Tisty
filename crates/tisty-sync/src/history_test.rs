fn arrived(
    one: &Machine,
    shared: &Path,
    id: &str,
    body: &str,
    wrote: tisty_core::history::Hand,
) -> Moved {
    theirs(shared, id, body);
    let at = shared.join("docs").join(format!("{id}.md"));
    let printed = std::collections::BTreeMap::from([(
        id.to_string(),
        super::papers::Answers {
            newest: tisty_core::docs::print_of(&at).unwrap(),
            wrote: Some(wrote),
            ..Default::default()
        },
    )]);
    super::papers::carry_papers_leaning_on(
        &one.data,
        shared,
        &[id.to_string()],
        &[],
        None,
        Some(&printed),
        &|_, _| false,
        &|_| false,
        false,
        false,
        true,
        &mut |_| {},
    )
    .unwrap()
}

#[test]
fn what_arrives_keeps_what_stood_here_by_the_cut_and_not_on_every_arrival() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let id = "dev_a-0001";
    paper(&one, id, "# antes\n");
    theirs(shared.path(), id, "# antes\n");
    carry_papers(&one.data, shared.path(), &[id.to_string()]).unwrap();
    let at = |minutes: i64| jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap();
    let claude = Some("Claude Code".to_string());

    for (body, wrote) in [
        ("# uno\n", (at(0), None)),
        ("# uno y dos\n", (at(5), None)),
        ("# uno, dos y tres\n", (at(6), claude.clone())),
        ("# uno, dos, tres y cuatro\n", (at(50), claude.clone())),
        ("# y cinco\n", (at(51), claude)),
    ] {
        let done = arrived(&one, shared.path(), id, body, wrote);
        assert_eq!(done.brought, 1, "{body}");
    }

    let kept: std::collections::BTreeSet<String> = tisty_core::history::versions(&one.data, id)
        .into_iter()
        .map(|one| one.print)
        .collect();
    let told = tisty_core::docs::print_of_text;
    assert_eq!(
        kept,
        std::collections::BTreeSet::from([
            told("# antes\n"),
            told("# uno y dos\n"),
            told("# uno, dos y tres\n"),
        ]),
        "one version when nobody had told the history, one when another hand wrote, one after the quiet"
    );
    assert!(
        tisty_core::history::back(&one.data, id).is_none(),
        "a body the log answers for takes no step back"
    );
    assert!(
        !shared.path().join(tisty_core::history::FOLDER).exists(),
        "the history travelled to the meeting place"
    );
}

#[test]
fn keeping_theirs_leaves_what_was_here_in_its_history() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    at_odds(&one, shared.path());

    settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs, None).unwrap();

    let kept = tisty_core::history::versions(&one.data, "dev_a-0001");
    assert_eq!(kept.len(), 1);
    assert_eq!(
        tisty_core::history::read(&one.data, "dev_a-0001", &kept[0].print).as_deref(),
        Some("# Minuta\n\nlo mio")
    );
    assert!(!shared.path().join(tisty_core::history::FOLDER).exists());
}

#[test]
fn a_clean_join_keeps_what_this_side_said_before_it() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    let mine = "# Kit\n\nla introduccion del mac\n\nel cuerpo\n\nel cierre\n";
    filed(&one, "uno-0001", "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();
    let person = tisty_core::history::Writer::now(&one.data, None);
    tisty_core::docs::write_by(&here, &person, "uno-0001", mine).unwrap();
    wrote_body(
        &there,
        "uno-0001",
        "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n\nlo de windows\n",
    );

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(done.joined, vec!["uno-0001".to_string()]);
    let kept: Vec<String> = tisty_core::history::versions(&one.data, "uno-0001")
        .into_iter()
        .map(|one| one.print)
        .collect();
    assert!(
        kept.contains(&tisty_core::docs::print_of_text(mine)),
        "the same person wrote here a moment before, and the join still has to keep it"
    );
}

#[test]
fn a_body_landing_where_none_stood_tells_its_writer_to_the_next_arrival() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let id = "dev_b-0001";
    let at = |minutes: i64| jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap();
    let claude = Some("Claude Code".to_string());
    let kept = || -> Vec<String> {
        tisty_core::history::versions(&one.data, id)
            .into_iter()
            .map(|one| one.print)
            .collect()
    };

    let landed = arrived(&one, shared.path(), id, "# uno\n", (at(0), claude.clone()));
    assert_eq!(landed.brought, 1);
    arrived(&one, shared.path(), id, "# uno y dos\n", (at(5), claude));
    assert!(
        kept().is_empty(),
        "the same hand wrote again five minutes after the body landed, and a version was cut"
    );

    arrived(&one, shared.path(), id, "# y tres\n", (at(6), None));
    assert_eq!(
        kept(),
        vec![tisty_core::docs::print_of_text("# uno y dos\n")],
        "another hand wrote, and what stood here was not kept"
    );
}
