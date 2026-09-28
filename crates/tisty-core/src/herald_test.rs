use super::*;
use std::sync::Mutex;

struct Counting {
    named: &'static str,
    heard: Mutex<Vec<Happening>>,
    breaks: bool,
    deaf_to_carries: bool,
}

impl Counting {
    fn new(named: &'static str) -> Self {
        Self {
            named,
            heard: Mutex::new(Vec::new()),
            breaks: false,
            deaf_to_carries: false,
        }
    }
}

impl Channel for Counting {
    fn named(&self) -> &'static str {
        self.named
    }

    fn wants(&self, what: &Happening) -> bool {
        !(self.deaf_to_carries && matches!(what, Happening::Carried { .. }))
    }

    fn tell(&self, what: &Happening) -> Result<(), Trouble> {
        self.heard.lock().unwrap().push(what.clone());
        if self.breaks {
            return Err(Trouble {
                channel: self.named,
                why: "no".into(),
            });
        }
        Ok(())
    }
}

fn filed(title: &str) -> Happening {
    Happening::Filed {
        title: title.into(),
    }
}

fn at(when: &str) -> crate::DateSpec {
    crate::DateSpec::floating(when.parse().unwrap(), "America/Santiago")
}

fn moment(when: &str) -> jiff::Timestamp {
    when.parse::<jiff::civil::DateTime>()
        .unwrap()
        .to_zoned(zone())
        .unwrap()
        .timestamp()
}

fn zone() -> jiff::tz::TimeZone {
    jiff::tz::TimeZone::get("America/Santiago").unwrap()
}

fn fired(op: crate::Op) -> crate::Event {
    crate::Event::new(crate::DeviceId("a".into()), jiff::Timestamp::UNIX_EPOCH, op)
}

fn with(reminders: Vec<crate::DateSpec>) -> crate::State {
    made(reminders, None)
}

fn every_day(reminders: Vec<crate::DateSpec>) -> crate::State {
    made(
        reminders,
        Some(crate::model::Repeat::done(crate::model::Cadence {
            every: 1,
            unit: crate::model::Unit::Day,
        })),
    )
}

fn made(reminders: Vec<crate::DateSpec>, repeat: Option<crate::model::Repeat>) -> crate::State {
    let mut state = crate::State::default();
    let id = ulid::Ulid::generate();
    let mut add = crate::event::TaskAdd::new("tomar la pastilla".to_string(), "a0".to_string());
    add.reminders = reminders;
    add.repeat = repeat;
    state.apply(&fired(crate::Op::TaskAdd { id, d: add }));
    state
}

#[test]
fn a_reminder_that_fell_inside_the_window_is_owed() {
    let state = with(vec![at("2026-08-11T09:45:00")]);

    let owed = owed(
        &state,
        moment("2026-08-11T09:44:00"),
        moment("2026-08-11T09:46:00"),
        &zone(),
    );

    assert_eq!(owed.len(), 1);
    assert_eq!(owed[0].what.title(), Some("tomar la pastilla"));
}

#[test]
fn one_still_ahead_is_not_owed_yet() {
    let state = with(vec![at("2026-08-11T10:00:00")]);

    assert!(
        owed(
            &state,
            moment("2026-08-11T09:44:00"),
            moment("2026-08-11T09:46:00"),
            &zone(),
        )
        .is_empty()
    );
}

#[test]
fn one_already_told_is_not_told_twice() {
    let state = with(vec![at("2026-08-11T09:45:00")]);

    assert!(
        owed(
            &state,
            moment("2026-08-11T09:46:00"),
            moment("2026-08-11T09:50:00"),
            &zone(),
        )
        .is_empty()
    );
}

#[test]
fn nothing_older_than_the_lookback_is_told() {
    let state = with(vec![at("2026-07-11T09:45:00")]);

    assert!(
        owed(
            &state,
            moment("2026-07-01T00:00:00"),
            moment("2026-08-11T09:46:00"),
            &zone(),
        )
        .is_empty()
    );
}

#[test]
fn a_closed_task_says_nothing() {
    let mut state = with(vec![at("2026-08-11T09:45:00")]);
    let id = *state.tasks.keys().next().unwrap();
    state.apply(&fired(crate::Op::TaskDone { id, filled: false }));

    assert!(
        owed(
            &state,
            moment("2026-08-11T09:44:00"),
            moment("2026-08-11T09:46:00"),
            &zone(),
        )
        .is_empty()
    );
}

#[test]
fn several_come_out_oldest_first() {
    let state = with(vec![at("2026-08-11T09:45:00"), at("2026-08-11T09:15:00")]);

    let owed = owed(
        &state,
        moment("2026-08-11T09:00:00"),
        moment("2026-08-11T09:46:00"),
        &zone(),
    );

    assert_eq!(owed.len(), 2);
    assert!(owed[0].at < owed[1].at);
}

#[test]
fn everyone_registered_hears_it() {
    let heralds = Heralds::default()
        .with(Box::new(Counting::new("screen")))
        .with(Box::new(Counting::new("sound")));

    assert!(!heralds.tell(&filed("comprar pan")).lost());
    assert_eq!(heralds.names(), vec!["screen", "sound"]);
}

#[test]
fn one_that_fails_does_not_silence_the_others() {
    let mut broken = Counting::new("mail");
    broken.breaks = true;
    let heralds = Heralds::default()
        .with(Box::new(broken))
        .with(Box::new(Counting::new("screen")));

    let told = heralds.tell(&filed("comprar pan"));

    assert_eq!(told.trouble.len(), 1);
    assert_eq!(told.trouble[0].channel, "mail");
    assert!(!told.lost(), "the screen did hear it");
}

#[test]
fn a_channel_can_decline_what_does_not_concern_it() {
    let mut picky = Counting::new("phone");
    picky.deaf_to_carries = true;
    let heralds = Heralds::default().with(Box::new(picky));

    assert!(!heralds.tell(&Happening::Carried { brought: 2 }).lost());
}

#[test]
fn a_happening_no_channel_could_deliver_is_lost() {
    let mut broken = Counting::new("screen");
    broken.breaks = true;
    let heralds = Heralds::default().with(Box::new(broken));

    assert!(heralds.tell(&filed("comprar pan")).lost());
}

#[test]
fn nobody_wanting_it_is_not_the_same_as_losing_it() {
    let mut picky = Counting::new("phone");
    picky.deaf_to_carries = true;
    let heralds = Heralds::default().with(Box::new(picky));

    assert!(!heralds.tell(&Happening::Carried { brought: 2 }).lost());
}

#[test]
fn nobody_listening_is_not_an_error() {
    assert!(!Heralds::default().tell(&filed("comprar pan")).lost());
}

#[test]
fn a_habit_skipped_yesterday_still_rings_today() {
    let state = every_day(vec![at("2026-08-11T09:00:00")]);

    let told = owed(
        &state,
        moment("2026-08-12T08:59:00"),
        moment("2026-08-12T09:01:00"),
        &zone(),
    );

    assert_eq!(told.len(), 1, "the day after should ring: {told:?}");
}

#[test]
fn a_habit_left_for_a_week_rings_once_today_and_not_seven_times() {
    let state = every_day(vec![at("2026-08-05T09:00:00")]);

    let told = owed(
        &state,
        moment("2026-08-12T08:59:00"),
        moment("2026-08-12T09:01:00"),
        &zone(),
    );

    assert_eq!(told.len(), 1, "{told:?}");
}

#[test]
fn a_habit_says_nothing_at_an_hour_that_is_not_its_own() {
    let state = every_day(vec![at("2026-08-11T09:00:00")]);

    assert!(
        owed(
            &state,
            moment("2026-08-12T14:00:00"),
            moment("2026-08-12T15:00:00"),
            &zone(),
        )
        .is_empty()
    );
}

#[test]
fn a_plain_reminder_still_rings_only_the_once() {
    let state = with(vec![at("2026-08-11T09:00:00")]);

    assert!(
        owed(
            &state,
            moment("2026-08-12T08:59:00"),
            moment("2026-08-12T09:01:00"),
            &zone(),
        )
        .is_empty()
    );
}
