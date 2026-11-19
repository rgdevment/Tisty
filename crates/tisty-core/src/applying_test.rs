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

fn gist(id: DocId, summary: &str) -> Op {
    Op::DocGist {
        id,
        d: Gist {
            print: "print".into(),
            summary: summary.into(),
            notes: String::new(),
        },
    }
}

fn summary_of(state: &State, id: DocId) -> Option<&str> {
    state.agent_notes.get(&id).map(|one| one.summary.as_str())
}

#[test]
fn a_later_note_replaces_the_one_before_and_is_dated_by_its_event() {
    let (id, add) = added("dev_agent-0001", None);
    let state = State::replay(&[
        ev(1, add),
        ev(2, gist(id, "first")),
        ev(3, gist(id, "second")),
    ]);

    assert_eq!(summary_of(&state, id), Some("second"));
    assert_eq!(
        state.agent_notes[&id].at,
        jiff::Timestamp::from_millisecond(3).unwrap()
    );
}

#[test]
fn a_document_taken_away_takes_its_note_and_its_pages_notes_with_it() {
    let (book, add_book) = added("dev_agent-0001", None);
    let (page, add_page) = added("dev_agent-0002", Some(book));
    let state = State::replay(&[
        ev(1, add_book),
        ev(2, add_page),
        ev(3, gist(book, "the book")),
        ev(4, gist(page, "the page")),
        ev(5, Op::DocDelete { id: book }),
    ]);

    assert!(state.agent_notes.is_empty(), "{:?}", state.agent_notes);
}
