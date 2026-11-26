use super::*;
use crate::answers::papers::one_step_back;
use tisty_core::event::{StepAdd, TaskAdd};

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn on_disk(session: &Session, id: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), id).unwrap()
}

fn agent_writes(session: &Session, id: &str, body: &str) {
    tisty_core::docs::write(&session.paths.docs(), id, body).unwrap();
}

fn told(said: &[&str]) -> String {
    format!(
        "{}
",
        said.join(
            "

"
        )
    )
}

fn base() -> String {
    told(&["# Doc", "A", "medio", "otro", "Z"])
}

fn mine() -> String {
    told(&["# Doc", "A mia", "medio", "otro", "Z"])
}

fn theirs() -> String {
    told(&["# Doc", "A del agente", "medio", "otro", "Z del agente"])
}

fn task_with_page(session: &mut Session, body: &str) -> (tisty_core::TaskId, String) {
    let id = ulid::Ulid::generate();
    session
        .commit_all(vec![
            Op::TaskAdd {
                id,
                d: TaskAdd::new("Mudanza", "a0"),
            },
            Op::StepAdd {
                id,
                d: StepAdd {
                    step: ulid::Ulid::generate(),
                    text: "cajas".into(),
                    order: "a0".into(),
                },
            },
        ])
        .unwrap();
    let (made, _) = tisty_core::task_docs::made_with(
        &session.paths.docs(),
        &session.writer(),
        &session.state,
        id,
        body,
        None,
    )
    .unwrap();
    let Op::DocAdd { d, .. } = &made else {
        panic!("a task's page is made by adding a document");
    };
    let file = d.file.clone();
    session.commit(made).unwrap();
    (id, file)
}

#[test]
fn a_task_page_standing_by_mine_keeps_what_the_agent_wrote_in_other_blocks() {
    let (_tmp, mut session) = session();
    let (task, file) = task_with_page(&mut session, &base());
    agent_writes(&session, &file, &theirs());
    let session = Mutex::new(session);
    let id = task.to_string();

    let refused = merging(&session, id.clone(), base(), mine(), false);
    assert!(!refused.unwrap());
    assert_eq!(on_disk(&held(&session), &file), theirs());

    let landed = merging(&session, id, base(), mine(), true);

    assert!(landed.unwrap());
    let session = held(&session);
    let kept = told(&["# Doc", "A mia", "medio", "otro", "Z del agente"]);
    let page = on_disk(&session, &file);
    assert!(page.starts_with(&kept), "{page:?}");
    assert_eq!(one_step_back(&session, &file), Some(theirs()));
}

#[cfg(windows)]
#[test]
fn a_task_page_another_program_holds_open_is_held_open_not_a_failure_whichever_way_it_is_saved() {
    use std::os::windows::fs::OpenOptionsExt;

    for (share, mine_stands) in [(1u32, true), (1, false), (3, true), (3, false)] {
        let (_tmp, mut session) = session();
        let (task, file) = task_with_page(&mut session, &base());
        agent_writes(
            &session,
            &file,
            &told(&["# Doc", "A", "medio", "otro", "Z del agente"]),
        );
        let at = tisty_core::docs::resolve(&session.paths.docs(), &file).unwrap();
        let _held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(share)
            .open(&at)
            .unwrap();
        let session = Mutex::new(session);

        let refused = merging(&session, task.to_string(), base(), mine(), mine_stands);

        assert_eq!(
            refused.unwrap_err().code,
            "documentHeldOpen",
            "share {share} mine {mine_stands}"
        );
    }
}
