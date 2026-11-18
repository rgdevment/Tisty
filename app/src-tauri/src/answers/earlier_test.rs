use tisty_core::event::{DocAdd, Said, StepAdd, StepRef, TaskAdd};
use tisty_core::{Paths, Store};

use super::*;

const CLAUDE: &str = "Claude Code";

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
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
            d: DocAdd {
                file: made.id.clone(),
                order: tisty_core::order::first(),
                said: Some(Said::of(body)),
                ..DocAdd::default()
            },
        })
        .unwrap();
    made.id
}

fn typed(session: &mut Session, id: &str, body: &str) {
    tisty_core::docs::write_by(&session.paths.docs(), &session.writer(), id, body).unwrap();
    let hand = signing(&session.state);
    session.retell(id, body, hand);
}

fn told_from(session: &mut Session, device: &DeviceId, via: Option<&str>, id: &str, body: &str) {
    let doc = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .unwrap()
        .id;
    let mut theirs = Store::open(session.paths.store(), device.clone())
        .unwrap()
        .speaking_through(via.map(str::to_string));
    theirs
        .append(Op::DocSaid {
            id: doc,
            d: Said::of(body),
        })
        .unwrap();
    session.reload().unwrap();
}

fn assistant_typed(session: &mut Session, id: &str, body: &str) {
    let by = history::Writer::now(session.paths.data(), Some(CLAUDE.into()));
    tisty_core::docs::write_by(&session.paths.docs(), &by, id, body).unwrap();
    session.retell(id, body, None);
}

fn another_machine() -> DeviceId {
    DeviceId(tisty_core::config::new_device_id())
}

fn body_of(session: &Session, id: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), id).unwrap()
}

fn print(body: &str) -> String {
    tisty_core::docs::print_of_text(body)
}

fn rows(session: &Session, id: &str) -> Vec<history::Row> {
    let log = session.log().unwrap();
    listed(&asked(session, id), &log, id, jiff::Timestamp::now()).unwrap()
}

const UNO: &str = "# Acta\n\nuno\n";
const DOS: &str = "# Acta\n\ndos\n";
const TRES: &str = "# Acta\n\ntres\n";

#[test]
fn a_document_lists_what_it_said_before_and_who_wrote_it() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    typed(&mut session, &doc, DOS);

    let all = rows(&session, &doc);

    assert_eq!(all.len(), 1);
    assert_eq!(all[0].print, print(UNO));
    assert!(all[0].here);
    assert_eq!(all[0].via, None, "the person wrote it");
    assert_eq!(all[0].device.as_ref(), Some(&session.config.device_id));
    assert!(all[0].wrote.is_some());
    assert_eq!(all[0].bytes, UNO.len() as u64);
}

#[test]
fn a_document_nobody_wrote_over_has_no_history() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);

    assert!(rows(&session, &doc).is_empty());
    let nowhere = listed(
        &asked(&session, "nope-0001"),
        &[],
        "nope-0001",
        jiff::Timestamp::now(),
    );
    assert_eq!(nowhere.unwrap_err().code, "noSuchDoc");
}

#[test]
fn what_another_machine_replaced_is_listed_marked_and_cannot_be_opened_here() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    let there = another_machine();
    told_from(&mut session, &there, Some(CLAUDE), &doc, DOS);
    told_from(&mut session, &there, None, &doc, TRES);

    let all = rows(&session, &doc);

    assert_eq!(
        all.len(),
        1,
        "only the body written elsewhere and cut there"
    );
    assert_eq!(all[0].print, print(DOS));
    assert!(!all[0].here);
    assert_eq!(all[0].via.as_deref(), Some(CLAUDE));
    assert_eq!(all[0].device.as_ref(), Some(&there));
    assert_eq!(all[0].kept_by.as_ref(), Some(&there));
    assert_eq!(
        read(&session, &doc, &all[0].print).unwrap_err().code,
        "versionGone"
    );
    assert_eq!(
        went_back_to(&mut session, &doc, &print(DOS))
            .unwrap_err()
            .code,
        "versionGone"
    );
    assert_eq!(body_of(&session, &doc), UNO);
}

#[test]
fn an_assistant_this_machine_hosts_does_not_write_elsewhere() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    let agent = another_machine();
    Store::open(session.paths.store(), agent.clone())
        .unwrap()
        .append_batch(vec![
            Op::DeviceJoin {
                d: agent.clone(),
                k: Some(tisty_core::event::DeviceKind::Agent),
                p: None,
            },
            Op::DeviceHost {
                d: agent.clone(),
                of: session.config.device_id.clone(),
                p: None,
            },
        ])
        .unwrap();
    session.reload().unwrap();
    assert_eq!(
        session.state.hosts.get(&agent),
        Some(&session.config.device_id)
    );
    told_from(&mut session, &agent, Some(CLAUDE), &doc, DOS);
    told_from(&mut session, &agent, None, &doc, TRES);

    assert!(
        rows(&session, &doc).is_empty(),
        "a body this machine no longer holds was listed as kept on another"
    );
}

#[test]
fn going_back_to_a_version_can_be_undone_by_the_one_step_back() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    typed(&mut session, &doc, DOS);
    typed(&mut session, &doc, TRES);
    assert_eq!(read(&session, &doc, &print(UNO)).unwrap(), UNO);

    let said = went_back_to(&mut session, &doc, &print(UNO)).unwrap();

    assert_eq!(said.title, "Acta");
    assert_eq!(body_of(&session, &doc), UNO);
    assert_eq!(one_step_back(&session, &doc).as_deref(), Some(TRES));
    let now = session
        .state
        .docs
        .values()
        .find(|one| one.file == doc)
        .unwrap();
    assert_eq!(
        now.print.as_deref(),
        Some(print(UNO).as_str()),
        "the log was not told"
    );

    went_back(&mut session, &doc).unwrap();
    assert_eq!(body_of(&session, &doc), TRES);
}

#[test]
fn going_back_to_what_the_document_already_says_is_refused_and_keeps_no_step_back() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    typed(&mut session, &doc, DOS);
    typed(&mut session, &doc, UNO);
    assert!(
        rows(&session, &doc).is_empty(),
        "what it says now was listed as earlier"
    );
    let before = one_step_back(&session, &doc);

    let why = went_back_to(&mut session, &doc, &print(UNO)).unwrap_err();

    assert_eq!(why.code, "versionSame");
    assert_eq!(body_of(&session, &doc), UNO);
    assert_eq!(one_step_back(&session, &doc), before);
}

#[test]
fn a_locked_document_goes_back_to_nothing() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    typed(&mut session, &doc, DOS);
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == doc)
        .unwrap()
        .id;
    session.commit(Op::DocLock { id }).unwrap();

    let why = went_back_to(&mut session, &doc, &print(UNO)).unwrap_err();

    assert_eq!(why.code, "documentLocked");
    assert_eq!(body_of(&session, &doc), DOS);
}

#[test]
fn emptying_a_history_takes_its_versions_and_the_step_back_but_not_the_document() {
    let (_tmp, mut session) = session();
    let doc = filed(&mut session, UNO);
    typed(&mut session, &doc, DOS);
    went_back_to(&mut session, &doc, &print(UNO)).unwrap();
    assert!(one_step_back(&session, &doc).is_some());
    let kept = history::versions(session.paths.data(), &doc).len();
    assert!(
        kept >= 1 && rows(&session, &doc).is_empty(),
        "the shelf holds what the list does not show"
    );

    let gone = emptied(&session, &doc).unwrap();

    assert_eq!(gone, kept);
    assert!(history::versions(session.paths.data(), &doc).is_empty());
    assert!(one_step_back(&session, &doc).is_none());
    assert_eq!(body_of(&session, &doc), UNO);
    assert_eq!(emptied(&session, &doc).unwrap(), 0);
}

#[test]
fn only_the_limits_offered_are_kept() {
    let (_tmp, mut session) = session();
    assert_eq!(session.config.history_days(), 30);

    assert_eq!(kept_days(&mut session, 5).unwrap(), 5);
    let kept = tisty_core::Config::load_or_init(&session.paths).unwrap();
    assert_eq!(
        kept.history_days(),
        5,
        "the choice did not reach the settings file"
    );

    assert_eq!(
        kept_days(&mut session, 7).unwrap_err().code,
        "notAHistoryLimit"
    );
    assert_eq!(session.config.history_days(), 5);
    for days in history::DAYS {
        assert_eq!(kept_days(&mut session, days).unwrap(), days);
    }
}

fn task_with_page(
    session: &mut Session,
    steps: &[&str],
    body: &str,
) -> (tisty_core::TaskId, String) {
    let id = ulid::Ulid::generate();
    let mut ops = vec![Op::TaskAdd {
        id,
        d: TaskAdd::new("Mudanza", "a0"),
    }];
    for (n, text) in steps.iter().enumerate() {
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step: ulid::Ulid::generate(),
                text: (*text).to_string(),
                order: format!("a{n}"),
            },
        });
    }
    session.commit_all(ops).unwrap();
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

fn steps_of(session: &Session, id: tisty_core::TaskId) -> Vec<(String, bool)> {
    let task = &session.state.tasks[&id];
    tisty_core::stages::ordered(task)
        .into_iter()
        .map(|one| (one.text.clone(), one.done))
        .collect()
}

const PAGE: &str = "---\ntask: x\n---\n# Mudanza\n\nprimera idea\n\n- [ ] cajas\n";
const PAGE_LATER: &str = "---\ntask: x\n---\n# Mudanza\n\notra idea\n\n- [ ] cajas\n- [ ] camión\n";

#[test]
fn a_task_s_page_goes_back_to_its_words_and_keeps_the_steps_and_ticks_the_task_has() {
    let (_tmp, mut session) = session();
    let (task, file) = task_with_page(&mut session, &["cajas"], PAGE);
    let step = session.state.tasks[&task].steps[0].id;
    assistant_typed(&mut session, &file, PAGE_LATER);
    let added = ulid::Ulid::generate();
    session
        .commit_all(vec![
            Op::StepAdd {
                id: task,
                d: StepAdd {
                    step: added,
                    text: "camión".into(),
                    order: "a5".into(),
                },
            },
            Op::StepDone {
                id: task,
                d: StepRef { step },
            },
        ])
        .unwrap();

    went_back_to(&mut session, &file, &print(PAGE)).unwrap();

    let now = body_of(&session, &file);
    assert!(now.contains("primera idea") && !now.contains("otra idea"));
    assert!(
        now.contains("- [x] cajas"),
        "a step done since was unticked by an old page: {now}"
    );
    assert!(
        now.contains("camión"),
        "a step added since lost its line: {now}"
    );
    assert_eq!(
        steps_of(&session, task),
        vec![("cajas".to_string(), true), ("camión".to_string(), false)]
    );

    went_back(&mut session, &file).unwrap();
    assert!(body_of(&session, &file).contains("otra idea"));
    assert_eq!(steps_of(&session, task).len(), 2);
}

#[test]
fn a_box_the_old_page_names_and_the_task_no_longer_has_comes_back_as_a_step() {
    let (_tmp, mut session) = session();
    let (task, file) = task_with_page(&mut session, &["cajas"], PAGE);
    let step = session.state.tasks[&task].steps[0].id;
    let without = "---\ntask: x\n---\n# Mudanza\n\notra idea\n";
    assistant_typed(&mut session, &file, without);
    session
        .commit(Op::StepRemove {
            id: task,
            d: StepRef { step },
        })
        .unwrap();
    assert!(steps_of(&session, task).is_empty());

    went_back_to(&mut session, &file, &print(PAGE)).unwrap();

    assert_eq!(steps_of(&session, task), vec![("cajas".to_string(), false)]);
}

#[test]
fn a_closed_task_s_page_is_read_and_never_gone_back_to() {
    let (_tmp, mut session) = session();
    let (task, file) = task_with_page(&mut session, &["cajas"], PAGE);
    assistant_typed(&mut session, &file, PAGE_LATER);
    session
        .commit(Op::TaskDone {
            id: task,
            filled: false,
        })
        .unwrap();

    assert_eq!(rows(&session, &file).len(), 1);
    assert_eq!(read(&session, &file, &print(PAGE)).unwrap(), PAGE);
    let why = went_back_to(&mut session, &file, &print(PAGE)).unwrap_err();

    assert_eq!(why.code, "taskClosed");
    assert_eq!(body_of(&session, &file), PAGE_LATER);
}
