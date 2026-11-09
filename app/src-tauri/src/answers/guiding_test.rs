use super::sown;
use crate::Session;
use tisty_core::Paths;

fn shipped(tongue: &str) -> Option<std::path::PathBuf> {
    Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("resources/guide/{tongue}")))
}

#[test]
fn the_guide_comes_in_both_languages_as_one_document_opened_in_the_persons() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();

    let cover = sown(&mut session, "en", "Guide", shipped).unwrap();

    let kept = session
        .state
        .papers()
        .find(|one| one.file == cover.id)
        .unwrap();
    assert_eq!(
        kept.lang.as_deref(),
        Some("en"),
        "the person's own language opens first"
    );
    let langs: Vec<Option<String>> = session
        .state
        .tongues_of(kept.id)
        .iter()
        .map(|one| one.lang.clone())
        .collect();
    assert_eq!(langs, vec![Some("en".to_string()), Some("es".to_string())]);
    for one in session.state.tongues_of(kept.id) {
        assert_eq!(
            session.state.pages_of(one.id).len(),
            3,
            "each language keeps its pages"
        );
        assert!(one.sown);
        assert!(session.state.pages_of(one.id).iter().all(|page| page.sown));
    }
    assert_eq!(session.state.folders.len(), 1, "one folder holds both");

    let walks = std::fs::read_dir(paths.data().join("attachments"))
        .unwrap()
        .filter(|one| {
            one.as_ref()
                .is_ok_and(|one| one.file_name().to_string_lossy().starts_with("walk-"))
        })
        .count();
    assert_eq!(walks, 1, "the same video is kept once for both languages");
    assert!(
        tisty_core::untouched::untouched(&paths.store(), &paths.docs()),
        "a store holding only the guide still reads as untouched"
    );
}

fn old_guide(session: &mut Session, folder: ulid::Ulid, told: &str) -> tisty_core::model::DocId {
    // As an earlier Tisty sowed it: the same words, saying no language.
    let at = tisty_markdown::front::span(told).map_or(0, |one| one.end);
    let body = &told[at..];
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    let id = ulid::Ulid::generate();
    session
        .commit(tisty_core::Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                guest: true,
                by: Some(super::WRITTEN_BY.into()),
                file: made.id,
                order: "a0".into(),
                said: Some(tisty_core::event::Said::of(body)),
                folder: Some(folder),
                ..Default::default()
            },
        })
        .unwrap();
    id
}

fn lived_in(session: &mut Session) -> ulid::Ulid {
    let folder = ulid::Ulid::generate();
    session
        .commit_all(vec![
            tisty_core::Op::FolderAdd {
                id: folder,
                d: tisty_core::event::FolderAdd {
                    name: "Guía".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                    color: None,
                },
            },
            tisty_core::Op::ListAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::ListAdd {
                    name: "Mía".into(),
                    order: "a0".into(),
                    color: None,
                },
            },
        ])
        .unwrap();
    folder
}

#[test]
fn old_guides_nobody_wrote_in_say_their_language_and_come_together() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let folder = lived_in(&mut session);
    let es = old_guide(&mut session, folder, super::super::settings::GUIDE_ES);
    let en = old_guide(&mut session, folder, super::super::settings::GUIDE_EN);

    super::spoken_again(&mut session);

    assert_eq!(session.state.docs[&es].lang.as_deref(), Some("es"));
    assert_eq!(session.state.tongues_of(en).len(), 2);
    super::spoken_again(&mut session);
    assert_eq!(
        session.state.tongues_of(en).len(),
        2,
        "a second opening changes nothing"
    );
}

#[test]
fn an_old_guide_someone_wrote_in_is_left_as_it_is() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let folder = lived_in(&mut session);
    let es = old_guide(&mut session, folder, super::super::settings::GUIDE_ES);
    let file = session.state.docs[&es].file.clone();
    let mine = "# Cómo funciona Tisty\n\nMis notas.\n";
    tisty_core::docs::write(&session.paths.docs(), &file, mine).unwrap();
    session.retell(&file, mine, Some("Mario".into()));

    super::spoken_again(&mut session);

    assert_eq!(session.state.docs[&es].lang, None);
    assert_eq!(
        tisty_core::docs::read(&session.paths.docs(), &file).unwrap(),
        mine
    );
}

#[test]
fn an_old_guide_in_another_language_gets_this_one_beside_it_and_joined() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let folder = lived_in(&mut session);
    let en = old_guide(&mut session, folder, super::super::settings::GUIDE_EN);
    super::spoken_again(&mut session);

    let made = super::sown_beside(&mut session, "es", en, shipped).unwrap();

    let es = session
        .state
        .papers()
        .find(|one| one.file == made.id)
        .unwrap();
    assert_eq!(es.lang.as_deref(), Some("es"));
    assert_eq!(es.folder, Some(folder));
    assert_eq!(session.state.tongues_of(en).len(), 2);
    assert_eq!(session.state.pages_of(es.id).len(), 3);
}
