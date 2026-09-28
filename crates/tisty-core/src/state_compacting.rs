use crate::event::{DeviceId, Event, Op};
use crate::state::State;
use ulid::Ulid;

fn ev(who: &str, ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId(who.into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn added(
    events: &mut Vec<Event>,
    ms: i64,
    page_of: Option<crate::model::DocId>,
) -> crate::model::DocId {
    let id = Ulid::generate();
    let told = State::replay(events);
    let order = crate::order::last_of(
        told.docs
            .values()
            .filter(|one| one.page_of == page_of)
            .map(|one| one.order.as_str()),
    );
    events.push(ev(
        "dev_a",
        ms,
        Op::DocAdd {
            id,
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: format!("dev_a-{ms:04}"),
                order,
                folder: None,
                page_of,
            },
        },
    ));
    id
}

fn rebased(events: &mut Vec<Event>, who: &str, ms: i64, run: &[crate::model::DocId]) {
    let keys: Vec<&str> = run.iter().map(|_| "").collect();
    for (n, (id, order)) in run.iter().zip(crate::order::resequenced(&keys)).enumerate() {
        events.push(ev(
            who,
            ms + n as i64,
            Op::DocMove {
                id: *id,
                d: crate::event::Filed {
                    folder: None,
                    page_of: None,
                    order: order.or_else(|| Some(crate::order::first())),
                },
            },
        ));
    }
}

#[test]
fn two_machines_rebasing_one_run_at_once_still_read_it_the_same_way() {
    let mut events = Vec::new();
    let up = added(&mut events, 1, None);
    let run: Vec<crate::model::DocId> = (2..6).map(|ms| added(&mut events, ms, Some(up))).collect();

    let mut theirs: Vec<crate::model::DocId> = run.clone();
    theirs.swap(1, 2);
    rebased(&mut events, "dev_a", 10, &run);
    rebased(&mut events, "dev_b", 11, &theirs);

    let mut sorted = events.clone();
    sorted.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let here: Vec<crate::model::DocId> = State::replay(&sorted)
        .pages_of(up)
        .iter()
        .map(|one| one.id)
        .collect();

    let mut shuffled = events;
    shuffled.reverse();
    shuffled.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let there: Vec<crate::model::DocId> = State::replay(&shuffled)
        .pages_of(up)
        .iter()
        .map(|one| one.id)
        .collect();

    assert_eq!(here, there, "a tie is broken the same way on both machines");
    assert_eq!(here.len(), run.len(), "and nothing falls out of the run");
}
