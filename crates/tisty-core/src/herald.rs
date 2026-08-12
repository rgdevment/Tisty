//! Telling someone that something happened, without deciding how.

use serde::{Deserialize, Serialize};

/// Something worth telling someone about, named for what happened rather than
/// for how it would be told.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "what")]
pub enum Happening {
    Filed {
        title: String,
    },
    Due {
        title: String,
        task: String,
    },
    /// A lid closed at ten and opened at eight owes a dozen at once. One line
    /// beats a dozen notifications and a dozen overlapping tones.
    Missed {
        count: usize,
    },
    Carried {
        brought: usize,
    },
}

impl Happening {
    pub fn title(&self) -> Option<&str> {
        match self {
            Happening::Filed { title } | Happening::Due { title, .. } => Some(title),
            Happening::Missed { .. } | Happening::Carried { .. } => None,
        }
    }
}

/// Reminders that came due in `(since, now]`, oldest first.
///
/// A machine that was off for a month must not wake up to two hundred popups,
/// so nothing older than `LOOKBACK` is told — the task is still sitting overdue
/// in the list, which is the honest place for it.
pub fn owed(
    state: &crate::State,
    since: jiff::Timestamp,
    now: jiff::Timestamp,
    here: &jiff::tz::TimeZone,
) -> Vec<Due> {
    let from = since.max(now - LOOKBACK);
    let mut owed: Vec<Due> = state
        .tasks
        .values()
        .filter(|task| task.status == crate::model::Status::Open)
        .flat_map(|task| {
            task.reminders
                .iter()
                .flat_map(|one| rings(task, one, here, from, now))
                .map(|at| Due {
                    at,
                    what: Happening::Due {
                        title: task.title.clone(),
                        task: task.id.to_string(),
                    },
                })
        })
        .collect();
    owed.sort_by_key(|one| one.at);
    owed
}

/// When a task repeats, its reminder is a rule and not a date. The successor is
/// only born on completion, so a habit skipped once would go quiet for ever —
/// which is the day it is needed most. It rings while the task stays open, and
/// completing or dropping it is what stops the ringing.
fn rings(
    task: &crate::model::Task,
    one: &crate::DateSpec,
    here: &jiff::tz::TimeZone,
    from: jiff::Timestamp,
    now: jiff::Timestamp,
) -> Vec<jiff::Timestamp> {
    let Ok(first) = one.instant(here) else {
        return Vec::new();
    };
    let Some(repeat) = task.repeat else {
        return match first > from && first <= now {
            true => vec![first],
            false => Vec::new(),
        };
    };

    let cadence = repeat.cadence();
    let mut said = Vec::new();
    let mut at = one.at;
    // A reminder abandoned years ago would otherwise be walked one day at a
    // time on every tick. Past the cap it stops: the task is still sitting
    // overdue in the list, which is the honest place for it.
    for _ in 0..STEPS {
        let Ok(stamp) = one.moved(at).instant(here) else {
            break;
        };
        if stamp > now {
            break;
        }
        if stamp > from {
            said.push(stamp);
        }
        let Some(next) = cadence.after(at) else {
            break;
        };
        at = next;
    }
    said
}

const STEPS: usize = 4_000;

const LOOKBACK: jiff::SignedDuration = jiff::SignedDuration::from_hours(12);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Due {
    pub at: jiff::Timestamp,
    pub what: Happening,
}

/// How many owed reminders are worth one notification each.
pub const ONE_BY_ONE: usize = 3;

/// Told one by one while there are few, gathered into a single line when a
/// suspended machine wakes up owing a night's worth.
pub fn gathered(owed: Vec<Due>) -> Vec<Happening> {
    if owed.len() <= ONE_BY_ONE {
        return owed.into_iter().map(|one| one.what).collect();
    }
    vec![Happening::Missed { count: owed.len() }]
}

/// A way of telling. Register one and it hears every happening it wants.
pub trait Channel: Send + Sync {
    fn named(&self) -> &'static str;

    fn wants(&self, _what: &Happening) -> bool {
        true
    }

    fn tell(&self, what: &Happening) -> Result<(), Trouble>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trouble {
    pub channel: &'static str,
    pub why: String,
}

#[derive(Default)]
pub struct Heralds {
    channels: Vec<Box<dyn Channel>>,
}

impl Heralds {
    pub fn with(mut self, channel: Box<dyn Channel>) -> Self {
        self.channels.push(channel);
        self
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.channels.iter().map(|one| one.named()).collect()
    }

    /// One channel failing must not silence the rest.
    pub fn tell(&self, what: &Happening) -> Told {
        let mut told = Told::default();
        for one in self.channels.iter().filter(|one| one.wants(what)) {
            told.asked += 1;
            match one.tell(what) {
                Ok(()) => told.heard += 1,
                Err(why) => told.trouble.push(why),
            }
        }
        told
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Told {
    /// Channels that wanted it. Zero is an answer, not a failure.
    pub asked: usize,
    pub heard: usize,
    pub trouble: Vec<Trouble>,
}

impl Told {
    /// Nobody wanting to hear it is fine. Everybody wanting to and failing is
    /// not: that is a reminder nobody will ever see, and it must not be
    /// written off as said.
    pub fn lost(&self) -> bool {
        self.asked > 0 && self.heard == 0
    }
}

#[cfg(test)]
mod tests {
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
            Some(crate::model::Repeat::Done(crate::model::Cadence {
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

    /// Otherwise every tick of the watcher tells you the same thing again.
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

    /// A machine off for a month must not wake up to two hundred popups.
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
        state.apply(&fired(crate::Op::TaskDone { id }));

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

    /// A reminder nobody could deliver must not be written off as said.
    #[test]
    fn a_happening_no_channel_could_deliver_is_lost() {
        let mut broken = Counting::new("screen");
        broken.breaks = true;
        let heralds = Heralds::default().with(Box::new(broken));

        assert!(heralds.tell(&filed("comprar pan")).lost());
    }

    /// Otherwise every quiet happening would be retried for ever.
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

    /// The reason the whole thing changed: a daily medicine whose successor was
    /// never born, because the day it was skipped is the day it matters.
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

    /// The cadence still decides: a reminder that does not fall on today rings
    /// on none of the days in between.
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

    /// Without a cadence nothing changes: one reminder, one ring, once.
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
}
