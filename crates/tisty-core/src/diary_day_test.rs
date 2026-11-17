use super::*;

const NOTE: &str = "> [!IDEA] 2026-11-16 10:05 · yellow · right\n> Comprar pan";
const OTHER_NOTE: &str =
    "> [!IDEA] 2026-11-16 18:40 · teal · right\n> Llamar a Jaime\n> antes del jueves";

fn month() -> String {
    format!(
        "# Noviembre\n\n## Lunes 16\n\nPaseo con Rina.\n\n{NOTE}\n\nCena con Marisol.\n\n{OTHER_NOTE}\n\n## Domingo 15\n\nLluvia todo el día.\n\n## Sábado 14\n\nFeria del barrio.\n"
    )
}

const MONDAY: DayAt = DayAt { day: 16, nth: 0 };
const SUNDAY: DayAt = DayAt { day: 15, nth: 0 };

fn day_texts(body: &str) -> Vec<(i8, String)> {
    let lines: Vec<&str> = body.lines().collect();
    cuts(body)
        .iter()
        .map(|one| {
            (
                one.day,
                lines[one.heading + 1..one.end]
                    .join("\n")
                    .trim()
                    .to_string(),
            )
        })
        .collect()
}

#[test]
fn a_note_is_taken_whole_from_its_marker_to_its_last_quoted_line() {
    let (words, notes) = parted(&format!(
        "\nPaseo con Rina.\n\n{NOTE}\n\nCena con Marisol.\n\n{OTHER_NOTE}\n"
    ));

    assert_eq!(words, "Paseo con Rina.\n\nCena con Marisol.");
    assert_eq!(notes, vec![NOTE.to_string(), OTHER_NOTE.to_string()]);
}

#[test]
fn a_note_inside_a_list_or_a_fence_is_words_not_a_note() {
    let day = "- Feria\n\n  > [!IDEA] 2026-11-16 10:05 · yellow · right\n  > dentro\n\n```\n> [!IDEA] 2026-11-16 11:00 · teal · right\n> código\n```";

    let (words, notes) = parted(day);

    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(words, day);
}

#[test]
fn the_words_a_day_shows_leave_its_notes_out() {
    assert_eq!(
        words(&month(), MONDAY).as_deref(),
        Some("Paseo con Rina.\n\nCena con Marisol.")
    );
    assert_eq!(
        words(&month(), SUNDAY).as_deref(),
        Some("Lluvia todo el día.")
    );
    assert_eq!(words(&month(), DayAt { day: 13, nth: 0 }), None);
}

#[test]
fn new_words_keep_the_notes_after_them_in_their_order_and_leave_every_other_day_alone() {
    let body = month();
    let was = words(&body, MONDAY).unwrap();

    let after = rewritten(
        &body,
        MONDAY,
        &was,
        "  Paseo largo con Rina.\n## Martes 3\nCine.\n# Noche\n## Tarde\n```\n## Lunes 2\n```  ",
    )
    .unwrap();

    let before_days = day_texts(&body);
    let after_days = day_texts(&after);
    assert_eq!(after_days.len(), 3);
    assert_eq!(after_days[1..], before_days[1..]);
    assert_eq!(
        after_days[0].1,
        format!(
            "Paseo largo con Rina.\n### Martes 3\nCine.\n### Noche\n## Tarde\n```\n## Lunes 2\n```\n\n{NOTE}\n\n{OTHER_NOTE}"
        )
    );
    assert!(after.starts_with("# Noviembre\n\n## Lunes 16\n\nPaseo largo"));
}

#[test]
fn a_page_written_with_crlf_stays_crlf_and_the_others_keep_their_bytes() {
    let body = month().replace('\n', "\r\n");
    let was = words(&body, SUNDAY).unwrap();

    let after = rewritten(&body, SUNDAY, &was, "Sol por la tarde.").unwrap();

    assert_eq!(after.matches('\n').count(), after.matches("\r\n").count());
    let monday = format!(
        "## Lunes 16\r\n\r\nPaseo con Rina.\r\n\r\n{}",
        NOTE.replace('\n', "\r\n")
    );
    assert!(after.contains(&monday), "{after}");
    assert!(after.contains(
        "## Domingo 15\r\n\r\nSol por la tarde.\r\n\r\n## Sábado 14\r\n\r\nFeria del barrio.\r\n"
    ));
}

#[test]
fn emptied_words_leave_the_notes_and_a_day_left_with_nothing_goes() {
    let body = month();

    let only_notes = rewritten(&body, MONDAY, &words(&body, MONDAY).unwrap(), "  ").unwrap();
    assert_eq!(
        day_texts(&only_notes)[0].1,
        format!("{NOTE}\n\n{OTHER_NOTE}")
    );

    let gone = rewritten(&body, SUNDAY, "Lluvia todo el día.", "\n").unwrap();
    assert_eq!(
        day_texts(&gone)
            .iter()
            .map(|(day, _)| *day)
            .collect::<Vec<_>>(),
        vec![16, 14]
    );
    assert!(
        gone.contains("antes del jueves\n\n## Sábado 14\n\nFeria del barrio.\n"),
        "{gone}"
    );
}

#[test]
fn words_changed_since_the_box_opened_are_not_written_over() {
    let body = month();

    assert_eq!(rewritten(&body, SUNDAY, "Lluvia.", "Sol."), None);
    assert_eq!(
        rewritten(&body, DayAt { day: 13, nth: 0 }, "", "Sol."),
        None
    );
}

#[test]
fn of_two_days_with_one_number_only_the_one_asked_for_changes() {
    let body = "# Noviembre\n\n## Lunes 16\n\nPrimero.\n\n## Lunes 16\n\nSegundo.\n";

    let after = rewritten(body, DayAt { day: 16, nth: 1 }, "Segundo.", "Otro.").unwrap();

    assert_eq!(
        after,
        "# Noviembre\n\n## Lunes 16\n\nPrimero.\n\n## Lunes 16\n\nOtro.\n"
    );
    assert_eq!(
        places([16, 15, 16, 16]),
        vec![
            DayAt { day: 16, nth: 0 },
            DayAt { day: 15, nth: 0 },
            DayAt { day: 16, nth: 1 },
            DayAt { day: 16, nth: 2 },
        ]
    );
}

#[test]
fn a_heading_that_names_no_day_is_part_of_the_day_and_is_rewritten_with_it() {
    let body = "# Noviembre\n\n## Domingo 15\n\nMañana.\n\n## Para recordar\n\nLa llave.\n\n## Sábado 14\n\nFeria.\n";
    let was = words(body, SUNDAY).unwrap();
    assert_eq!(was, "Mañana.\n\n## Para recordar\n\nLa llave.");

    let after = rewritten(body, SUNDAY, &was, "Todo en uno.").unwrap();
    let kept = rewritten(body, SUNDAY, &was, &was.replace("Mañana", "Temprano")).unwrap();

    assert_eq!(
        after,
        "# Noviembre\n\n## Domingo 15\n\nTodo en uno.\n\n## Sábado 14\n\nFeria.\n"
    );
    assert_eq!(kept, body.replace("Mañana", "Temprano"), "{kept}");
}

#[test]
fn a_note_taken_off_leaves_the_words_and_the_other_notes_as_they_were() {
    let body = month();

    let after = unstuck(&body, MONDAY, &body, 0).unwrap();

    assert_eq!(
        day_texts(&after)[0].1,
        format!("Paseo con Rina.\n\nCena con Marisol.\n\n{OTHER_NOTE}")
    );
    assert_eq!(day_texts(&after)[1..], day_texts(&body)[1..]);
    assert!(!after.contains("\n\n\n"), "{after}");
}

#[test]
fn the_last_note_of_a_day_with_no_words_takes_the_day_with_it() {
    let body = format!("# Noviembre\n\n## Lunes 16\n\n{NOTE}\n\n## Domingo 15\n\nLluvia.\n");

    let after = unstuck(&body, MONDAY, &body, 0).unwrap();

    assert_eq!(after, "# Noviembre\n\n## Domingo 15\n\nLluvia.\n");
}

#[test]
fn a_note_that_is_not_the_one_the_page_showed_stays() {
    let shown = month();
    let moved = shown.replace("Comprar pan", "Comprar leche");

    assert_eq!(unstuck(&moved, MONDAY, &shown, 0), None);
    assert_eq!(unstuck(&shown, MONDAY, &shown, 2), None);
    assert_eq!(unstuck(&shown, SUNDAY, &shown, 0), None);
}
