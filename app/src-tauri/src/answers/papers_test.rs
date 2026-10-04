use super::told_as_settled;
use crate::Session;
use tisty_core::{Op, Paths};

#[test]
fn what_a_settled_document_holds_is_answered_for_at_once() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();
    let first = "# Acta\n\nlo de aqui\n";
    let made = tisty_core::docs::create(&paths.docs(), &tisty_core::DeviceId("uno".into()), first)
        .unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: Some(tisty_core::event::Said::of(first)),
                file: made.id.clone(),
                order: "a0".into(),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();

    let kept_theirs = "# Acta\n\nlo de la otra maquina\n";
    tisty_core::docs::write(&paths.docs(), &made.id, kept_theirs).unwrap();
    told_as_settled(&mut session, &made.id);

    let paper = session
        .state
        .docs
        .values()
        .find(|one| one.file == made.id)
        .unwrap();
    assert_eq!(
        paper.print,
        tisty_core::event::Said::of(kept_theirs).print,
        "every other machine would ask again about the body this one already settled"
    );
}
