use super::*;
use crate::event::{DocAdd, Event, Flag, ListAdd, Said, TaskAdd};
use crate::{DeviceId, TaskId};

const ZONE: &str = "America/Santiago";

struct Desk {
    room: tempfile::TempDir,
    state: State,
    clock: i64,
}

impl Desk {
    fn new() -> Self {
        Self {
            room: tempfile::tempdir().unwrap(),
            state: State::default(),
            clock: 1_000,
        }
    }

    fn root(&self) -> &Path {
        self.room.path()
    }

    fn apply(&mut self, op: Op) {
        self.clock += 1;
        let at = jiff::Timestamp::from_millisecond(self.clock).unwrap();
        self.state
            .apply(&Event::new(DeviceId("dev_here".into()), at, op));
    }

    fn list(&mut self, name: &str) -> ListId {
        let id = ulid::Ulid::generate();
        self.apply(Op::ListAdd {
            id,
            d: ListAdd {
                name: name.into(),
                order: name.into(),
                color: None,
            },
        });
        id
    }

    fn task(&mut self, title: &str) -> TaskId {
        let id = ulid::Ulid::generate();
        let order = self.state.next_task_order();
        self.apply(Op::TaskAdd {
            id,
            d: TaskAdd::new(title.to_string(), order),
        });
        id
    }

    fn routine(&mut self, list: ListId, now: DateTime) -> TaskId {
        let asked = Asked {
            title: "Revisión semanal",
            tag: "revisión-semanal",
            list,
            weekday: Weekday::Friday,
            at: Time::constant(17, 0, 0, 0),
            zone: ZONE,
        };
        let plan = routine_plan(&self.state, &asked, now).unwrap();
        for op in plan.ops {
            self.apply(op);
        }
        plan.task
    }

    fn doc(
        &mut self,
        file: &str,
        front: &str,
        shape: impl FnOnce(&mut DocAdd),
    ) -> crate::model::DocId {
        std::fs::write(
            self.root().join(format!("{file}.md")),
            format!("{front}# {file}\n"),
        )
        .unwrap();
        let id = ulid::Ulid::generate();
        let mut add = DocAdd {
            file: file.into(),
            order: file.into(),
            wrote: Some(jiff::Timestamp::from_second(1_000_000_000).unwrap()),
            said: Some(Said {
                title: file.into(),
                ..Said::default()
            }),
            ..DocAdd::default()
        };
        shape(&mut add);
        self.apply(Op::DocAdd { id, d: add });
        id
    }
}

fn monday() -> DateTime {
    "2026-11-16T20:00".parse().unwrap()
}

#[test]
fn the_nudge_is_an_ordinary_routine_on_the_next_friday_ringing_at_five() {
    let mut desk = Desk::new();
    let home = desk.list("Personal");

    let id = desk.routine(home, monday());

    let task = &desk.state.tasks[&id];
    assert_eq!(task.title, "Revisión semanal");
    assert_eq!(task.list, Some(home));
    assert_eq!(
        task.date.as_ref().map(DateSpec::date),
        Some("2026-11-20".parse().unwrap())
    );
    assert_eq!(
        task.repeat,
        Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week
        }))
    );
    let rings: Vec<DateTime> = task.reminders.iter().map(|one| one.at).collect();
    assert_eq!(rings, vec!["2026-11-20T17:00".parse().unwrap()]);
    assert_eq!(routine(&desk.state).map(|one| one.id), Some(id));
}

#[test]
fn a_friday_already_past_five_waits_for_the_next_one() {
    let friday_night: DateTime = "2026-11-20T18:00".parse().unwrap();
    let friday_noon: DateTime = "2026-11-20T12:00".parse().unwrap();
    let five = Time::constant(17, 0, 0, 0);

    assert_eq!(
        next_on(Weekday::Friday, five, friday_night),
        "2026-11-27".parse().unwrap()
    );
    assert_eq!(
        next_on(Weekday::Friday, five, friday_noon),
        "2026-11-20".parse().unwrap()
    );
}

#[test]
fn it_is_due_on_its_day_and_after_and_not_before() {
    let mut desk = Desk::new();
    let home = desk.list("Personal");
    desk.routine(home, monday());

    assert!(due(&desk.state, "2026-11-19".parse().unwrap()).is_none());
    assert!(due(&desk.state, "2026-11-20".parse().unwrap()).is_some());
    assert!(due(&desk.state, "2026-11-23".parse().unwrap()).is_some());
}

#[test]
fn the_review_is_known_by_its_tag_after_a_rename_and_in_its_next_round() {
    let mut desk = Desk::new();
    let home = desk.list("Personal");
    let first = desk.routine(home, monday());
    desk.state.tasks.get_mut(&first).unwrap().title = "Ordenar la semana".into();

    let friday = "2026-11-20T17:30:00-03:00[America/Santiago]"
        .parse()
        .unwrap();
    for op in desk.state.completing(first, friday) {
        desk.apply(op);
    }

    let next = routine(&desk.state).expect("the next round is the review");
    assert_ne!(next.id, first);
    assert_eq!(
        next.date.as_ref().map(DateSpec::date),
        Some("2026-11-27".parse().unwrap())
    );
    assert_eq!(next.reminders.len(), 1, "the bell comes along");
}

#[test]
fn either_language_s_tag_names_it_and_a_near_word_does_not() {
    assert!(marks_the_review("revisión-semanal"));
    assert!(marks_the_review("revision-semanal"));
    assert!(marks_the_review("Weekly-Review"));
    assert!(!marks_the_review("revision"));
    assert!(!marks_the_review("review"));
}

#[test]
fn an_unknown_tag_is_traded_for_one_it_knows() {
    let mut desk = Desk::new();
    let home = desk.list("Personal");
    let asked = Asked {
        title: "Weekly review",
        tag: "chores",
        list: home,
        weekday: Weekday::Sunday,
        at: Time::constant(18, 0, 0, 0),
        zone: ZONE,
    };

    let plan = routine_plan(&desk.state, &asked, monday()).unwrap();
    for op in plan.ops {
        desk.apply(op);
    }

    assert_eq!(routine(&desk.state).map(|one| one.id), Some(plan.task));
}

#[test]
fn what_waits_is_counted_once_and_never_the_review_or_a_hidden_task() {
    let mut desk = Desk::new();
    let home = desk.list("Personal");
    desk.routine(home, monday());
    let mut review = desk.state.tasks.values().next().unwrap().clone();
    review.list = None;
    desk.state.tasks.insert(review.id, review);
    desk.task("Llamar a Jaime");
    let hidden = desk.task("Regalo para Marisol");
    desk.state.tasks.get_mut(&hidden).unwrap().hidden = true;
    desk.doc("hevp3wf9-0001", "---\nbrainstorm: on\n---\n", |add| {
        if let Some(said) = add.said.as_mut() {
            said.brainstorm = true;
        }
    });
    let sheet = desk.root().join("hevp3wf9-0001.md");
    std::fs::write(
        &sheet,
        "---\nbrainstorm: on\n---\n# Ideas\n\n> [!IDEA] 2026-11-16 10:00 · yellow\n> Un huerto\n\n> [!IDEA] 2026-11-16 10:01 · blue\n> Un perro\n",
    )
    .unwrap();
    let marked = desk.doc("hevp3wf9-0002", "", |_| {});
    desk.apply(Op::DocFlag {
        id: marked,
        d: Flag {
            body: "Una copia".into(),
            at: None,
            by: None,
            via: None,
            reason: None,
            instead: None,
        },
    });
    desk.doc("hevp3wf9-0003", "", |_| {});

    let counted = waiting(desk.root(), &desk.state, &Used::new(), 1_800_000_000);

    assert_eq!(
        counted,
        Waiting {
            unlisted: 1,
            ideas: crate::brainstorm::ideas(&std::fs::read_to_string(&sheet).unwrap()),
            marked: 1,
            forgotten: 1,
        }
    );
    assert_eq!(counted.ideas, 2);
}
