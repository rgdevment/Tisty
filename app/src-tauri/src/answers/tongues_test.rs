use super::{tongued, translated};
use crate::Session;
use tisty_core::model::DocId;
use tisty_core::{Op, Paths};

struct Desk {
    _tmp: tempfile::TempDir,
    session: Session,
}

fn desk() -> Desk {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    Desk { _tmp: tmp, session }
}

fn wrote(session: &mut Session, body: &str, page_of: Option<DocId>) -> DocId {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    let id = ulid::Ulid::generate();
    session
        .commit(Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                file: made.id,
                order: "a0".into(),
                said: Some(tisty_core::event::Said::of(body)),
                page_of,
                ..Default::default()
            },
        })
        .unwrap();
    id
}

fn body_of(session: &Session, id: DocId) -> String {
    tisty_core::docs::read(&session.paths.docs(), &session.state.docs[&id].file).unwrap()
}

fn id_of(session: &Session, file: &str) -> DocId {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id
}

#[test]
fn translating_copies_the_words_and_their_pages_into_the_other_language_and_joins_them() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "# Paseos\n\n![Rina](tisty:doc/x)\n", None);
    wrote(&mut session, "# Rina\n", Some(es));

    let made = translated(&mut session, &es.to_string(), "EN", Some("es".into())).unwrap();
    let en = id_of(&session, &made.id);

    assert_eq!(
        body_of(&session, es),
        "---\nlang: es\n---\n# Paseos\n\n![Rina](tisty:doc/x)\n"
    );
    assert!(body_of(&session, en).starts_with("---\nlang: en\n---\n# Paseos\n"));
    assert_eq!(session.state.tongues_of(es).len(), 2);
    assert_eq!(
        session.state.pages_of(en).len(),
        1,
        "the pages come along to be translated"
    );
    assert!(
        session
            .state
            .twin_page(session.state.pages_of(es)[0].id, en)
            .is_some()
    );
}

#[test]
fn a_document_that_says_no_language_has_to_be_told_one_first() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "# Paseos\n", None);

    let refused = translated(&mut session, &es.to_string(), "en", None).unwrap_err();
    assert_eq!(refused.code, "tongueUnsaid");
    let refused =
        translated(&mut session, &es.to_string(), "Inglés", Some("es".into())).unwrap_err();
    assert_eq!(refused.code, "notALanguage");
}

#[test]
fn a_language_the_group_already_has_is_refused() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    translated(&mut session, &es.to_string(), "en", None).unwrap();

    let refused = translated(&mut session, &es.to_string(), "en", None).unwrap_err();
    assert_eq!(refused.code, "tongueTaken");
    let refused = translated(&mut session, &es.to_string(), "es", None).unwrap_err();
    assert_eq!(refused.code, "tongueTaken");
}

#[test]
fn joining_moves_it_to_the_folder_of_its_languages_and_parting_leaves_it_where_it_is() {
    let Desk { _tmp, mut session } = desk();
    let folder = ulid::Ulid::generate();
    session
        .commit(Op::FolderAdd {
            id: folder,
            d: tisty_core::event::FolderAdd {
                name: "RSRC".into(),
                order: "a0".into(),
                parent: None,
                icon: None,
                color: None,
                computer: None,
            },
        })
        .unwrap();
    let es = wrote(&mut session, "# Red\n", None);
    session
        .commit(Op::DocMove {
            id: es,
            d: tisty_core::event::Filed {
                folder: Some(Some(folder)),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let en = wrote(&mut session, "# Network\n", None);

    tongued(
        &mut session,
        &en.to_string(),
        Some(es.to_string()),
        Some("en".into()),
        Some("es".into()),
    )
    .unwrap();

    assert_eq!(session.state.docs[&en].folder, Some(folder));
    assert_eq!(session.state.kin(es), vec![en]);

    tongued(&mut session, &en.to_string(), None, None, None).unwrap();
    assert!(session.state.tongues_of(es).is_empty());
    assert_eq!(session.state.docs[&en].folder, Some(folder));
}

#[test]
fn parting_the_anchor_leaves_the_others_together() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let pt = translated(&mut session, &es.to_string(), "pt", None).unwrap();
    let (en, pt) = (id_of(&session, &en.id), id_of(&session, &pt.id));

    tongued(&mut session, &es.to_string(), None, None, None).unwrap();

    assert!(session.state.tongues_of(es).is_empty());
    let left: Vec<DocId> = session
        .state
        .tongues_of(en)
        .iter()
        .map(|one| one.id)
        .collect();
    assert_eq!(left, vec![en, pt]);
}

#[test]
fn a_page_has_no_languages_of_its_own() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let page = wrote(&mut session, "# Rina\n", Some(es));

    let refused = translated(&mut session, &page.to_string(), "en", Some("es".into())).unwrap_err();
    assert_eq!(refused.code, "pageHasNoTongue");
}

#[test]
fn putting_one_language_away_puts_away_the_ones_beside_it() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);

    session.drop_doc(&en.to_string()).unwrap();

    assert!(!session.state.docs.contains_key(&es));
    assert!(!session.state.docs.contains_key(&en));
}

#[test]
fn a_search_finds_each_group_once_in_the_language_the_words_turned_up_in() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let alone = wrote(&mut session, "# Otro\n", None);
    let file = |id: DocId| session.state.docs[&id].file.clone();
    let found = vec![en.id.clone(), file(es), file(alone)];

    let once = super::once_each(&session.state, found, |one| one.as_str());

    assert_eq!(once, vec![en.id, file(alone)]);
}

#[test]
fn the_paper_a_document_is_laid_on_is_the_same_in_its_other_languages() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);

    super::dressed(
        &mut session,
        &es.to_string(),
        Some("folio".into()),
        true,
        false,
        Some("coral".into()),
    )
    .unwrap();
    assert_eq!(
        body_of(&session, en),
        "---\nlang: en\npaper: folio\nsheets: on\ndesign: coral\n---\n# Paseos\n"
    );

    super::dressed(
        &mut session,
        &es.to_string(),
        Some("a4".into()),
        false,
        false,
        Some("tisty".into()),
    )
    .unwrap();
    assert_eq!(body_of(&session, en), "---\nlang: en\n---\n# Paseos\n");
}

#[test]
fn a_change_of_paper_leaves_the_design_of_another_language_as_it_is() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);
    let file = session.state.docs[&en].file.clone();
    tisty_core::docs::amend(&session.paths.docs(), &session.writer(), &file, |was| {
        Some(tisty_markdown::front::with_field(was, "design", "coral"))
    })
    .unwrap();
    let paper = |session: &mut Session, design: Option<String>| {
        super::dressed(
            session,
            &es.to_string(),
            Some("letter".into()),
            false,
            false,
            design,
        )
        .unwrap();
    };
    paper(&mut session, None);
    let body = body_of(&session, en);
    assert!(
        body.contains("paper: letter") && body.contains("design: coral"),
        "{body}"
    );
    paper(&mut session, Some("bosque".into()));
    assert!(body_of(&session, en).contains("design: bosque"));
}

#[test]
fn a_design_of_the_persons_own_is_not_replaced_in_another_language() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(
        &mut session,
        "---\nlang: es\ndesign: Ana Perez\n---\n# Paseos\n",
        None,
    );
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);
    let was = body_of(&session, en);
    super::dressed(
        &mut session,
        &es.to_string(),
        Some("a4".into()),
        false,
        false,
        Some("coral".into()),
    )
    .unwrap();
    assert_eq!(body_of(&session, en), was);
}

#[test]
fn a_language_that_joins_wears_the_design_of_its_anchor() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(
        &mut session,
        "---\nlang: es\ndesign: bosque\n---\n# Paseos\n",
        None,
    );
    let en = wrote(
        &mut session,
        "---\nlang: en\ndesign: coral\n---\n# Walks\n",
        None,
    );
    tongued(
        &mut session,
        &en.to_string(),
        Some(es.to_string()),
        None,
        None,
    )
    .unwrap();
    assert!(body_of(&session, es).contains("design: bosque"));
    assert!(body_of(&session, en).contains("design: bosque"));
}

#[test]
fn a_design_nobody_knows_stays_written_in_another_language() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(
        &mut session,
        "---\nlang: es\ndesign: pergamino\n---\n# Paseos\n",
        None,
    );
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);
    let was = body_of(&session, en);
    assert!(was.contains("design: pergamino"), "{was}");

    super::dressed(
        &mut session,
        &es.to_string(),
        Some("a4".into()),
        false,
        false,
        Some("tisty".into()),
    )
    .unwrap();
    assert_eq!(body_of(&session, en), was);
}

#[test]
fn a_group_with_one_language_locked_is_not_deleted_in_part() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "---\nlang: es\n---\n# Paseos\n", None);
    let en = translated(&mut session, &es.to_string(), "en", None).unwrap();
    let en = id_of(&session, &en.id);
    let pt = translated(&mut session, &es.to_string(), "pt", None).unwrap();
    let pt = id_of(&session, &pt.id);
    session.commit(Op::DocLock { id: pt }).unwrap();

    assert_eq!(
        session.drop_doc(&es.to_string()).unwrap_err().code,
        "documentLocked"
    );
    for one in [es, en, pt] {
        assert!(
            session.state.docs.contains_key(&one),
            "nothing goes unless all can"
        );
    }
}

#[test]
fn a_refused_join_leaves_both_documents_as_they_were() {
    let Desk { _tmp, mut session } = desk();
    let es = wrote(&mut session, "# Red\n", None);
    let other = wrote(&mut session, "# Network\n", None);

    let refused = tongued(
        &mut session,
        &other.to_string(),
        Some(es.to_string()),
        Some("es".into()),
        Some("es".into()),
    )
    .unwrap_err();

    assert_eq!(refused.code, "tongueTaken");
    assert_eq!(body_of(&session, es), "# Red\n");
    assert_eq!(body_of(&session, other), "# Network\n");
}
