use super::*;
use crate::answers::papers::one_step_back;
use tisty_core::Op;
use tisty_core::event::{DocAdd, Said};

use crate::answers::turns::Turn;

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

fn on_disk(session: &Session, id: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), id).unwrap()
}

fn agent_writes(session: &Session, id: &str, body: &str) {
    tisty_core::docs::write(&session.paths.docs(), id, body).unwrap();
}

fn told(said: &[&str]) -> String {
    format!("{}\n", said.join("\n\n"))
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

fn woven(done: Merge) -> Woven {
    match done {
        Merge::Woven(woven) => woven,
        Merge::Torn { torn, .. } => panic!("it should have woven, but said torn: {torn:?}"),
    }
}

fn turn(n: u64) -> Option<Turn> {
    Some(Turn {
        hand: "a".to_string(),
        n,
    })
}

#[test]
fn standing_by_mine_keeps_what_the_agent_wrote_in_other_blocks() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());

    let done = merged(&mut session, id.clone(), Some(&base()), &mine(), true).unwrap();

    let expected = told(&["# Doc", "A mia", "medio", "otro", "Z del agente"]);
    assert_eq!(on_disk(&session, &id), expected);
    assert_eq!(woven(done).body, expected);
}

#[test]
fn without_standing_by_mine_a_clash_writes_nothing_and_says_how_many_places() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());

    let done = merged(&mut session, id.clone(), Some(&base()), &mine(), false).unwrap();

    assert!(matches!(done, Merge::Torn { torn: Some(1), .. }));
    assert_eq!(on_disk(&session, &id), theirs());
}

#[test]
fn a_clash_with_no_base_to_weave_from_says_the_whole_page_would_go_back() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());

    let done = merged(&mut session, id.clone(), None, &mine(), false).unwrap();

    assert!(matches!(done, Merge::Torn { torn: None, .. }));
    assert_eq!(on_disk(&session, &id), theirs());
}

#[test]
fn the_agent_writing_between_the_clash_and_the_click_is_woven_against_the_new_disk() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());
    let seen = merged(&mut session, id.clone(), Some(&base()), &mine(), false).unwrap();
    assert!(matches!(seen, Merge::Torn { torn: Some(1), .. }));
    let later = told(&[
        "# Doc",
        "A del agente",
        "medio",
        "otro nuevo",
        "Z del agente",
    ]);
    agent_writes(&session, &id, &later);

    merged(&mut session, id.clone(), Some(&base()), &mine(), true).unwrap();

    assert_eq!(
        on_disk(&session, &id),
        told(&["# Doc", "A mia", "medio", "otro nuevo", "Z del agente"]),
        "what the agent wrote after the bar was shown was written over blind"
    );
}

#[test]
fn standing_by_mine_keeps_what_it_replaced_as_the_step_back() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());

    merged(&mut session, id.clone(), Some(&base()), &mine(), true).unwrap();

    assert_eq!(one_step_back(&session, &id), Some(theirs()));
}

#[test]
fn the_whole_page_put_back_for_lack_of_a_base_is_kept_as_a_version_to_go_back_to() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());

    let done = merged(&mut session, id.clone(), None, &mine(), true).unwrap();

    assert_eq!(woven(done).body, mine());
    assert_eq!(on_disk(&session, &id), mine());
    assert_eq!(
        one_step_back(&session, &id),
        Some(theirs()),
        "the agent's words were erased with no way back"
    );
}

#[test]
fn if_the_version_cannot_be_kept_nothing_is_written() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());
    let history = session.paths.data().join(tisty_core::history::FOLDER);
    let _ = std::fs::remove_dir_all(&history);
    std::fs::write(&history, "not a folder").unwrap();

    let done = merged(&mut session, id.clone(), Some(&base()), &mine(), true);

    assert!(done.is_err(), "it wrote without keeping what it replaced");
    assert_eq!(on_disk(&session, &id), theirs());
}

#[test]
fn the_same_turn_sent_again_weaves_to_the_same_page_and_keeps_the_same_step_back() {
    let (_tmp, mut session) = session();
    let id = filed(&mut session, &base());
    agent_writes(&session, &id, &theirs());
    let go = |session: &mut Session| {
        super::super::turns::in_turn(
            session,
            &id,
            turn(1),
            |session| merged(session, id.clone(), Some(&base()), &mine(), true),
            |done| matches!(done, Merge::Woven(_)),
        )
        .unwrap()
    };

    go(&mut session);
    let once = on_disk(&session, &id);
    go(&mut session);

    assert_eq!(on_disk(&session, &id), once);
    assert_eq!(one_step_back(&session, &id), Some(theirs()));
}

#[test]
fn a_page_kept_with_crlf_stays_so_after_standing_by_mine() {
    let (_tmp, mut session) = session();
    let crlf = |text: String| text.replace('\n', "\r\n");
    let id = filed(&mut session, &crlf(base()));
    agent_writes(&session, &id, &crlf(theirs()));

    merged(&mut session, id.clone(), Some(&base()), &mine(), true).unwrap();

    let now = on_disk(&session, &id);
    assert_eq!(
        now,
        crlf(told(&["# Doc", "A mia", "medio", "otro", "Z del agente"]))
    );
    assert!(!now.replace("\r\n", "").contains('\n'), "{now:?}");
}

#[cfg(windows)]
#[test]
fn a_page_another_program_shares_for_reading_is_held_open_not_a_failure_whichever_way_it_is_saved()
{
    use std::os::windows::fs::OpenOptionsExt;

    for (share, mine_stands) in [(1u32, true), (1, false), (3, true), (3, false)] {
        let (_tmp, mut session) = session();
        let id = filed(&mut session, &base());
        agent_writes(
            &session,
            &id,
            &told(&["# Doc", "A", "medio", "otro", "Z del agente"]),
        );
        let file = tisty_core::docs::resolve(&session.paths.docs(), &id).unwrap();
        let _held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(share)
            .open(&file)
            .unwrap();

        let refused = merged(
            &mut session,
            id.clone(),
            Some(&base()),
            &mine(),
            mine_stands,
        )
        .err()
        .expect("a held page cannot be written");

        assert_eq!(
            refused.code, "documentHeldOpen",
            "share {share} mine {mine_stands}"
        );
    }
}

#[test]
fn a_clash_where_both_changed_the_properties_says_so_and_one_where_only_the_words_clashed_does_not()
{
    let (_tmp, mut session) = session();
    let with_title = |title: &str, body: &str| format!("---\ntitle: {title}\n---\n\n{body}");
    let base = with_title("uno", &base());
    let id = filed(&mut session, &base);
    agent_writes(&session, &id, &with_title("tres", &theirs()));
    let mine_front = with_title("dos", &mine());

    let both = merged(&mut session, id.clone(), Some(&base), &mine_front, false).unwrap();
    assert!(matches!(
        both,
        Merge::Torn {
            torn: Some(1),
            front: true
        }
    ));

    agent_writes(&session, &id, &with_title("uno", &theirs()));
    let words_only = merged(&mut session, id.clone(), Some(&base), &mine_front, false).unwrap();
    assert!(matches!(
        words_only,
        Merge::Torn {
            torn: Some(1),
            front: false
        }
    ));
}

#[test]
fn standing_by_mine_keeps_the_agents_words_in_the_block_beside_the_clash() {
    let (_tmp, mut session) = session();
    let base = told(&["p1", "p2", "p3", "p4", "p5", "p6"]);
    let mine = told(&["M1", "p2", "p3", "p4", "p5", "M6"]);
    let theirs = told(&["T1", "T2", "p3", "T4", "p5", "T6"]);
    let id = filed(&mut session, &base);
    agent_writes(&session, &id, &theirs);

    let done = merged(&mut session, id.clone(), Some(&base), &mine, true).unwrap();

    let expected = told(&["M1", "T2", "p3", "T4", "p5", "M6"]);
    assert_eq!(on_disk(&session, &id), expected);
    assert_eq!(woven(done).body, expected);
}
