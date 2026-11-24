use std::collections::BTreeSet;

use super::{WakeOffer, marked, reminded, unasked, wake_offer};
use crate::waking::Waking;
use crate::wiring::{Manual, Seen, hooks::Hooks};

fn standing(offered: bool, wakes: bool) -> Waking {
    Waking {
        offered,
        wakes,
        theirs: false,
    }
}

#[test]
fn the_first_reminder_on_a_copy_that_does_not_start_with_the_machine_asks() {
    assert_eq!(
        wake_offer(false, true, || standing(true, false)),
        WakeOffer::Ask(standing(true, false))
    );
}

#[test]
fn with_no_reminder_yet_nothing_is_asked_and_the_machine_is_not_read() {
    assert_eq!(
        wake_offer(false, false, || unreachable!("nothing to ask about")),
        WakeOffer::Not
    );
}

#[test]
fn once_asked_it_is_never_asked_again() {
    assert_eq!(
        wake_offer(true, true, || unreachable!("already asked")),
        WakeOffer::Not
    );
}

#[test]
fn a_copy_that_already_starts_with_the_machine_settles_the_question() {
    assert_eq!(
        wake_offer(false, true, || standing(true, true)),
        WakeOffer::Settled
    );
}

#[test]
fn a_startup_windows_holds_is_not_offered_from_the_window() {
    let held = Waking {
        offered: true,
        wakes: false,
        theirs: true,
    };
    assert_eq!(wake_offer(false, true, || held), WakeOffer::Not);
}

#[test]
fn a_copy_that_cannot_start_with_the_machine_is_not_asked() {
    assert_eq!(
        wake_offer(false, true, || standing(false, false)),
        WakeOffer::Not
    );
}

#[test]
fn only_an_open_task_with_a_reminder_counts() {
    let mut state = tisty_core::State::default();
    let id = ulid::Ulid::generate();
    let mut add = tisty_core::event::TaskAdd::new("comprar pan", "a0");
    add.reminders = vec![tisty_core::DateSpec::floating(
        "2026-08-12T09:00:00".parse().unwrap(),
        "America/Santiago",
    )];
    let fired = |op| {
        tisty_core::Event::new(
            tisty_core::DeviceId("a".into()),
            jiff::Timestamp::UNIX_EPOCH,
            op,
        )
    };
    assert!(!reminded(&state));

    state.apply(&fired(tisty_core::Op::TaskAdd { id, d: add }));
    assert!(reminded(&state));

    state.apply(&fired(tisty_core::Op::TaskDone { id, filled: false }));
    assert!(!reminded(&state));
}

fn row(id: &str, place: Option<&str>, hooked: bool) -> Seen {
    Seen {
        id: id.into(),
        name: id.into(),
        also: None,
        place: place.map(str::to_string),
        at: String::new(),
        wired: true,
        astray: false,
        gone: false,
        apart: false,
        unread: false,
        installed: true,
        points: None,
        maybe: false,
        by_hand: false,
        manual: Manual {
            file: String::new(),
            snippet: String::new(),
            cli: None,
        },
        hooks: hooked.then(|| Hooks {
            on: false,
            at: "~/.claude/settings.json".into(),
            ..Hooks::default()
        }),
    }
}

fn plugged_rows(seen: Vec<Seen>, plugged: &[&str]) -> Vec<Option<bool>> {
    marked(seen, &plugged.iter().copied().collect::<BTreeSet<_>>())
        .into_iter()
        .map(|one| one.hooks.map(|hooks| hooks.plugged))
        .collect()
}

#[test]
fn claude_codes_hooks_row_says_the_plugin_brings_them_on_this_computer_only() {
    let seen = vec![
        row("claude-code", None, true),
        row("claude-code", Some("Ubuntu"), true),
        row("cursor", None, true),
        row("codex", None, false),
    ];
    assert_eq!(
        plugged_rows(seen, &["claude"]),
        vec![Some(true), Some(false), Some(false), None]
    );
}

#[test]
fn a_look_with_no_click_never_says_whether_the_plugin_brings_them() {
    let seen = unasked(vec![
        row("claude-code", None, true),
        row("cursor", None, true),
        row("codex", None, false),
    ]);
    let unknown: Vec<_> = seen
        .iter()
        .map(|one| one.hooks.as_ref().map(|it| it.plugin_unknown))
        .collect();
    assert_eq!(unknown, vec![Some(true), Some(false), None]);
    assert!(
        seen.iter()
            .all(|one| one.hooks.as_ref().is_none_or(|it| !it.plugged))
    );
}

#[test]
fn without_the_plugin_no_hooks_row_says_it() {
    let seen = vec![row("claude-code", None, true), row("cursor", None, true)];
    assert_eq!(plugged_rows(seen, &[]), vec![Some(false), Some(false)]);
}

#[test]
fn the_plugins_records_are_read_only_after_this_computers_yes() {
    let home = tempfile::tempdir().unwrap();
    let installed = home.path().join(".claude/plugins/installed_plugins.json");
    std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
    std::fs::write(
        &installed,
        r#"{"plugins":{"tisty@rgdevment":[{"scope":"user"}]}}"#,
    )
    .unwrap();
    let mut ledger = tisty_core::agents::Ledger::fresh("machine-a");
    let refused = super::plugged_with(&ledger, || unreachable!("never looked before the yes"));
    assert!(refused.is_empty());

    ledger.consent(true);
    let homes = || Some(crate::wiring::agents::Homes::with(home.path(), &[]));
    assert_eq!(
        super::plugged_with(&ledger, homes),
        BTreeSet::from(["claude"])
    );

    ledger.consent(false);
    ledger.take_tisty_skills(true);
    assert_eq!(
        super::plugged_with(&ledger, homes),
        BTreeSet::from(["claude"]),
        "the skills' own yes reads it too"
    );
}
