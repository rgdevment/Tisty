use ulid::Ulid;

use super::*;
use crate::event::{DocAdd, Said};

const HERE: &str = "dev_here";
const THERE: &str = "dev_there";
const AGENT_HERE: &str = "agent_here";
const AGENT_THERE: &str = "agent_there";
const AGENT_ADRIFT: &str = "agent_adrift";
const CLAUDE: Option<&str> = Some("claude-code");

fn at(minutes: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap()
}

fn device(named: &str) -> DeviceId {
    DeviceId(named.into())
}

fn body(n: usize) -> String {
    format!("# Minutes\n\nline {n}\n")
}

fn print(n: usize) -> String {
    crate::docs::print_of_text(&body(n))
}

fn said(id: DocId, minutes: i64, by: &str, via: Option<&str>, n: usize) -> Event {
    let mut event = Event::new(
        device(by),
        at(minutes),
        Op::DocSaid {
            id,
            d: Said::of(&body(n)),
        },
    );
    event.via = via.map(str::to_string);
    event
}

fn born(id: DocId, minutes: i64, by: &str, n: usize) -> Event {
    Event::new(
        device(by),
        at(minutes),
        Op::DocAdd {
            id,
            d: DocAdd {
                file: "dev_here-0001".into(),
                order: "a0".into(),
                said: Some(Said::of(&body(n))),
                ..Default::default()
            },
        },
    )
}

fn on_disk(n: usize, replaced: i64) -> Version {
    Version {
        print: print(n),
        replaced: at(replaced),
        bytes: body(n).len() as u64,
        spare: false,
    }
}

fn hosted() -> State {
    let mut state = State::default();
    for (agent, host) in [
        (AGENT_HERE, Some(HERE)),
        (AGENT_THERE, Some(THERE)),
        (AGENT_ADRIFT, None),
    ] {
        state.assistants.insert(device(agent));
        if let Some(host) = host {
            state.hosts.insert(device(agent), device(host));
        }
    }
    state
}

fn listed(kept: &[Version], log: &[Event], id: DocId, since: i64) -> Vec<Shown> {
    listed_holding(kept, log, id, since, None)
}

fn listed_holding(
    kept: &[Version],
    log: &[Event],
    id: DocId,
    since: i64,
    says: Option<&str>,
) -> Vec<Shown> {
    let state = hosted();
    shown(
        kept,
        &wrote(log, id),
        |one| machine(&state, one),
        &Here {
            machine: &device(HERE),
            says,
            since: at(since),
        },
    )
}

#[test]
fn a_line_that_tells_the_same_body_again_is_no_write() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 5, HERE, None, 1),
        said(Ulid::generate(), 6, HERE, None, 7),
        said(id, 10, AGENT_HERE, CLAUDE, 2),
        said(id, 11, THERE, None, 2),
    ];

    let told = wrote(&log, id);

    assert_eq!(
        told.iter()
            .map(|one| (one.at, one.print.clone()))
            .collect::<Vec<_>>(),
        vec![(at(0), print(1)), (at(10), print(2))]
    );
    assert_eq!(told[1].via.as_deref(), CLAUDE);
    assert_eq!(told[1].by, device(AGENT_HERE));
    assert_eq!(told[1].bytes, Some(body(2).len() as u64));
}

#[test]
fn what_this_machine_keeps_says_who_wrote_it_and_when() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, AGENT_HERE, CLAUDE, 2),
        said(id, 90, HERE, None, 3),
    ];

    let all = listed(&[on_disk(1, 40), on_disk(2, 90)], &log, id, 0);

    assert_eq!(all.len(), 2);
    assert_eq!((all[0].print.clone(), all[0].replaced), (print(2), at(90)));
    let wrote = all[0].wrote.as_ref().unwrap();
    assert_eq!((wrote.at, wrote.via.as_deref()), (at(40), CLAUDE));
    let wrote = all[1].wrote.as_ref().unwrap();
    assert_eq!((wrote.at, wrote.via.as_deref()), (at(0), None));
    assert!(all.iter().all(|one| one.elsewhere.is_none()));
}

#[test]
fn a_body_written_twice_is_dated_by_the_writing_before_it_was_replaced() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, HERE, None, 2),
        said(id, 80, HERE, None, 1),
        said(id, 120, HERE, None, 3),
    ];

    let early = listed(&[on_disk(1, 40)], &log, id, 0);
    let late = listed(&[on_disk(1, 120)], &log, id, 0);

    assert_eq!(early[0].wrote.as_ref().unwrap().at, at(0));
    assert_eq!(late[0].wrote.as_ref().unwrap().at, at(80));
}

#[test]
fn a_version_on_disk_the_log_never_told_is_still_shown() {
    let id = Ulid::generate();
    let log = [born(id, 0, HERE, 1), said(id, 5, HERE, None, 3)];

    let all = listed(&[on_disk(2, 3)], &log, id, 0);

    assert_eq!(all.len(), 1);
    assert_eq!(all[0].print, print(2));
    assert_eq!(all[0].wrote, None);
    assert_eq!(all[0].bytes, Some(body(2).len() as u64));
}

#[test]
fn the_step_back_alone_is_no_version_of_the_list() {
    let id = Ulid::generate();
    let log = [born(id, 0, HERE, 1), said(id, 5, HERE, None, 2)];
    let spare = Version {
        spare: true,
        ..on_disk(1, 5)
    };

    assert!(listed(&[spare], &log, id, 0).is_empty());
}

#[test]
fn a_version_another_machine_wrote_over_is_named_as_kept_there() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, THERE, None, 2),
        said(id, 41, AGENT_THERE, CLAUDE, 3),
        said(id, 42, HERE, None, 4),
    ];

    let all = listed(&[on_disk(3, 42)], &log, id, 0);

    assert_eq!(
        all.iter()
            .map(|one| (one.print.clone(), one.elsewhere.clone()))
            .collect::<Vec<_>>(),
        vec![
            (print(3), None),
            (print(2), Some(device(THERE))),
            (print(1), Some(device(THERE))),
        ]
    );
    let there = &all[1];
    assert_eq!(there.replaced, at(41));
    assert_eq!(there.wrote.as_ref().unwrap().at, at(40));
    assert_eq!(there.bytes, Some(body(2).len() as u64));
}

#[test]
fn a_log_from_before_the_history_names_nothing_this_machine_wrote_over() {
    let id = Ulid::generate();
    let mut log = vec![born(id, 0, HERE, 0)];
    for n in 1..2_000 {
        let (by, via) = match n % 3 {
            0 => (AGENT_HERE, CLAUDE),
            _ => (HERE, None),
        };
        log.push(said(id, n as i64 * 45, by, via, n));
    }

    assert!(listed(&[], &log, id, 0).is_empty());
}

#[test]
fn an_assistant_nobody_hosts_is_not_taken_for_another_machine() {
    let id = Ulid::generate();
    let log = [born(id, 0, HERE, 1), said(id, 40, AGENT_ADRIFT, CLAUDE, 2)];

    assert!(listed(&[], &log, id, 0).is_empty());
}

#[test]
fn what_another_machine_kept_before_the_limit_is_left_out() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, THERE, None, 2),
        said(id, 100, THERE, None, 3),
        said(id, 160, HERE, None, 4),
    ];

    let all = listed(&[], &log, id, 100);

    assert_eq!(all.len(), 1);
    assert_eq!(all[0].print, print(2));
    assert_eq!(all[0].replaced, at(100));
}

#[test]
fn what_the_document_says_now_is_never_a_version_kept_elsewhere() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, THERE, None, 2),
        said(id, 80, THERE, None, 1),
    ];

    let all = listed(&[], &log, id, 0);

    assert_eq!(
        all.iter().map(|one| one.print.clone()).collect::<Vec<_>>(),
        vec![print(2)]
    );
}

#[test]
fn the_body_still_on_this_disk_is_here_though_the_log_says_another_machine_wrote_on() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, THERE, None, 2),
        said(id, 80, THERE, CLAUDE, 3),
    ];

    let arriving = listed_holding(&[], &log, id, 0, Some(&print(1)));

    assert_eq!(
        arriving
            .iter()
            .map(|one| one.print.clone())
            .collect::<Vec<_>>(),
        vec![print(2)]
    );
}

#[test]
fn a_body_another_machine_replaced_twice_is_shown_once_at_its_last() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, HERE, 1),
        said(id, 40, THERE, None, 2),
        said(id, 80, THERE, None, 1),
        said(id, 120, THERE, None, 3),
    ];

    let all = listed(&[], &log, id, 0);

    assert_eq!(
        all.iter()
            .map(|one| (one.print.clone(), one.replaced))
            .collect::<Vec<_>>(),
        vec![(print(1), at(120)), (print(2), at(80))]
    );
}

#[test]
fn writes_inside_the_half_hour_by_one_hand_cut_nothing_elsewhere_either() {
    let id = Ulid::generate();
    let log = [
        born(id, 0, THERE, 1),
        said(id, 5, THERE, None, 2),
        said(id, 10, THERE, None, 3),
    ];

    assert!(listed(&[], &log, id, 0).is_empty());
}
