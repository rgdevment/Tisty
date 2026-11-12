use super::sown;
use crate::Session;
use crate::answers::settings::WRITTEN_BY;
use tisty_core::Paths;
use tisty_core::untouched::{GUIDE_HUE, GUIDE_ICON};

fn shipped(tongue: &str) -> Option<std::path::PathBuf> {
    Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("resources/guide/{tongue}")))
}

#[test]
fn the_guide_comes_in_both_languages_as_one_document_opened_in_the_persons() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();

    let cover = sown(&mut session, "en", "Guide", WRITTEN_BY, shipped).unwrap();

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
            4,
            "each language keeps its pages"
        );
        assert!(one.sown);
        assert!(session.state.pages_of(one.id).iter().all(|page| page.sown));
    }
    assert_eq!(session.state.folders.len(), 1, "one folder holds both");
    let held = session.state.folders.values().next().unwrap();
    assert_eq!(held.icon.as_deref(), Some(GUIDE_ICON));
    assert_eq!(held.color.as_deref(), Some(GUIDE_HUE));
    assert!(tisty_core::model::icon::known(GUIDE_ICON));
    assert!(tisty_core::model::hue::kept(GUIDE_HUE).is_some());

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
fn old_guides_under_the_title_they_had_before_say_their_language_and_come_together() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let folder = lived_in(&mut session);
    let told_es =
        super::super::settings::GUIDE_ES.replacen("# ¿Qué es Tisty?", "# Cómo funciona Tisty", 1);
    let told_en =
        super::super::settings::GUIDE_EN.replacen("# What is Tisty?", "# How Tisty works", 1);
    let es = old_guide(&mut session, folder, &told_es);
    let en = old_guide(&mut session, folder, &told_en);

    super::spoken_again(&mut session);

    assert_eq!(session.state.docs[&es].lang.as_deref(), Some("es"));
    assert_eq!(session.state.tongues_of(en).len(), 2);
}

#[test]
fn every_title_the_guide_ships_with_is_known_as_the_guides() {
    for one in &super::SPOKEN {
        let title = tisty_core::docs::titled(one.told);
        assert_eq!(
            tisty_core::untouched::guide_tongue(&title),
            Some(one.tongue),
            "{title}"
        );
    }
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

    let made = super::sown_beside(&mut session, "es", en, WRITTEN_BY, shipped).unwrap();

    let es = session
        .state
        .papers()
        .find(|one| one.file == made.id)
        .unwrap();
    assert_eq!(es.lang.as_deref(), Some("es"));
    assert_eq!(es.folder, Some(folder));
    assert_eq!(session.state.tongues_of(en).len(), 2);
    assert_eq!(session.state.pages_of(es.id).len(), 4);
}

#[test]
fn the_guide_tisty_ships_is_signed_by_the_makers_root() {
    assert!(
        super::signed_author(&shipped).is_some(),
        "the guide changed since it was signed: the maker signs it again with `cargo run -p tisty-sign-guide`"
    );
}

#[test]
fn asked_again_the_signed_guide_puts_the_unsigned_one_away_and_comes_once_as_its_authors() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    sown(&mut session, "es", "Guía", WRITTEN_BY, shipped).unwrap();
    let old: Vec<tisty_core::model::DocId> = session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && one.by.as_deref() == Some(WRITTEN_BY))
        .map(|one| one.id)
        .collect();
    assert_eq!(old.len(), 2);

    assert_eq!(super::signed_guide_here(&mut session, "es").unwrap(), None);
    let first = super::signed_guide(&mut session, "es", "Guía", "rgdevment", shipped).unwrap();
    let again = super::signed_guide_here(&mut session, "es")
        .unwrap()
        .expect("the signed guide is found again");

    assert_eq!(first.id, again.id, "the signed guide is sown once");
    assert!(old.iter().all(|id| session.state.docs[id].archived));
    let signed: Vec<&tisty_core::model::Kept> = session
        .state
        .docs
        .values()
        .filter(|one| one.by.as_deref() == Some("rgdevment"))
        .collect();
    assert_eq!(signed.len(), 10, "both languages and their pages");
    assert!(
        signed
            .iter()
            .all(|one| one.sown && one.guest && !one.archived)
    );
    let cover = signed.iter().find(|one| one.file == first.id).unwrap();
    assert_eq!(cover.lang.as_deref(), Some("es"));
}

#[test]
fn a_guide_someone_wrote_in_or_copied_is_left_where_it_is_when_the_signed_one_comes() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();
    sown(&mut session, "es", "Guía", WRITTEN_BY, shipped).unwrap();
    let cover = |session: &Session, lang: &str| {
        session
            .state
            .docs
            .values()
            .find(|one| one.page_of.is_none() && one.lang.as_deref() == Some(lang))
            .map(|one| (one.id, one.file.clone()))
            .unwrap()
    };
    let (untouched, _) = cover(&session, "es");
    let (written_in, file) = cover(&session, "en");
    let body = tisty_core::docs::read(&paths.docs(), &file).unwrap() + "\nalgo mío\n";
    tisty_core::docs::write(&paths.docs(), &file, &body).unwrap();
    session
        .commit(tisty_core::Op::DocSaid {
            id: written_in,
            d: tisty_core::event::Said::of(&body).by(None),
        })
        .unwrap();
    let copy = session.copy_doc(&untouched.to_string()).unwrap();
    let copy = session
        .state
        .docs
        .values()
        .find(|one| one.file == copy.id)
        .map(|one| one.id)
        .unwrap();

    super::signed_guide(&mut session, "es", "Guía", "rgdevment", shipped).unwrap();

    assert!(session.state.docs[&untouched].archived);
    assert!(
        !session.state.docs[&written_in].archived,
        "written in, so it is the person's"
    );
    assert!(
        !session.state.docs[&copy].archived,
        "a copy is the person's"
    );
}

#[test]
fn a_signed_guide_the_person_put_away_is_opened_again_rather_than_sown_twice() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let first = super::signed_guide(&mut session, "es", "Guía", "rgdevment", shipped).unwrap();
    let covers: Vec<tisty_core::model::DocId> = session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && one.sown)
        .map(|one| one.id)
        .collect();
    for id in covers {
        session.commit(tisty_core::Op::DocArchive { id }).unwrap();
    }

    let again = super::signed_guide_here(&mut session, "es").unwrap();
    assert_eq!(again.map(|one| one.id), Some(first.id));
    let signed = session
        .state
        .docs
        .values()
        .filter(|one| one.by.as_deref() == Some("rgdevment"))
        .count();
    assert_eq!(signed, 10, "nothing sown twice");
}
