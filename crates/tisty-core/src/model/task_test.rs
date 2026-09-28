use super::*;

fn task() -> Task {
    Task::new(Ulid::generate(), "ship it", "a0")
}

fn entry(body: &str) -> LogEntry {
    LogEntry {
        id: Ulid::generate(),
        at: Timestamp::UNIX_EPOCH,
        tz: None,
        body: body.into(),
        by: None,
        via: None,
    }
}

#[test]
fn what_will_not_happen_sorts_last_of_all() {
    let mut all = vec![
        Priority::Minor,
        Priority::Unset,
        Priority::Do,
        Priority::Delegate,
        Priority::Decide,
    ];
    all.sort();
    assert_eq!(
        all,
        [
            Priority::Do,
            Priority::Decide,
            Priority::Delegate,
            Priority::Unset,
            Priority::Minor
        ]
    );
}

#[test]
fn priority_serialises_as_the_word_it_is() {
    assert_eq!(serde_json::to_string(&Priority::Do).unwrap(), "\"do\"");
    assert_eq!(
        serde_json::from_str::<Priority>("\"delegate\"").unwrap(),
        Priority::Delegate
    );
}

#[test]
fn the_old_levels_come_back_unclassified() {
    for level in ["1", "2", "3", "4"] {
        assert_eq!(
            serde_json::from_str::<Priority>(level).unwrap(),
            Priority::Unset
        );
    }
}

#[test]
fn what_the_first_matrix_wrote_is_still_read() {
    assert_eq!(
        serde_json::from_str::<Priority>("\"wont\"").unwrap(),
        Priority::Minor
    );
    assert_eq!(
        serde_json::to_string(&Priority::Minor).unwrap(),
        "\"minor\""
    );
}

#[test]
fn a_priority_nobody_defined_is_rejected() {
    assert!(serde_json::from_str::<Priority>("0").is_err());
    assert!(serde_json::from_str::<Priority>("5").is_err());
    assert!(serde_json::from_str::<Priority>("\"urgent\"").is_err());
}

#[test]
fn a_bare_task_serialises_to_the_minimum() {
    let json = serde_json::to_string(&task()).unwrap();
    for absent in [
        "description",
        "log",
        "steps",
        "date",
        "deadline",
        "list",
        "tags",
        "reminders",
        "completed_at",
        "read_as",
        "open_to_agents",
        "created_via",
    ] {
        assert!(
            !json.contains(absent),
            "'{absent}' should not appear in {json}"
        );
    }
}

#[test]
fn round_trips() {
    let mut t = task();
    t.tags = vec![Tag::new("work").unwrap()];
    t.description = Some("check the gateway".into());
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(t, serde_json::from_str::<Task>(&json).unwrap());
}

#[test]
fn created_at_comes_from_the_id() {
    let t = task();
    assert_eq!(t.created_at().as_millisecond(), t.id.timestamp_ms() as i64);
}

#[test]
fn weight_separates_the_trivial_from_the_documented() {
    let mut trivial = task();
    trivial.retally();

    let mut rich = task();
    rich.description = Some("el redirect de registration se cae en Brasil".into());
    rich.log
        .push(entry("el proxy lateral no arrancaba con la config nueva"));
    rich.retally();

    assert_eq!(trivial.weight(), 0);
    assert!(rich.weight() > trivial.weight());
}

fn worded(many: usize) -> String {
    vec!["x"; many].join(" ")
}

fn stepped(many: usize) -> Vec<Step> {
    (0..many)
        .map(|n| Step {
            id: Ulid::generate(),
            text: format!("paso {n}"),
            done: false,
            order: format!("a{n}"),
        })
        .collect()
}

#[test]
fn a_log_is_weighed_by_what_it_says_and_a_telegram_says_nothing() {
    for (words, want) in [(7, 0), (8, 1), (29, 1), (30, 2), (99, 2), (100, 3)] {
        let mut one = task();
        one.description = Some(worded(words));
        one.retally();

        assert_eq!(
            one.volume.prose, want,
            "{words} words weighed {} instead of {want}",
            one.volume.prose
        );
    }
}

#[test]
fn a_plan_climbs_at_three_steps_and_again_at_eight() {
    for (steps, want) in [(2, 0), (3, 1), (7, 1), (8, 2)] {
        let mut one = task();
        one.steps = stepped(steps);
        one.retally();

        assert_eq!(
            one.weight(),
            want,
            "{steps} steps weighed {} instead of {want}",
            one.weight()
        );
    }
}

#[test]
fn a_reference_climbs_at_one_and_again_at_three() {
    for (many, want) in [(0, 0), (1, 1), (2, 1), (3, 2)] {
        let mut one = task();
        let links: Vec<String> = (0..many)
            .map(|n| format!("[uno](https://x.example/{n})"))
            .collect();
        one.description = Some(links.join(" "));
        one.retally();

        let prose = one.volume.prose;
        assert_eq!(
            one.weight() - prose,
            want,
            "{many} references weighed {} instead of {want}",
            one.weight() - prose
        );
    }
}

#[test]
fn the_weight_is_what_the_three_of_them_come_to_together() {
    let mut one = task();
    one.description = Some(format!("{} [uno](https://x.example/1)", worded(30)));
    one.steps = stepped(8);
    one.retally();

    assert_eq!(one.volume.prose, 2);
    assert_eq!(
        one.weight(),
        5,
        "two of prose, two of plan and one reference"
    );
}

#[test]
fn the_agenda_never_outweighs_the_history() {
    let mut agenda = task();
    agenda.date = Some(DateSpec::all_day("2026-08-05".parse().unwrap(), "UTC"));
    agenda.deadline = Some(DateSpec::all_day("2026-08-09".parse().unwrap(), "UTC"));
    agenda.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];
    agenda.list = Some(Ulid::generate());
    agenda.reminders = vec![DateSpec::all_day("2026-08-04".parse().unwrap(), "UTC")];
    agenda.retally();

    let mut history = task();
    history.log.push(entry(
        "el gateway no propagaba la cabecera de idioma, asi que el backend respondia \
             siempre en ingles aunque el navegador pidiera otra cosa",
    ));
    history.retally();

    assert_eq!(agenda.weight(), 0);
    assert!(history.weight() > agenda.weight());
}

#[test]
fn weight_counts_substance_and_not_entries() {
    let mut noisy = task();
    for _ in 0..12 {
        noisy.log.push(entry("ok"));
    }
    noisy.retally();

    let mut written = task();
    written.log.push(entry(
        "el proxy lateral no arrancaba con la configuración nueva",
    ));
    written.retally();

    assert_eq!(noisy.weight(), 0);
    assert!(written.weight() > noisy.weight());
}

#[test]
fn weight_has_a_ceiling_so_a_disaster_cannot_top_everything() {
    let mut endless = task();
    for _ in 0..40 {
        endless
            .log
            .push(entry("volvió a fallar el despliegue del proxy lateral"));
    }
    endless.retally();
    assert_eq!(endless.weight(), 8);
}

#[test]
fn references_are_gathered_across_the_description_and_the_journal() {
    let mut t = task();
    t.description = Some("sale del ticket [[CUSLEG-3465]]".into());
    t.log.push(entry("el MR está en https://gl.example/mr/7"));
    t.log.push(entry("sigue siendo [[CUSLEG-3465]]"));
    t.retally();

    assert_eq!(
        t.references()
            .into_iter()
            .map(|one| one.target)
            .collect::<Vec<_>>(),
        ["CUSLEG-3465", "https://gl.example/mr/7"],
        "the same target twice is one reference, and the description comes first"
    );
    assert_eq!(t.volume.refs, 2);
}

#[test]
fn a_reference_adds_to_the_weight() {
    let mut bare = task();
    bare.description = Some("mirar el despliegue del proxy lateral".into());
    bare.retally();

    let mut pointed = task();
    pointed.description = Some("mirar el despliegue del proxy lateral [[CUSLEG-3465]]".into());
    pointed.retally();

    assert!(pointed.weight() > bare.weight());
}

#[test]
fn a_reference_that_leaves_the_prose_stops_counting() {
    let mut t = task();
    t.description = Some("sale de [[CUSLEG-3465]]".into());
    t.retally();
    assert_eq!(t.volume.refs, 1);

    t.description = Some("sale de otra parte".into());
    t.retally();
    assert_eq!(
        t.volume.refs, 0,
        "the index outlived the prose it came from"
    );
}

#[test]
fn a_completed_task_is_archived_not_gone() {
    let mut t = task();
    t.status = Status::Done;
    assert!(t.is_archived());
    assert!(!t.is_open());
}

fn daily() -> crate::model::Repeat {
    crate::model::Repeat::due(crate::model::Cadence {
        every: 1,
        unit: crate::model::Unit::Day,
    })
}

#[test]
fn a_repeating_task_is_a_routine_even_when_it_carries_a_journal() {
    let mut chore = task();
    chore.repeat = Some(daily());
    chore.log.push(entry(
        "the pharmacy was shut so it waited until the next morning, and the box was \
         already open by then",
    ));
    chore.retally();

    assert!(chore.weight() > 0, "the note is real substance");
    assert_eq!(
        chore.reading(),
        Reading::Routine,
        "the series outranks any single turn"
    );
}

#[test]
fn a_turn_is_a_routine_through_its_chain_alone() {
    let mut turn = task();
    turn.after = Some(Ulid::generate());
    turn.retally();

    assert_eq!(turn.reading(), Reading::Routine);
}

#[test]
fn a_task_with_nothing_written_is_a_trace() {
    let mut errand = task();
    errand.retally();

    assert_eq!(errand.reading(), Reading::Trace);
}

#[test]
fn the_agenda_alone_never_lifts_a_trace() {
    let mut errand = task();
    errand.date = Some(DateSpec::all_day("2026-08-05".parse().unwrap(), "UTC"));
    errand.deadline = Some(DateSpec::all_day("2026-08-09".parse().unwrap(), "UTC"));
    errand.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];
    errand.list = Some(Ulid::generate());
    errand.reminders = vec![DateSpec::all_day("2026-08-04".parse().unwrap(), "UTC")];
    errand.retally();

    assert_eq!(
        errand.reading(),
        Reading::Trace,
        "dates and labels are not something learnt"
    );
}

#[test]
fn a_note_alone_leaves_an_errand_where_it_was() {
    let mut one = task();
    one.retally();
    assert_eq!(one.reading(), Reading::Trace);

    one.log
        .push(entry("the courier leaves the parcel with the neighbour"));
    one.retally();

    assert_eq!(
        one.reading(),
        Reading::Trace,
        "somebody wrote one line on a errand; it is still an errand"
    );
}

#[test]
fn what_was_learnt_along_the_way_lifts_a_trace_into_a_story() {
    let mut one = task();
    one.log
        .push(entry("the courier leaves the parcel with the neighbour"));
    one.log.push(entry(
        "the neighbour is away until the fifteenth of the month",
    ));
    one.log.push(entry(
        "it went back to the depot and has to be asked for again",
    ));
    one.retally();

    assert_eq!(
        one.reading(),
        Reading::Story,
        "the layer is read from what is there, never stored"
    );
}

fn closed(mut one: Task) -> Task {
    one.status = Status::Done;
    one.completed_at = Some(Timestamp::UNIX_EPOCH);
    one
}

fn story() -> Task {
    let mut one = task();
    for line in [
        "the courier leaves the parcel with the neighbour",
        "the neighbour is away until the fifteenth of the month",
        "it went back to the depot and has to be asked for again",
    ] {
        one.log.push(entry(line));
    }
    one.retally();
    assert_eq!(one.reading(), Reading::Story);
    one
}

#[test]
fn a_trace_kept_as_a_story_reads_as_one_whatever_it_weighs() {
    let mut errand = task();
    errand.retally();
    errand.read_as = Some(Reading::Story);

    assert_eq!(errand.reading(), Reading::Story);
    assert_eq!(
        errand.weight(),
        0,
        "the weight keeps telling what was written"
    );
    assert_eq!(
        errand.heft(),
        STORY_AT,
        "and the heft lands it with the stories"
    );
}

#[test]
fn a_story_read_as_a_trace_reads_as_one_whatever_it_weighs() {
    let mut one = story();
    one.read_as = Some(Reading::Trace);

    assert_eq!(one.reading(), Reading::Trace);
    assert!(one.weight() >= STORY_AT);
    assert_eq!(one.heft(), STORY_AT - 1);
}

#[test]
fn a_routine_reads_as_a_routine_however_it_is_pinned() {
    let mut turn = task();
    turn.after = Some(Ulid::generate());
    turn.read_as = Some(Reading::Story);
    turn.retally();

    assert_eq!(turn.reading(), Reading::Routine);
    assert_eq!(turn.heft(), turn.weight());
}

#[test]
fn heft_leaves_what_was_not_converted_alone() {
    let mut errand = task();
    errand.retally();
    let one = story();

    assert_eq!(errand.heft(), errand.weight());
    assert_eq!(one.heft(), one.weight());
}

/// The content grows after the conversion: the layer the person chose holds, and the
/// weight goes on telling the truth about what is written.
#[test]
fn what_is_written_after_a_conversion_does_not_undo_it() {
    let mut one = closed(story());
    one.read_as = Some(Reading::Trace);
    for _ in 0..3 {
        one.log.push(entry(
            "another long entry about the parcel, the depot, the neighbour and the courier \
             who never rings twice",
        ));
    }
    one.retally();

    assert_eq!(one.reading(), Reading::Trace);
    assert_eq!(one.erasable(), Ok(()));
    assert!(one.weight() > STORY_AT, "{}", one.weight());
    assert_eq!(one.heft(), STORY_AT - 1);
}

#[test]
fn only_a_closed_trace_is_erasable() {
    let open = task();
    assert_eq!(open.erasable(), Err(Stays::Open));
    let mut open_story = story();
    open_story.read_as = Some(Reading::Trace);
    assert_eq!(
        open_story.erasable(),
        Err(Stays::Open),
        "open is open, converted or not"
    );

    let mut errand = closed(task());
    errand.retally();
    assert_eq!(errand.erasable(), Ok(()));
    errand.hidden = true;
    assert_eq!(errand.erasable(), Ok(()), "folded away changes nothing");
    errand.status = Status::Dropped;
    assert_eq!(errand.erasable(), Ok(()), "dropped is closed");

    let mut kept = closed(task());
    kept.retally();
    kept.read_as = Some(Reading::Story);
    assert_eq!(
        kept.erasable(),
        Err(Stays::Story),
        "kept as a story, it stays"
    );

    let mut one = closed(story());
    assert_eq!(one.erasable(), Err(Stays::Story));
    one.read_as = Some(Reading::Trace);
    assert_eq!(one.erasable(), Ok(()), "converted, it goes");

    let mut turn = closed(task());
    turn.after = Some(Ulid::generate());
    assert_eq!(turn.erasable(), Err(Stays::Routine));
    turn.read_as = Some(Reading::Trace);
    assert_eq!(
        turn.erasable(),
        Err(Stays::Routine),
        "a pin never reaches a routine"
    );
}

#[test]
fn a_conversion_round_trips_and_absent_is_absent() {
    let mut one = task();
    one.read_as = Some(Reading::Trace);
    let json = serde_json::to_string(&one).unwrap();
    assert!(json.contains(r#""read_as":"trace""#), "{json}");
    let back: Task = serde_json::from_str(&json).unwrap();
    assert_eq!(back.read_as, Some(Reading::Trace));

    let before: Task = serde_json::from_str(&serde_json::to_string(&task()).unwrap()).unwrap();
    assert_eq!(before.read_as, None);
}

#[test]
fn the_weight_a_single_note_and_a_link_carry_is_not_a_story() {
    let mut errand = task();
    errand.log.push(entry(
        "left at [the depot](https://parcels.example/1) after two tries",
    ));
    errand.retally();

    assert_eq!(
        errand.weight(),
        1,
        "a line that short is only its reference"
    );
    assert_eq!(
        errand.reading(),
        Reading::Trace,
        "this is the shape every «comprar pan» in a real archive has"
    );
}
