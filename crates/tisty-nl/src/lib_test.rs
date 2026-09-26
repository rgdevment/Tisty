#[test]
fn a_span_counts_letters_and_not_bytes() {
    for text in [
        "revisión del informe #trabajo",
        "🎉 la fiesta de mañana #casa",
        "compra pan #ñandú y leche",
    ] {
        let read = parse(text, &now(), "es");
        let letters: Vec<char> = text.chars().collect();
        for span in &read.spans {
            assert!(
                span.to <= letters.len(),
                "{text}: span {}..{} sale de {} letras ({} bytes)",
                span.from,
                span.to,
                letters.len(),
                text.len()
            );
            let written: String = letters[span.from..span.to].iter().collect();
            assert!(
                text.contains(written.trim()),
                "{text}: el span dice «{written}», que no está en el texto"
            );
        }
    }
}

use super::*;

fn now() -> Zoned {
    "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
}

fn spans(input: &str, locale: &str) -> Vec<(String, Mark, Certainty)> {
    let parsed = parse(input, &now(), locale);
    let chars: Vec<char> = input.chars().collect();
    parsed
        .spans
        .iter()
        .map(|s| (chars[s.from..s.to].iter().collect(), s.mark, s.certainty))
        .collect()
}

fn offered(input: &str, locale: &str) -> Option<(String, String, String)> {
    let parsed = parse(input, &now(), locale);
    let chars: Vec<char> = input.chars().collect();
    parsed.offers.first().map(|offer| {
        let span = offer.spans[0];
        (
            chars[span.from..span.to].iter().collect(),
            offer.date.date().to_string(),
            offer.title.clone(),
        )
    })
}

#[test]
fn a_bare_three_is_this_afternoon_and_not_tomorrow_dawn() {
    let seven: Zoned = "2026-08-05T07:00:00[America/Santiago]".parse().unwrap();
    let p = parse("tomar café a las 3", &seven, "es");
    let at = p.date.expect("a date");

    assert_eq!(at.date().to_string(), "2026-08-05");
    assert_eq!(at.at.time().to_string(), "15:00:00");
    assert_eq!(p.spans[0].certainty, Certainty::Assumed);
}

#[test]
fn the_same_three_rolls_over_once_the_afternoon_is_gone() {
    let evening: Zoned = "2026-08-05T18:00:00[America/Santiago]".parse().unwrap();
    let p = parse("tomar café a las 3", &evening, "es");
    let at = p.date.expect("a date");

    assert_eq!(at.date().to_string(), "2026-08-06");
    assert_eq!(at.at.time().to_string(), "15:00:00");
}

#[test]
fn a_described_noun_comes_back_as_an_offer() {
    assert_eq!(
        offered("revisar el informe del lunes", "es"),
        Some((
            "del lunes".to_string(),
            "2026-08-10".to_string(),
            "revisar el informe".to_string()
        ))
    );
}

#[test]
fn an_offer_before_a_noun_leaves_its_article_behind() {
    assert_eq!(
        offered("review the monday report", "en"),
        Some((
            "monday".to_string(),
            "2026-08-10".to_string(),
            "review the report".to_string()
        ))
    );
}

#[test]
fn a_word_that_means_something_else_is_not_offered() {
    for input in [
        "reunión por la mañana",
        "mañana de verano",
        "revisar lo de hace 3 días",
    ] {
        assert_eq!(offered(input, "es"), None, "{input}");
    }
}

#[test]
fn nothing_is_offered_once_something_was_taken() {
    let p = parse(
        "preparar la reunión del martes para el jueves",
        &now(),
        "es",
    );
    assert!(p.date.is_some());
    assert!(p.offers.is_empty());
}

#[test]
fn a_date_flag_takes_the_offer_it_would_otherwise_hold_back() {
    assert!(parse_date("del lunes", &now(), "es").is_some());
}

#[test]
fn a_phrase_without_anything_temporal_keeps_its_whole_title() {
    let p = parse("actualizar las dependencias", &now(), "es");
    assert_eq!(p.title, "actualizar las dependencias");
    assert!(p.date.is_none());
    assert!(p.spans.is_empty());
}

#[test]
fn markers_are_taken_out_of_the_title() {
    let p = parse("revisar el deploy #backend !hacer", &now(), "es");
    assert_eq!(p.title, "revisar el deploy");
    assert_eq!(p.priority, Some(Priority::Do));
    assert_eq!(p.tags.len(), 1);
}

#[test]
fn a_bare_date_needs_no_title_around_it() {
    assert!(parse_date("mañana", &now(), "es").is_some());
    assert!(parse_date("2026-12-24", &now(), "es").is_some());
    assert!(parse_date("next friday", &now(), "en").is_some());
    assert!(parse_date("not a date", &now(), "en").is_none());
}

#[test]
fn a_regional_locale_still_speaks_its_language() {
    let p = parse("comprar pan mañana", &now(), "es-CL");
    assert_eq!(p.title, "comprar pan");
    assert!(p.date.is_some());
}

#[test]
fn a_decomposed_enye_is_still_a_word() {
    let p = parse("comprar pan man\u{0303}ana", &now(), "es");
    assert_eq!(p.title, "comprar pan");
    assert!(p.date.is_some());
}

#[test]
fn quoted_text_is_never_interpreted() {
    let p = parse("\"reunión el lunes\"", &now(), "es");
    assert_eq!(p.title, "reunión el lunes");
    assert!(p.date.is_none());
    assert!(p.spans.is_empty());
}

#[test]
fn every_span_points_at_what_it_read() {
    assert_eq!(
        spans("comprar pan mañana #casa @compras !hacer", "es"),
        [
            ("mañana".to_string(), Mark::Date, Certainty::Sure),
            ("#casa".to_string(), Mark::Tag, Certainty::Sure),
            ("@compras".to_string(), Mark::List, Certainty::Sure),
            ("!hacer".to_string(), Mark::Priority, Certainty::Sure),
        ]
    );
}

#[test]
fn offsets_survive_a_marker_written_with_accents() {
    assert_eq!(
        spans("#niño revisar la sesión mañana", "es"),
        [
            ("#niño".to_string(), Mark::Tag, Certainty::Sure),
            ("mañana".to_string(), Mark::Date, Certainty::Sure),
        ]
    );
}

#[test]
fn a_phrase_split_by_the_title_reports_both_halves() {
    assert_eq!(
        spans("reunión el martes en la sala 3c a las 16:00", "es"),
        [
            ("el martes".to_string(), Mark::Date, Certainty::Sure),
            ("a las 16:00".to_string(), Mark::Date, Certainty::Sure),
        ]
    );
}

#[test]
fn a_marker_inside_the_phrase_keeps_its_own_span() {
    assert_eq!(
        spans("reunión el martes #trabajo a las 16:00", "es"),
        [
            ("el martes".to_string(), Mark::Date, Certainty::Sure),
            ("#trabajo".to_string(), Mark::Tag, Certainty::Sure),
            ("a las 16:00".to_string(), Mark::Date, Certainty::Sure),
        ]
    );
}

#[test]
fn a_list_between_the_verb_and_the_day_is_not_swallowed() {
    assert_eq!(
        spans("llamar a @juan mañana", "es"),
        [
            ("@juan".to_string(), Mark::List, Certainty::Sure),
            ("mañana".to_string(), Mark::Date, Certainty::Sure),
        ]
    );
}

#[test]
fn no_span_ever_covers_another() {
    for text in [
        "comprar pan para #casa mañana",
        "entregar el informe para @trabajo el lunes",
        "reunión el martes #trabajo a las 16:00",
        "llamar a @juan mañana !hacer",
    ] {
        let read = parse(text, &now(), "es");
        let mut ranges: Vec<_> = read.spans.iter().map(|s| (s.from, s.to)).collect();
        ranges.sort_unstable();
        for pair in ranges.windows(2) {
            assert!(
                pair[0].1 <= pair[1].0,
                "{text}: {:?} overlaps {:?}",
                pair[0],
                pair[1]
            );
        }
    }
}

#[test]
fn a_deadline_is_marked_as_one() {
    let read = spans("entregar el informe antes del viernes", "es");
    assert_eq!(read[0].1, Mark::Deadline);
}

#[test]
fn mid_sentence_without_a_signal_is_an_assumption() {
    let p = parse("llamar mañana al banco", &now(), "es");
    assert_eq!(p.title, "llamar al banco");
    assert_eq!(p.spans[0].certainty, Certainty::Assumed);
}

#[test]
fn a_phrase_at_the_end_is_no_assumption() {
    let p = parse("llamar al banco mañana", &now(), "es");
    assert_eq!(p.spans[0].certainty, Certainty::Sure);
}

#[test]
fn a_described_noun_is_left_alone() {
    let p = parse("revisar el informe del lunes", &now(), "es");
    assert_eq!(p.title, "revisar el informe del lunes");
    assert!(p.spans.is_empty());
}

#[test]
fn a_span_reaches_the_window_under_the_names_it_declares() {
    let span = Span {
        from: 3,
        to: 9,
        mark: Mark::Deadline,
        certainty: Certainty::Assumed,
    };
    assert_eq!(
        serde_json::to_string(&span).unwrap(),
        r#"{"from":3,"to":9,"mark":"deadline","certainty":"assumed"}"#
    );
}

#[test]
fn quoting_an_accented_word_keeps_the_rest_in_place() {
    let p = parse("mandar \"café ñandú\" mañana", &now(), "es");
    assert_eq!(p.title, "mandar \"café ñandú\"");
    assert!(p.date.is_some());
}

#[test]
fn a_priority_is_named_in_the_language_it_was_asked_for() {
    assert_eq!(priority_word(Priority::Do, "es"), "hacer");
    assert_eq!(priority_word(Priority::Decide, "es"), "planificar");
    assert_eq!(priority_word(Priority::Delegate, "es"), "delegar");
    assert_eq!(priority_word(Priority::Minor, "es"), "prescindible");
    assert_eq!(priority_word(Priority::Do, "en"), "do");
    assert_eq!(priority_word(Priority::Minor, "en"), "minor");
}

#[test]
fn a_priority_is_read_back_from_either_language() {
    assert_eq!(parse_priority("delegar", "es"), Some(Priority::Delegate));
    assert_eq!(parse_priority("IMPORTANTE", "es"), Some(Priority::Decide));
    assert_eq!(parse_priority("ninguna", "es"), Some(Priority::Unset));
    assert_eq!(parse_priority("delegate", "es"), Some(Priority::Delegate));
    assert_eq!(parse_priority("nimiedad", "es"), None);
}

#[test]
fn the_draft_carries_everything_the_parser_read() {
    let read = parse("pagar la luz mañana #casa !hacer @hogar", &now(), "es");
    let draft: Draft = read.clone().into();

    assert_eq!(draft.title, read.title);
    assert_eq!(draft.date, read.date);
    assert_eq!(draft.deadline, read.deadline);
    assert_eq!(draft.priority, read.priority);
    assert_eq!(draft.tags, read.tags);
    assert_eq!(draft.repeat, read.repeat);
    assert_eq!(draft.filing, Some(Filing::Marked("hogar".into())));
    assert!(draft.source.is_none());
    assert_eq!(draft.title, "pagar la luz");
    assert_eq!(draft.priority, Some(Priority::Do));
}

#[test]
fn a_draft_files_nowhere_when_no_list_was_marked() {
    let draft: Draft = parse("pagar la luz", &now(), "es").into();
    assert!(draft.filing.is_none());
}

#[test]
fn a_bare_number_is_neither_a_tag_nor_a_list() {
    let read = parse("revisar el pedido #12 @34", &now(), "es");

    assert!(read.tags.is_empty());
    assert!(read.list.is_none());
    assert_eq!(read.title, "revisar el pedido #12 @34");
}

#[test]
fn a_cadence_is_only_a_cadence_between_one_and_a_thousand() {
    let read = |n: u32| parse(&format!("revisar el archivo cada {n} días"), &now(), "es");

    assert!(read(1).repeat.is_some());
    assert!(read(999).repeat.is_some());
    assert!(read(1000).repeat.is_none());
    assert!(read(0).repeat.is_none());
}

#[test]
fn an_hour_that_has_just_struck_belongs_to_tomorrow() {
    let p = parse("llamar al banco a las 9:00", &now(), "es");
    let at = p.date.expect("a date");

    assert_eq!(at.date().to_string(), "2026-08-06");
    assert_eq!(at.at.time().to_string(), "09:00:00");
}

#[test]
fn an_hour_still_ahead_belongs_to_today() {
    let p = parse("llamar al banco a las 9:30", &now(), "es");
    let at = p.date.expect("a date");

    assert_eq!(at.date().to_string(), "2026-08-05");
    assert_eq!(at.at.time().to_string(), "09:30:00");
}

#[test]
fn a_title_gives_back_every_letter_no_span_covers() {
    let input = "revisar el informe mañana #casa";
    let read = parse(input, &now(), "es");

    assert_eq!(
        title_without(input, &read.spans, "es"),
        "revisar el informe"
    );
    assert_eq!(title_without(input, &[], "es"), input);
}

#[test]
fn a_title_ignores_a_span_it_cannot_read() {
    let input = "revisar el informe";
    let span = |from, to| Span {
        from,
        to,
        mark: Mark::Date,
        certainty: Certainty::Sure,
    };

    assert_eq!(title_without(input, &[span(0, 99)], "es"), input);
    assert_eq!(title_without(input, &[span(12, 3)], "es"), input);
    assert_eq!(title_without(input, &[span(7, 7)], "es"), input);
}

#[test]
fn a_title_takes_the_first_of_two_spans_that_overlap() {
    let input = "revisar el informe mañana #casa";
    let span = |from, to| Span {
        from,
        to,
        mark: Mark::Date,
        certainty: Certainty::Sure,
    };

    assert_eq!(
        title_without(input, &[span(0, 8), span(3, 12)], "es"),
        "el informe mañana #casa"
    );
}

#[test]
fn a_title_takes_both_of_two_spans_that_only_touch() {
    let input = "revisar el informe mañana #casa";
    let span = |from, to| Span {
        from,
        to,
        mark: Mark::Date,
        certainty: Certainty::Sure,
    };

    assert_eq!(
        title_without(input, &[span(19, 26), span(26, 31)], "es"),
        "revisar el informe"
    );
}

#[test]
fn a_date_that_opens_the_phrase_offers_the_title_without_it() {
    let p = parse("el martes reunión de equipo", &now(), "es");
    let offer = p.offers.first().expect("an offer");

    assert_eq!(offer.date.date().to_string(), "2026-08-11");
    assert_eq!(offer.title, "reunión de equipo");
    assert_eq!(p.title, "el martes reunión de equipo");
}

#[test]
fn an_hour_that_opens_the_phrase_is_offered_the_same_way() {
    let p = parse("a las 5 llamar al banco", &now(), "es");
    let offer = p.offers.first().expect("an offer");

    assert_eq!(offer.date.date().to_string(), "2026-08-05");
    assert_eq!(offer.title, "llamar al banco");
}

#[test]
fn only_one_thing_is_ever_offered() {
    let p = parse(
        "revisar el informe del lunes y el acta del martes",
        &now(),
        "es",
    );

    assert_eq!(p.offers.len(), 1);
    assert_eq!(p.offers[0].date.date().to_string(), "2026-08-11");
}

#[test]
fn a_word_that_only_props_a_date_up_is_dropped() {
    let v = vocab::for_locale("es");

    for word in [
        "el", "la", "a", "las", "al", "para", "antes", "hasta", "en", "dentro",
    ] {
        assert!(droppable(word, v), "«{word}» holds a span up on its own");
    }
    for word in ["informe", "banco", "lunes", "mañana"] {
        assert!(!droppable(word, v), "«{word}» is not a prop");
    }

    let en = vocab::for_locale("en");
    for word in ["the", "at", "on", "by", "until", "in", "within"] {
        assert!(droppable(word, en), "«{word}» holds a span up on its own");
    }
}

#[test]
fn a_cadence_written_inside_quotes_is_not_a_cadence() {
    let p = parse("mandar \"cada día algo\" mañana", &now(), "es");

    assert!(p.repeat.is_none());
    assert_eq!(p.title, "mandar \"cada día algo\"");
    assert!(p.date.is_some());
}

#[test]
fn a_cadence_carries_the_hour_the_phrase_gave_it() {
    let p = parse("regar las plantas cada 3 días a las 9:30", &now(), "es");
    let at = p.date.expect("a date");

    assert!(p.repeat.is_some());
    assert_eq!(at.at.time().to_string(), "09:30:00");
    assert!(at.has_time);
}

#[test]
fn a_marker_written_into_an_hour_splits_the_span_around_it() {
    let input = "llamar a las #casa 3 @oficina de la tarde";
    let p = parse(input, &now(), "es");
    let letters: Vec<char> = input.chars().collect();
    let said: Vec<(Mark, String)> = p
        .spans
        .iter()
        .map(|s| (s.mark, letters[s.from..s.to].iter().collect()))
        .collect();

    assert_eq!(p.title, "llamar");
    assert_eq!(
        said,
        vec![
            (Mark::Tag, "#casa".to_string()),
            (Mark::Date, "3".to_string()),
            (Mark::List, "@oficina".to_string()),
            (Mark::Date, "de la tarde".to_string()),
        ]
    );
    assert_eq!(p.date.expect("a date").at.time().to_string(), "15:00:00");
}
