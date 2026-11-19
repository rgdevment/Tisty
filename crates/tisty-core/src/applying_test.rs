use crate::event::{DeviceId, DocAdd, Event, Gist};
use crate::model::DocId;
use crate::{Op, State};
use ulid::Ulid;

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_agent".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn added(file: &str, page_of: Option<DocId>) -> (DocId, Op) {
    let id = Ulid::generate();
    let d = DocAdd {
        wrote: None,
        guest: false,
        sown: false,
        made: None,
        by: None,
        said: None,
        file: file.into(),
        order: "a0".into(),
        folder: None,
        page_of,
        task_of: None,
        edited: None,
    };
    (id, Op::DocAdd { id, d })
}

fn gist(id: DocId, summary: &str, at: Option<i64>) -> Op {
    Op::DocGist {
        id,
        d: Gist {
            print: "print".into(),
            summary: summary.into(),
            notes: String::new(),
            at: at.map(|ms| jiff::Timestamp::from_millisecond(ms).unwrap()),
        },
    }
}

fn summary_of(state: &State, id: DocId) -> Option<&str> {
    state.agent_notes.get(&id).map(|one| one.summary.as_str())
}

#[test]
fn a_later_note_replaces_the_one_before() {
    let (id, add) = added("dev_agent-0001", None);
    let state = State::replay(&[
        ev(1, add),
        ev(2, gist(id, "first", None)),
        ev(3, gist(id, "second", None)),
    ]);

    assert_eq!(summary_of(&state, id), Some("second"));
    assert_eq!(
        state.agent_notes[&id].at,
        jiff::Timestamp::from_millisecond(3).unwrap(),
        "a note written now is dated by its event"
    );
}

#[test]
fn a_note_carried_late_from_another_machine_never_buries_a_newer_one() {
    let (id, add) = added("dev_agent-0001", None);
    let state = State::replay(&[
        ev(1, add),
        ev(5, gist(id, "written here", None)),
        ev(9, gist(id, "kept there long ago", Some(2))),
    ]);

    assert_eq!(summary_of(&state, id), Some("written here"));
}

#[test]
fn a_note_carried_from_another_machine_counts_from_when_it_was_written() {
    let (id, add) = added("dev_agent-0001", None);
    let state = State::replay(&[
        ev(1, add),
        ev(5, gist(id, "written here", None)),
        ev(6, gist(id, "kept there later", Some(7))),
    ]);

    assert_eq!(summary_of(&state, id), Some("kept there later"));
    assert_eq!(
        state.agent_notes[&id].at,
        jiff::Timestamp::from_millisecond(7).unwrap()
    );
}

#[test]
fn a_document_taken_away_takes_its_note_and_its_pages_notes_with_it() {
    let (book, add_book) = added("dev_agent-0001", None);
    let (page, add_page) = added("dev_agent-0002", Some(book));
    let state = State::replay(&[
        ev(1, add_book),
        ev(2, add_page),
        ev(3, gist(book, "the book", None)),
        ev(4, gist(page, "the page", None)),
        ev(5, Op::DocDelete { id: book }),
    ]);

    assert!(state.agent_notes.is_empty(), "{:?}", state.agent_notes);
}

#[test]
fn of_two_notes_written_at_the_same_moment_the_one_replayed_last_stays() {
    let (id, add) = added("dev_agent-0001", None);
    let state = State::replay(&[
        ev(1, add),
        ev(5, gist(id, "written here", None)),
        ev(6, gist(id, "kept there at the same moment", Some(5))),
    ]);

    assert_eq!(
        summary_of(&state, id),
        Some("kept there at the same moment")
    );
}
