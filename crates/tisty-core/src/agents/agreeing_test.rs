use super::*;
use crate::event::{DocAdd, Event, Op, Said};
use crate::history::{Step, Writer};
use ulid::Ulid;

const HERE: &str = "dev_here";
const THERE: &str = "dev_there";
const AGENT: &str = "dev_agent";
const FILE: &str = "agre1234-0001";

struct Room {
    data: tempfile::TempDir,
    state: State,
    id: Ulid,
    ms: i64,
}

impl Room {
    fn new(who: &str, body: &str, agreed: bool) -> Self {
        let mut room = Self {
            data: tempfile::tempdir().unwrap(),
            state: State::default(),
            id: Ulid::generate(),
            ms: 0,
        };
        room.apply(
            AGENT,
            Op::DeviceJoin {
                d: DeviceId(AGENT.into()),
                k: Some(crate::event::DeviceKind::Agent),
                p: None,
            },
        );
        let said = match agreed {
            true => Said::of(body).agreed(),
            false => Said::of(body),
        };
        let id = room.id;
        room.apply(
            who,
            Op::DocAdd {
                id,
                d: DocAdd {
                    file: FILE.into(),
                    order: "a0".into(),
                    said: Some(said),
                    ..Default::default()
                },
            },
        );
        room
    }

    fn apply(&mut self, who: &str, op: Op) {
        self.ms += 1;
        let at = jiff::Timestamp::from_millisecond(self.ms).unwrap();
        self.state.apply(&Event::new(DeviceId(who.into()), at, op));
    }

    fn says(&mut self, who: &str, was: &str, body: &str) {
        Writer::unanswered(self.data.path())
            .replaces(FILE, was.as_bytes(), body, Step::Stays)
            .unwrap();
        let id = self.id;
        self.apply(
            who,
            Op::DocSaid {
                id,
                d: Said::of(body),
            },
        );
    }

    fn agreeing(&self, body: &str) -> Agreeing {
        let kept = self.state.docs.get(&self.id).unwrap();
        agreeing(
            &self.state,
            self.data.path(),
            kept,
            body,
            &DeviceId(HERE.into()),
        )
    }
}

#[test]
fn words_never_approved_say_so() {
    let room = Room::new(THERE, "Fresh.\n", false);
    let said = room.agreeing("Fresh.\n");
    assert_eq!(said.approved, Approved::Never);
    assert_eq!(said.hand, Hand::Elsewhere);
}

#[test]
fn the_approved_words_kept_here_are_handed_back_to_compare() {
    let mut room = Room::new(HERE, "Approved.\n", true);
    room.says(AGENT, "Approved.\n", "Changed by an agent.\n");
    let said = room.agreeing("Changed by an agent.\n");
    assert_eq!(said.approved, Approved::Kept("Approved.\n".into()));
    assert_eq!(said.hand, Hand::Agent);
}

#[test]
fn approved_words_this_computer_never_kept_are_not_made_up() {
    let mut room = Room::new(THERE, "Approved there.\n", true);
    let id = room.id;
    room.apply(
        THERE,
        Op::DocSaid {
            id,
            d: Said::of("Changed there.\n"),
        },
    );
    let said = room.agreeing("Changed there.\n");
    assert_eq!(said.approved, Approved::Elsewhere);
    assert_eq!(said.hand, Hand::Elsewhere);
}

#[test]
fn words_on_disk_no_line_of_the_log_wrote_changed_outside_tisty() {
    let room = Room::new(HERE, "Approved.\n", true);
    assert_eq!(room.agreeing("Edited on disk.\n").hand, Hand::Outside);
}

#[test]
fn words_seen_before_come_back_as_an_earlier_version() {
    let mut room = Room::new(HERE, "One.\n", true);
    room.says(HERE, "One.\n", "Two.\n");
    room.says(THERE, "Two.\n", "One.\n");
    let id = room.id;
    room.apply(
        THERE,
        Op::DocSaid {
            id,
            d: Said::of("Three.\n"),
        },
    );
    room.says(THERE, "Three.\n", "Two.\n");
    assert_eq!(room.agreeing("Two.\n").hand, Hand::Earlier);
}

#[test]
fn words_this_computer_wrote_without_approving_say_so() {
    let mut room = Room::new(THERE, "Approved there.\n", true);
    let id = room.id;
    room.apply(
        HERE,
        Op::DocSaid {
            id,
            d: Said::of("Mine.\n"),
        },
    );
    assert_eq!(room.agreeing("Mine.\n").hand, Hand::Here);
}
