use std::path::{Path, PathBuf};

use super::super::{Writer, empty};
use super::*;
use crate::event::{DocAdd, Said};

const DOC: &str = "mac0-0001";
const HERE: &str = "here";
const THERE: &str = "there";
const CLAUDE: Option<&str> = Some("Claude Code");

fn at(minutes: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap()
}

fn desk() -> (tempfile::TempDir, PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    (room, docs)
}

fn print(body: &str) -> String {
    crate::docs::print_of_text(body)
}

fn noted(id: DocId, device: &str, minutes: i64, via: Option<&str>, body: &str) -> Event {
    let said = Op::DocSaid {
        id,
        d: Said::of(body),
    };
    Event {
        via: via.map(str::to_string),
        ..Event::new(DeviceId(device.into()), at(minutes), said)
    }
}

fn made(id: DocId, device: &str, minutes: i64, via: Option<&str>, body: &str) -> Event {
    let add = Op::DocAdd {
        id,
        d: DocAdd {
            file: DOC.into(),
            order: "a".into(),
            said: Some(Said::of(body)),
            ..DocAdd::default()
        },
    };
    Event {
        via: via.map(str::to_string),
        ..Event::new(DeviceId(device.into()), at(minutes), add)
    }
}

fn written(docs: &Path, data: &Path, minutes: i64, via: Option<&str>, body: &str) {
    let by = Writer::at(data, via.map(str::to_string), at(minutes));
    match crate::docs::resolve(docs, DOC).is_ok_and(|one| one.is_file()) {
        true => crate::docs::write_by(docs, &by, DOC, body).unwrap(),
        false => {
            crate::docs::create_named_by(docs, &by, DOC, body).unwrap();
        }
    }
}

fn elsewhere(device: &DeviceId) -> bool {
    device.0 != HERE
}

fn shown(data: &Path, events: &[Event], id: DocId, minutes: i64) -> Vec<Row> {
    rows(data, DOC, &logged(events, id), 30, at(minutes), elsewhere)
}

fn prints(rows: &[Row]) -> Vec<String> {
    rows.iter().map(|one| one.print.clone()).collect()
}

#[test]
fn the_cut_replayed_over_the_notes_a_write_leaves_names_the_versions_on_disk() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    let told: [(i64, Option<&str>, &str); 7] = [
        (0, None, "# A\n"),
        (5, None, "# B\n"),
        (10, CLAUDE, "# C\n"),
        (12, CLAUDE, "# D\n"),
        (50, CLAUDE, "# E\n"),
        (55, None, "# F\n"),
        (56, None, "# G\n"),
    ];
    let mut events = Vec::new();
    for (n, (minutes, via, body)) in told.iter().enumerate() {
        written(&docs, data, *minutes, *via, body);
        events.push(match n {
            0 => made(id, HERE, *minutes, *via, body),
            _ => noted(id, HERE, *minutes, *via, body),
        });
    }

    let cut: Vec<String> = logged(&events, id)
        .into_iter()
        .filter(|one| one.cut)
        .map(|one| one.print)
        .collect();
    assert_eq!(cut, vec![print("# B\n"), print("# D\n"), print("# E\n")]);

    let listed = shown(data, &events, id, 60);
    assert_eq!(
        prints(&listed),
        vec![print("# E\n"), print("# D\n"), print("# B\n")],
        "newest first, and the same bodies the disk holds"
    );
    assert!(listed.iter().all(|one| one.here));
    let who: Vec<Option<&str>> = listed.iter().map(|one| one.via.as_deref()).collect();
    assert_eq!(who, vec![CLAUDE, CLAUDE, None]);
    let wrote: Vec<Option<jiff::Timestamp>> = listed.iter().map(|one| one.wrote).collect();
    assert_eq!(wrote, vec![Some(at(50)), Some(at(12)), Some(at(5))]);
    assert_eq!(listed[2].bytes, "# B\n".len() as u64);
}

#[test]
fn the_first_body_comes_in_the_note_that_made_the_document() {
    let id = DocId::generate();
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, HERE, 40, None, "# B\n"),
    ];

    let all = logged(&events, id);

    assert_eq!(all.len(), 2);
    assert_eq!(all[0].print, print("# A\n"));
    assert!(all[0].cut, "forty quiet minutes cut the first body");
    assert_eq!(all[0].replaced, Some(at(40)));
    assert_eq!(all[1].replaced, None);
}

#[test]
fn a_note_that_reads_the_same_body_again_cuts_nothing() {
    let id = DocId::generate();
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, THERE, 1, CLAUDE, "# A\n"),
        noted(id, HERE, 2, None, "# B\n"),
    ];

    let all = logged(&events, id);

    assert_eq!(all.len(), 2, "the body read again is not a second body");
    assert!(
        !all[0].cut,
        "another machine reading it was taken for another writer"
    );
    assert_eq!(all[0].device, DeviceId(HERE.into()));
}

#[test]
fn notes_about_another_document_are_not_this_ones_history() {
    let id = DocId::generate();
    let other = DocId::generate();
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(other, HERE, 40, None, "# Z\n"),
    ];

    assert_eq!(logged(&events, id).len(), 1);
}

#[test]
fn a_version_another_machine_wrote_is_listed_without_its_text() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    written(&docs, data, 0, None, "# A\n");
    written(&docs, data, 90, CLAUDE, "# D\n");
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, THERE, 40, None, "# B\n"),
        noted(id, THERE, 45, CLAUDE, "# C\n"),
        noted(id, THERE, 90, CLAUDE, "# D\n"),
    ];

    let listed = shown(data, &events, id, 100);

    assert_eq!(
        prints(&listed),
        vec![print("# C\n"), print("# B\n"), print("# A\n")]
    );
    let here: Vec<bool> = listed.iter().map(|one| one.here).collect();
    assert_eq!(here, vec![false, false, true]);
    assert_eq!(listed[0].device, Some(DeviceId(THERE.into())));
    assert_eq!(listed[0].bytes, "# C\n".len() as u64);
}

#[test]
fn a_machine_whose_log_is_older_than_its_history_lists_nothing_it_cannot_open() {
    let (room, _docs) = desk();
    let id = DocId::generate();
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, HERE, 40, None, "# B\n"),
        noted(id, HERE, 90, CLAUDE, "# C\n"),
    ];

    assert!(
        shown(room.path(), &events, id, 100).is_empty(),
        "what this machine wrote before it kept versions was listed as kept elsewhere"
    );
}

#[test]
fn a_history_emptied_here_still_names_what_another_machine_wrote() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    written(&docs, data, 0, None, "# A\n");
    written(&docs, data, 40, None, "# B\n");
    written(&docs, data, 200, None, "# D\n");
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, HERE, 40, None, "# B\n"),
        noted(id, THERE, 100, None, "# C\n"),
        noted(id, HERE, 200, None, "# D\n"),
    ];
    assert_eq!(shown(data, &events, id, 210).len(), 3);

    assert_eq!(empty(data, DOC).unwrap(), 2);

    let listed = shown(data, &events, id, 210);
    assert_eq!(prints(&listed), vec![print("# C\n")]);
    assert!(!listed[0].here);
}

#[test]
fn a_version_kept_elsewhere_past_the_limit_is_left_out() {
    let (room, _docs) = desk();
    let id = DocId::generate();
    let day = 24 * 60;
    let events = vec![
        made(id, THERE, 0, None, "# A\n"),
        noted(id, THERE, day, None, "# B\n"),
        noted(id, THERE, 9 * day, None, "# C\n"),
    ];
    let said = logged(&events, id);
    let listed = |days| rows(room.path(), DOC, &said, days, at(10 * day), elsewhere);

    assert_eq!(prints(&listed(30)), vec![print("# B\n"), print("# A\n")]);
    assert_eq!(prints(&listed(5)), vec![print("# B\n")]);
    assert!(
        prints(&listed(1)).len() == 1,
        "replaced a day ago is still inside one day"
    );
}

#[test]
fn a_version_on_disk_the_log_never_named_is_listed_with_no_writer() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    written(&docs, data, 0, None, "# A\n");
    written(&docs, data, 40, None, "# B\n");
    let events = vec![noted(id, HERE, 40, None, "# B\n")];

    let listed = shown(data, &events, id, 50);

    assert_eq!(prints(&listed), vec![print("# A\n")]);
    assert_eq!(listed[0].wrote, None);
    assert_eq!(listed[0].device, None);
    assert_eq!(listed[0].replaced, at(40));
    assert!(listed[0].here);
}

#[test]
fn the_step_back_the_cut_did_not_keep_is_no_row() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    written(&docs, data, 0, None, "# A\n");
    let by = Writer::at(data, None, at(1));
    let now = crate::docs::print_of_text("# A\n");
    crate::docs::rewrite(&docs, &by, DOC, "# B\n", &now).unwrap();
    let events = vec![
        made(id, HERE, 0, None, "# A\n"),
        noted(id, HERE, 1, None, "# B\n"),
    ];

    assert_eq!(super::super::versions(data, DOC).len(), 1);
    assert!(shown(data, &events, id, 5).is_empty());
}

#[test]
fn a_body_written_twice_is_one_row_dated_by_its_last_replacing() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    let told: [(i64, &str); 4] = [(0, "# A\n"), (40, "# B\n"), (80, "# A\n"), (120, "# C\n")];
    let mut events = Vec::new();
    for (n, (minutes, body)) in told.iter().enumerate() {
        written(&docs, data, *minutes, None, body);
        events.push(match n {
            0 => made(id, HERE, *minutes, None, body),
            _ => noted(id, HERE, *minutes, None, body),
        });
    }

    let listed = shown(data, &events, id, 130);

    assert_eq!(prints(&listed), vec![print("# A\n"), print("# B\n")]);
    assert_eq!(listed[0].wrote, Some(at(80)));
    assert_eq!(listed[0].replaced, at(120));
}

#[test]
fn a_version_that_says_what_the_document_says_now_is_not_listed() {
    let (room, docs) = desk();
    let data = room.path();
    let id = DocId::generate();
    let told: [(i64, &str); 3] = [(0, "# A\n"), (40, "# B\n"), (80, "# A\n")];
    let mut events = Vec::new();
    for (n, (minutes, body)) in told.iter().enumerate() {
        written(&docs, data, *minutes, None, body);
        events.push(match n {
            0 => made(id, HERE, *minutes, None, body),
            _ => noted(id, HERE, *minutes, None, body),
        });
    }
    assert_eq!(super::super::versions(data, DOC).len(), 2);

    let listed = shown(data, &events, id, 90);

    assert_eq!(prints(&listed), vec![print("# B\n")]);
}
