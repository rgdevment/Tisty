use super::*;

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn filed(session: &mut Session, body: &str) -> String {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: made.id.clone(),
                order: made.id.clone(),
                folder: None,
                page_of: None,
                task_of: None,
            },
        })
        .unwrap();
    made.id
}

#[test]
fn keeping_both_again_finds_the_copy_it_already_made_instead_of_another() {
    let (_tmp, mut session) = session();
    filed(&mut session, "# Idea\n\nlo mio\n");
    let copy = filed(&mut session, "# Idea (otra versión)\n\nlo suyo\n");
    let beside = (None, None, "a0".to_string());

    assert_eq!(
        twin_of(
            &session,
            Some(&beside),
            "# Idea (otra versión)\n\nlo suyo\n"
        ),
        Some(copy)
    );
    assert_eq!(
        twin_of(
            &session,
            Some(&beside),
            "# Idea (otra versión)\n\nalgo nuevo\n"
        ),
        None,
        "a different arrival was taken for one already kept"
    );
}

#[test]
fn a_copy_written_with_its_closing_line_is_still_the_same_arrival() {
    let (_tmp, mut session) = session();
    let copy = filed(&mut session, "# Idea (otra versión)\n\nlo suyo");
    let beside = (None, None, "a0".to_string());

    assert_eq!(
        twin_of(&session, Some(&beside), "# Idea (otra versión)\n\nlo suyo"),
        Some(copy)
    );
}

#[test]
fn a_settled_body_is_written_down_so_the_other_machine_has_a_history_that_answers_for_it() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, "# Idea\n\nlo viejo\n");
    session.retell(&id, "# Idea\n\nlo nuevo\n", None);
    let at = tisty_core::docs::resolve(&session.paths.docs(), &id).unwrap();
    std::fs::write(&at, "# Idea\n\nlo viejo\n").unwrap();

    told_as_settled(&mut session, &id);

    let kept = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .unwrap();
    assert_eq!(
        kept.print,
        tisty_core::event::Said::of("# Idea\n\nlo viejo\n").print
    );
}

#[test]
fn a_file_from_outside_tells_what_the_disk_knows_and_no_author() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("notas.md");
    std::fs::write(&at, "# Notas\n").unwrap();

    let told = super::outside_facts(&at).unwrap();

    assert_eq!(told.bytes, 8);
    assert!(told.wrote.is_some_and(|when| when > 0));
    assert_eq!(told.pages, 0);
    assert_eq!(told.author, None);
}

#[test]
fn a_file_from_outside_that_went_away_is_said_missing_by_its_path() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("gone.md");

    let refused = super::outside_facts(&at).map(|_| ()).unwrap_err();

    assert_eq!(refused.code, "noSuchDoc");
    assert_eq!(
        refused.name.as_deref(),
        Some(at.display().to_string().as_str())
    );
}

fn arrived(session: &mut Session, by: &str, guest: bool) -> tisty_core::model::Kept {
    let made = tisty_core::docs::create(
        &session.paths.docs(),
        &session.config.device_id,
        "# De Ana\n",
    )
    .unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest,
                made: None,
                by: Some(by.into()),
                said: None,
                file: made.id.clone(),
                order: made.id.clone(),
                folder: None,
                page_of: None,
                task_of: None,
            },
        })
        .unwrap();
    session
        .state
        .docs
        .values()
        .find(|one| one.file == made.id)
        .unwrap()
        .clone()
}

#[test]
fn on_paper_my_documents_carry_my_name_and_a_guests_keeps_theirs() {
    let (_tmp, mut session) = session();
    let mine = filed(&mut session, "# Mío\n");
    for alias in ["ana", "mario"] {
        session
            .commit(Op::Signed {
                d: tisty_core::event::Signature {
                    alias: Some(alias.into()),
                    name: Some("Mario Hidalgo".into()),
                    ..Default::default()
                },
            })
            .unwrap();
    }
    let unsigned = session
        .state
        .docs
        .values()
        .find(|one| one.file == mine)
        .unwrap()
        .clone();
    let theirs = arrived(&mut session, "ana", true);
    let old_hand = arrived(&mut session, "ana", false);

    assert_eq!(
        signed_on_paper(&session.state, &unsigned).as_deref(),
        Some("Mario Hidalgo")
    );
    assert_eq!(
        signed_on_paper(&session.state, &theirs).as_deref(),
        Some("ana")
    );
    assert_eq!(
        signed_on_paper(&session.state, &old_hand).as_deref(),
        Some("Mario Hidalgo")
    );
    assert_eq!(edited_on_paper(&session.state, &theirs, "mario"), "mario");
    assert_eq!(
        edited_on_paper(&session.state, &old_hand, "mario"),
        "Mario Hidalgo"
    );
}

#[test]
fn on_paper_an_unsigned_guest_document_names_nobody() {
    let (_tmp, mut session) = session();
    session
        .commit(Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some("mario".into()),
                name: Some("Mario Hidalgo".into()),
                ..Default::default()
            },
        })
        .unwrap();
    let mut theirs = arrived(&mut session, "ana", true);
    theirs.by = None;

    assert_eq!(signed_on_paper(&session.state, &theirs), None);
}

#[test]
fn a_name_to_sign_is_held_to_what_a_share_takes() {
    assert_eq!(name_to_sign(None).unwrap(), None);
    assert_eq!(name_to_sign(Some("   ".into())).unwrap(), None);
    assert_eq!(
        name_to_sign(Some("  Ana Pérez 2 ".into()))
            .unwrap()
            .as_deref(),
        Some("Ana Pérez 2")
    );
    for unfit in ["12345", "A1234", "ab12 3"] {
        assert_eq!(
            name_to_sign(Some(unfit.into())).unwrap_err().code,
            "nameUnfit",
            "{unfit}"
        );
    }
    let long = "a".repeat(tisty_core::event::NAME_AT_MOST + 1);
    assert_eq!(name_to_sign(Some(long)).unwrap_err().code, "nameTooLong");
}
