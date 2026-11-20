use super::*;

const DOC: &str = "mac0-0001";
const UNO: &str = "# Acta\n\nuno\n";
const DOS: &str = "# Acta\n\ndos\n";
const TRES: &str = "# Acta\n\ntres\n";
const CUATRO: &str = "# Acta\n\ncuatro\n";

fn at(minutes: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap()
}

fn desk() -> (tempfile::TempDir, PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    (room, docs)
}

fn person(data: &Path, minutes: i64) -> Writer<'_> {
    Writer::at(data, None, at(minutes))
}

fn assistant(data: &Path, minutes: i64) -> Writer<'_> {
    Writer::at(data, Some("Claude Code".into()), at(minutes))
}

fn print(body: &str) -> String {
    crate::docs::print_of_text(body)
}

fn cut_kept(data: &Path) -> Vec<String> {
    let mut all = versions(data, DOC);
    all.retain(|one| !one.spare);
    all.reverse();
    all.into_iter().map(|one| one.print).collect()
}

fn on_disk(data: &Path) -> Vec<String> {
    let mut all: Vec<String> = versions(data, DOC)
        .into_iter()
        .map(|one| one.print)
        .collect();
    all.sort();
    all
}

fn sorted(mut prints: Vec<String>) -> Vec<String> {
    prints.sort();
    prints
}

#[test]
fn a_body_left_alone_for_half_an_hour_becomes_a_version_when_it_is_written_over() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    crate::docs::write_by(&docs, &person(data, 29), DOC, DOS).unwrap();
    assert!(
        versions(data, DOC).is_empty(),
        "a change inside the half hour was kept"
    );

    crate::docs::write_by(&docs, &person(data, 59), DOC, TRES).unwrap();

    assert_eq!(cut_kept(data), vec![print(DOS)]);
    assert_eq!(read(data, DOC, &print(DOS)).as_deref(), Some(DOS));
    assert_eq!(
        versions(data, DOC)[0].replaced,
        at(59),
        "a version says when it was replaced"
    );
}

#[test]
fn a_body_becomes_a_version_as_soon_as_somebody_else_writes_over_it() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    crate::docs::write_by(&docs, &assistant(data, 1), DOC, DOS).unwrap();
    crate::docs::write_by(&docs, &person(data, 2), DOC, TRES).unwrap();
    crate::docs::write_by(&docs, &person(data, 3), DOC, CUATRO).unwrap();

    assert_eq!(cut_kept(data), vec![print(UNO), print(DOS)]);
}

#[test]
fn two_assistants_are_two_writers() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();

    let codex = Writer::at(data, Some("Codex".into()), at(1));
    crate::docs::write_by(&docs, &codex, DOC, DOS).unwrap();

    assert_eq!(cut_kept(data), vec![print(UNO)]);
}

#[test]
fn the_window_s_save_and_adding_to_the_end_keep_a_version_by_the_same_cut() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    assert!(crate::docs::write_over(&docs, &person(data, 40), DOC, DOS, &print(UNO)).unwrap());
    let whole = crate::docs::append(&docs, &assistant(data, 41), DOC, "y algo mas").unwrap();
    crate::docs::append(&docs, &assistant(data, 42), DOC, "y otra").unwrap();

    assert_eq!(cut_kept(data), vec![print(UNO), print(DOS)]);
    assert!(
        back(data, DOC).is_none(),
        "neither of them keeps a step back"
    );
    assert_ne!(print(&whole), print(DOS));
}

#[test]
fn a_body_the_history_was_never_told_about_is_kept_when_it_is_written_over() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::write(&docs, DOC, UNO).unwrap();

    crate::docs::write_by(&docs, &person(data, 0), DOC, DOS).unwrap();
    assert_eq!(cut_kept(data), vec![print(UNO)], "no marker at all");

    crate::docs::write(&docs, DOC, TRES).unwrap();
    crate::docs::write_by(&docs, &person(data, 1), DOC, CUATRO).unwrap();

    assert_eq!(
        sorted(cut_kept(data)),
        sorted(vec![print(UNO), print(TRES)]),
        "a body written behind the history's back has no writer to weigh"
    );
}

#[test]
fn what_arrives_with_nobody_answering_for_it_always_keeps_what_it_replaces() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    crate::docs::write_by(&docs, &Writer::unanswered(data), DOC, DOS).unwrap();
    crate::docs::write_by(&docs, &Writer::unanswered(data), DOC, TRES).unwrap();

    assert_eq!(on_disk(data), sorted(vec![print(UNO), print(DOS)]));

    crate::docs::write_by(&docs, &Writer::now(data, None), DOC, CUATRO).unwrap();

    assert!(
        on_disk(data).contains(&print(TRES)),
        "a body nobody answered for was written over a moment later with nothing kept"
    );
}

#[test]
fn a_write_that_changes_nothing_keeps_nothing() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    crate::docs::write_by(&docs, &assistant(data, 90), DOC, UNO).unwrap();

    assert!(versions(data, DOC).is_empty());
}

#[test]
fn a_write_that_keeps_its_step_back_keeps_it_even_when_the_cut_does_not() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();

    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();

    assert_eq!(
        back(data, DOC),
        Some(Before {
            body: UNO.to_string(),
            left: Some(print(DOS)),
        })
    );
    assert!(
        cut_kept(data).is_empty(),
        "the step back alone is not a version"
    );

    crate::docs::rewrite(&docs, &assistant(data, 2), DOC, TRES, &print(DOS)).unwrap();

    assert_eq!(back(data, DOC).map(|one| one.body).as_deref(), Some(DOS));
    assert_eq!(
        on_disk(data),
        vec![print(DOS)],
        "the step back before it is dropped"
    );
}

#[test]
fn a_version_the_cut_kept_is_not_dropped_when_it_comes_back_as_the_step_back() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 40), DOC, DOS, &print(UNO)).unwrap();
    assert_eq!(cut_kept(data), vec![print(UNO)]);

    crate::docs::rewrite(&docs, &assistant(data, 41), DOC, UNO, &print(DOS)).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 42), DOC, DOS, &print(UNO)).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 43), DOC, TRES, &print(DOS)).unwrap();

    assert_eq!(
        cut_kept(data),
        vec![print(UNO)],
        "going back and forth took a version the cut had kept"
    );
    let uno = versions(data, DOC)
        .into_iter()
        .find(|one| one.print == print(UNO))
        .unwrap();
    assert_eq!(
        uno.replaced,
        at(40),
        "a step back that cut nothing moved the day the version was replaced"
    );
    assert_eq!(back(data, DOC).map(|one| one.body).as_deref(), Some(DOS));
}

#[test]
fn a_step_back_that_the_cut_then_keeps_stops_being_spare() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 2), DOC, UNO, &print(DOS)).unwrap();

    crate::docs::write_by(&docs, &person(data, 3), DOC, TRES).unwrap();
    crate::docs::write_by(&docs, &assistant(data, 4), DOC, DOS).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 5), DOC, CUATRO, &print(DOS)).unwrap();

    assert!(
        cut_kept(data).contains(&print(UNO)),
        "the person wrote over it, so the cut kept it"
    );
    assert!(cut_kept(data).contains(&print(TRES)));
}

#[test]
fn real_writing_leaves_the_step_back_where_it_was() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();

    crate::docs::write_over(&docs, &person(data, 2), DOC, TRES, &print(DOS)).unwrap();

    assert_eq!(
        back(data, DOC),
        Some(Before {
            body: UNO.to_string(),
            left: Some(print(DOS)),
        }),
        "so going back sees there was writing since"
    );
}

#[test]
fn a_step_back_put_back_after_a_refused_write_reads_as_it_did_before() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();
    let point = restore_point(data, DOC).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 2), DOC, TRES, &print(DOS)).unwrap();

    crate::docs::rewrite_if(&docs, &assistant(data, 3), DOC, TRES, DOS).unwrap();
    set_restore_point(data, DOC, &point).unwrap();

    assert_eq!(
        back(data, DOC),
        Some(Before {
            body: UNO.to_string(),
            left: Some(print(DOS)),
        })
    );
    assert_eq!(on_disk(data), vec![print(UNO)]);
}

#[test]
fn the_step_back_can_be_pointed_at_a_body_no_write_just_replaced() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 2), DOC, TRES, &print(DOS)).unwrap();

    keep_back(data, DOC, UNO, TRES).unwrap();

    assert_eq!(
        back(data, DOC),
        Some(Before {
            body: UNO.to_string(),
            left: Some(print(TRES)),
        })
    );
    assert_eq!(on_disk(data), vec![print(UNO)]);
}

#[test]
fn versions_past_the_limit_go_but_the_last_one_and_the_step_back_stay() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::write_by(&docs, &person(data, 60), DOC, DOS).unwrap();
    crate::docs::write_by(&docs, &person(data, 120), DOC, TRES).unwrap();
    crate::docs::rewrite(&docs, &person(data, 125), DOC, CUATRO, &print(TRES)).unwrap();
    let a_day = 24 * 60;

    assert_eq!(
        pruned(data, 30, at(60 + 30 * a_day)),
        0,
        "the oldest was replaced exactly thirty days ago"
    );
    assert_eq!(pruned(data, 30, at(400 * a_day)), 1);

    assert_eq!(
        on_disk(data),
        sorted(vec![print(DOS), print(TRES)]),
        "the last version the cut kept and the step back outlive any limit"
    );
    assert_eq!(pruned(data, 1, at(800 * a_day)), 0);
}

#[test]
fn each_limit_offered_counts_from_the_day_a_version_was_replaced() {
    for days in DAYS {
        let (room, docs) = desk();
        let data = room.path();
        crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
        crate::docs::write_by(&docs, &person(data, 60), DOC, DOS).unwrap();
        crate::docs::write_by(&docs, &person(data, 120), DOC, TRES).unwrap();
        let limit = i64::from(days) * 24 * 60;

        assert_eq!(
            pruned(data, days, at(60 + limit)),
            0,
            "{days} days, on the day"
        );
        assert_eq!(
            pruned(data, days, at(61 + limit)),
            1,
            "{days} days, a minute past"
        );
        assert_eq!(cut_kept(data), vec![print(DOS)]);
    }
}

#[test]
fn only_the_limits_offered_are_taken_and_anything_else_is_thirty_days() {
    assert_eq!(days_kept(None), 30);
    for days in [1, 5, 15, 30] {
        assert_eq!(days_kept(Some(days)), days);
    }
    for days in [0, 2, 31, 365] {
        assert_eq!(days_kept(Some(days)), 30, "{days}");
    }
}

#[test]
fn what_sat_in_originals_becomes_each_document_s_first_version_and_its_step_back() {
    let (room, docs) = desk();
    let data = room.path();
    let (originals, stood) = (data.join("originals"), data.join("originals-at"));
    std::fs::create_dir_all(&originals).unwrap();
    std::fs::create_dir_all(&stood).unwrap();
    crate::docs::write(&docs, DOC, DOS).unwrap();
    std::fs::write(originals.join("mac0-0001.md"), UNO).unwrap();
    std::fs::write(stood.join("mac0-0001.md"), print(DOS)).unwrap();
    crate::docs::write(&docs, "mac0-0003", TRES).unwrap();
    std::fs::write(originals.join("mac0-0003.md"), "# Suelta").unwrap();
    std::fs::write(originals.join("mac0-0002.md"), "# De un documento borrado").unwrap();

    assert_eq!(
        back(data, DOC),
        Some(Before {
            body: UNO.to_string(),
            left: Some(print(DOS)),
        })
    );

    assert_eq!(cut_kept(data), vec![print(UNO)]);
    assert_eq!(
        back(data, "mac0-0003"),
        Some(Before {
            body: "# Suelta".to_string(),
            left: None,
        }),
        "a copy with no print beside it still comes along"
    );
    assert!(
        versions(data, "mac0-0002").is_empty() && !data.join("history/mac0-0002").exists(),
        "the copy of a document that is gone was brought along"
    );
    assert!(!originals.exists() && !stood.exists());
}

#[test]
fn a_history_already_there_is_not_written_over_by_an_older_copy() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();
    std::fs::create_dir_all(data.join("originals")).unwrap();
    std::fs::write(data.join("originals/mac0-0001.md"), "# Mas vieja").unwrap();

    adopt_originals(data);

    assert_eq!(back(data, DOC).map(|one| one.body).as_deref(), Some(UNO));
    assert!(!data.join("originals").exists());
}

#[test]
fn a_document_that_is_gone_takes_its_history_with_it() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();

    let mark = crate::docs::resolve(&data.join(super::GONE_BACK), DOC).unwrap();
    std::fs::create_dir_all(mark.parent().unwrap()).unwrap();
    std::fs::write(&mark, "a\nb\n").unwrap();

    forget(data, DOC);

    assert!(versions(data, DOC).is_empty() && back(data, DOC).is_none());
    assert!(!data.join("history").join(DOC).exists());
    assert!(!mark.exists(), "the mark that it went back outlived it");
}

#[test]
fn a_history_emptied_by_hand_holds_nothing_and_still_knows_who_wrote_last() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();

    assert_eq!(empty(data, DOC).unwrap(), 1);

    assert!(versions(data, DOC).is_empty() && back(data, DOC).is_none());
    crate::docs::write_by(&docs, &assistant(data, 2), DOC, TRES).unwrap();
    assert!(
        versions(data, DOC).is_empty(),
        "the same writer a minute later was taken for a stranger"
    );
}

#[test]
fn the_rule_replayed_over_what_the_log_says_names_the_versions_on_disk() {
    let (room, docs) = desk();
    let data = room.path();
    let claude = Some("Claude Code");
    let said: [(i64, Option<&str>, &str); 7] = [
        (0, None, "# A\n"),
        (5, None, "# B\n"),
        (10, claude, "# C\n"),
        (12, claude, "# D\n"),
        (50, claude, "# E\n"),
        (55, None, "# F\n"),
        (56, None, "# G\n"),
    ];

    for (n, (minutes, via, body)) in said.iter().enumerate() {
        let by = Writer::at(data, via.map(str::to_string), at(*minutes));
        match n {
            0 => crate::docs::create_named_by(&docs, &by, DOC, body).map(|_| ()),
            _ => crate::docs::write_by(&docs, &by, DOC, body),
        }
        .unwrap();
    }

    let told: Vec<String> = said
        .windows(2)
        .filter(|pair| cuts(Some((at(pair[0].0), pair[0].1)), at(pair[1].0), pair[1].1))
        .map(|pair| print(pair[0].2))
        .collect();
    assert_eq!(told, vec![print("# B\n"), print("# D\n"), print("# E\n")]);
    assert_eq!(cut_kept(data), told);
}

#[test]
fn a_body_that_is_not_text_is_kept_byte_for_byte() {
    let (room, _docs) = desk();
    let data = room.path();
    let bytes = [0x66, 0x6f, 0xff, 0xfe, 0x62, 0x61, 0x72];

    person(data, 0)
        .replaces(DOC, &bytes, UNO, Step::Stays)
        .unwrap();

    let kept = versions(data, DOC);
    assert_eq!(kept.len(), 1);
    let at = data
        .join("history")
        .join(DOC)
        .join(format!("{}.md", kept[0].print));
    assert_eq!(std::fs::read(at).unwrap(), bytes);
    assert_eq!(read(data, DOC, &kept[0].print), None);
}

#[test]
fn a_history_can_never_be_named_outside_the_store() {
    let (room, _docs) = desk();
    let data = room.path();

    for id in [
        "../escaped",
        "..\\escaped",
        "/etc/passwd",
        "CON",
        "mac0-0001 ",
    ] {
        assert!(
            person(data, 0).replaces(id, b"x", "y", Step::Back).is_err(),
            "{id:?}"
        );
        assert!(versions(data, id).is_empty() && back(data, id).is_none());
        assert!(empty(data, id).is_err());
    }
    assert_eq!(read(data, DOC, "../../docs/mac0-0001"), None);
    assert!(!data.join("history").exists());
}

#[test]
fn the_walk_that_prunes_is_taken_once_a_day() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::write_by(&docs, &person(data, 60), DOC, DOS).unwrap();
    crate::docs::write_by(&docs, &person(data, 120), DOC, TRES).unwrap();
    let a_day = 24 * 60;

    assert_eq!(
        pruned_when_due(data, 1, at(100)),
        0,
        "nothing is past a day yet"
    );
    assert_eq!(
        pruned_when_due(data, 1, at(100 + a_day - 1)),
        0,
        "walked again before a day had passed"
    );
    assert_eq!(pruned_when_due(data, 1, at(100 + a_day)), 1);
}

#[test]
fn a_copy_that_will_not_move_does_not_keep_the_others_from_moving() {
    let (room, docs) = desk();
    let data = room.path();
    let originals = data.join("originals");
    std::fs::create_dir_all(&originals).unwrap();
    crate::docs::write(&docs, DOC, DOS).unwrap();
    std::fs::write(originals.join("mac0-0001.md"), UNO).unwrap();
    crate::docs::write(&docs, "mac0-0002", TRES).unwrap();
    std::fs::write(originals.join("mac0-0002.md"), "# Otra").unwrap();
    std::fs::create_dir_all(data.join("history")).unwrap();
    std::fs::write(data.join("history/mac0-0002"), "not a folder").unwrap();

    adopt_originals(data);

    assert_eq!(back(data, DOC).map(|one| one.body).as_deref(), Some(UNO));
    assert!(
        !originals.join("mac0-0001.md").exists(),
        "the copy that moved is walked again at every write"
    );
    assert!(
        originals.join("mac0-0002.md").exists(),
        "the copy that could not move was thrown away"
    );
}

#[test]
fn a_save_that_cuts_nothing_writes_the_marker_of_the_step_back_no_more() {
    let (room, docs) = desk();
    let data = room.path();
    let shelf = data.join("history").join(DOC);
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    crate::docs::write_by(&docs, &person(data, 1), DOC, DOS).unwrap();
    assert!(
        !shelf.join("state.json").exists(),
        "a document with no version and no step back was given a marker"
    );

    crate::docs::rewrite(&docs, &assistant(data, 2), DOC, TRES, &print(DOS)).unwrap();
    let marked = std::fs::metadata(shelf.join("state.json"))
        .unwrap()
        .modified()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    crate::docs::write_by(&docs, &assistant(data, 3), DOC, CUATRO).unwrap();
    crate::docs::write_by(&docs, &assistant(data, 4), DOC, UNO).unwrap();

    assert_eq!(
        std::fs::metadata(shelf.join("state.json"))
            .unwrap()
            .modified()
            .unwrap(),
        marked,
        "a save that cut nothing and moved no step back wrote the marker again"
    );
    assert_eq!(cut_kept(data), vec![print(DOS)]);
}

#[test]
fn who_wrote_last_torn_by_a_crash_makes_the_next_write_cut_and_leaves_the_step_back() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &assistant(data, 0), DOC, UNO).unwrap();
    crate::docs::rewrite(&docs, &assistant(data, 1), DOC, DOS, &print(UNO)).unwrap();

    std::fs::write(
        data.join("history").join(DOC).join("last.json"),
        b"{\"at\":\"20",
    )
    .unwrap();

    assert_eq!(back(data, DOC).map(|one| one.body).as_deref(), Some(UNO));
    crate::docs::write_by(&docs, &assistant(data, 2), DOC, TRES).unwrap();
    assert!(
        cut_kept(data).contains(&print(DOS)),
        "a writer nobody can name any more was taken for the same one"
    );
}

#[test]
fn a_writer_that_cannot_be_noted_never_refuses_the_save_and_the_next_write_cuts() {
    let (room, docs) = desk();
    let data = room.path();
    let noted = data.join("history").join(DOC).join("last.json");
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();
    std::fs::remove_file(&noted).unwrap();
    std::fs::create_dir(&noted).unwrap();

    crate::docs::write_by(&docs, &person(data, 1), DOC, DOS).unwrap();
    assert_eq!(crate::docs::read(&docs, DOC).unwrap(), DOS);

    crate::docs::write_by(&docs, &person(data, 2), DOC, TRES).unwrap();
    assert!(
        cut_kept(data).contains(&print(DOS)),
        "a hand nobody could note was taken for the one writing now"
    );
}

#[test]
fn a_body_begun_with_nobody_answering_for_it_forgets_who_wrote_there_before() {
    let (room, docs) = desk();
    let data = room.path();
    crate::docs::create_named_by(&docs, &person(data, 0), DOC, UNO).unwrap();

    Writer::unanswered(data).begins(DOC, UNO);
    crate::docs::write_by(&docs, &person(data, 1), DOC, DOS).unwrap();

    assert_eq!(cut_kept(data), vec![print(UNO)]);
}
