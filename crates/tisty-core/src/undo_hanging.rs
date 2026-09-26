use super::*;
use crate::event::DeviceId;
use ulid::Ulid;

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn doc(
    events: &mut Vec<Event>,
    ms: i64,
    folder: Option<crate::model::FolderId>,
) -> crate::model::DocId {
    let id = Ulid::generate();
    let told = State::replay(events);
    let order = crate::order::last_of(
        told.docs
            .values()
            .filter(|one| one.page_of.is_none() && one.folder == folder)
            .map(|one| one.order.as_str()),
    );
    events.push(ev(
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
                folder,
                page_of: None,
            },
        },
    ));
    id
}

fn folder(events: &mut Vec<Event>, ms: i64) -> crate::model::FolderId {
    let id = Ulid::generate();
    events.push(ev(
        ms,
        Op::FolderAdd {
            id,
            d: crate::event::FolderAdd {
                name: format!("Folder {ms}"),
                parent: None,
                order: crate::order::first(),
                icon: None,
                color: None,
            },
        },
    ));
    id
}

fn hang(events: &mut Vec<Event>, ms: i64, id: crate::model::DocId, up: crate::model::DocId) {
    let told = State::replay(events);
    let order = crate::order::last_of(
        told.docs
            .values()
            .filter(|one| one.page_of == Some(up))
            .map(|one| one.order.as_str()),
    );
    events.push(ev(
        ms,
        Op::DocMove {
            id,
            d: crate::event::Filed {
                folder: None,
                page_of: Some(Some(up)),
                order: Some(order),
            },
        },
    ));
}

fn unhang(events: &mut Vec<Event>, ms: i64, id: crate::model::DocId) -> State {
    let told = State::replay(events);
    events.push(ev(
        ms,
        Op::DocMove {
            id,
            d: unhung(events, &told, id),
        },
    ));
    State::replay(events)
}

#[test]
fn unhanging_puts_it_back_in_the_folder_and_the_place_it_came_from() {
    let mut events = Vec::new();
    let home = folder(&mut events, 1);
    let work = folder(&mut events, 2);
    let one = doc(&mut events, 3, Some(home));
    let two = doc(&mut events, 4, Some(home));
    let up = doc(&mut events, 5, Some(work));

    let was = State::replay(&events).docs[&one].clone();
    hang(&mut events, 6, one, up);
    assert_eq!(State::replay(&events).docs[&one].folder, Some(work));

    let after = unhang(&mut events, 7, one);
    assert_eq!(after.docs[&one].page_of, None);
    assert_eq!(after.docs[&one].folder, Some(home));
    assert_eq!(
        after.docs[&one].order, was.order,
        "and in the place it held"
    );
    assert!(after.docs[&two].order > after.docs[&one].order);
}

#[test]
fn a_document_born_a_page_lands_beside_the_one_it_hung_from() {
    let mut events = Vec::new();
    let work = folder(&mut events, 1);
    let up = doc(&mut events, 2, Some(work));
    let beside = doc(&mut events, 3, Some(work));
    let page = Ulid::generate();
    events.push(ev(
        4,
        Op::DocAdd {
            id: page,
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: "dev_a-0004".into(),
                order: crate::order::first(),
                folder: None,
                page_of: Some(up),
            },
        },
    ));

    let after = unhang(&mut events, 5, page);
    assert_eq!(after.docs[&page].page_of, None);
    assert_eq!(after.docs[&page].folder, Some(work));
    assert!(
        after.docs[&page].order > after.docs[&beside].order,
        "at the end of the folder it lands in, not in front of it"
    );
}

#[test]
fn unhanging_into_a_folder_that_is_gone_lands_beside_what_it_hung_from() {
    let mut events = Vec::new();
    let home = folder(&mut events, 1);
    let work = folder(&mut events, 2);
    let one = doc(&mut events, 3, Some(home));
    let up = doc(&mut events, 4, Some(work));
    hang(&mut events, 5, one, up);
    events.push(ev(6, Op::FolderDelete { id: home }));

    let after = unhang(&mut events, 7, one);
    assert_eq!(after.docs[&one].page_of, None);
    assert_eq!(after.docs[&one].folder, Some(work));
}

#[test]
fn unhanging_something_that_is_not_a_page_leaves_the_folder_it_is_in() {
    let mut events = Vec::new();
    let home = folder(&mut events, 1);
    let one = doc(&mut events, 2, Some(home));

    let told = State::replay(&events);
    let d = unhung(&events, &told, one);
    assert_eq!(d.folder, Some(Some(home)), "not adrift in unfiled");

    events.push(ev(3, Op::DocMove { id: one, d }));
    let after = State::replay(&events);
    assert_eq!(after.docs[&one].folder, Some(home));
    assert_eq!(after.docs[&one].page_of, None);
}

#[test]
fn a_page_moved_straight_from_one_document_to_another_is_still_unhung() {
    let mut events = Vec::new();
    let home = folder(&mut events, 1);
    let one = doc(&mut events, 2, Some(home));
    let up = doc(&mut events, 3, Some(home));
    let other = doc(&mut events, 4, Some(home));

    hang(&mut events, 5, one, up);
    hang(&mut events, 6, one, other);
    assert_eq!(State::replay(&events).docs[&one].page_of, Some(other));

    let after = unhang(&mut events, 7, one);
    assert_eq!(
        after.docs[&one].page_of, None,
        "inverting the last hang would have re-hung it under the first"
    );
    assert_eq!(after.docs[&one].folder, Some(home));
}
