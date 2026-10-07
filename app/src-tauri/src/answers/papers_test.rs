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
