use super::{noted, outside, put_back, rewritten, section, signed};

#[test]
fn the_first_note_opens_the_assistant_s_own_section_at_the_end() {
    assert_eq!(
        noted(
            "## Notas\n\nLo mío.\n",
            "**hoy** · Claude\nPrecio: 40.",
            true
        ),
        "## Notas\n\nLo mío.\n\n## Notas del asistente\n\n**hoy** · Claude\nPrecio: 40.\n"
    );
    assert_eq!(
        noted("", "Price: 40.", false),
        "## Assistant notes\n\nPrice: 40.\n"
    );
}

#[test]
fn a_note_s_own_headings_and_open_fence_stay_inside_the_section() {
    let page = noted("## Notas\n", "## Hallazgos\nUno.\n```\ncode", true);
    assert_eq!(
        page,
        "## Notas\n\n## Notas del asistente\n\n### Hallazgos\nUno.\n```\ncode\n```\n"
    );
    assert_eq!(noted(&page, "Dos.", true), format!("{}\nDos.\n", page));
}

#[test]
fn a_later_note_goes_at_the_end_of_that_section_and_before_the_next() {
    let body = "## Notas del asistente\n\nUna.\n\n## Al cerrarla\n\nBien.\n";
    assert_eq!(
        noted(body, "Dos.", true),
        "## Notas del asistente\n\nUna.\n\nDos.\n\n## Al cerrarla\n\nBien.\n"
    );
    assert_eq!(
        noted("## Assistant notes\n\nOne.\n", "Two.", true),
        "## Assistant notes\n\nOne.\n\nTwo.\n"
    );
}

#[test]
fn writing_the_notes_again_replaces_them_and_leaves_the_rest_of_the_page() {
    let body = "## Notas\n\nLo mío.\n\n## Notas del asistente\n\nUna.\n\nDos.\n\n## Al cerrarla\n\nBien.\n";

    assert_eq!(
        rewritten(body, "Resumen.", true),
        "## Notas\n\nLo mío.\n\n## Notas del asistente\n\nResumen.\n\n## Al cerrarla\n\nBien.\n"
    );
    assert_eq!(
        rewritten("## Notas\n\nLo mío.\n", "Resumen.", true),
        noted("## Notas\n\nLo mío.\n", "Resumen.", true),
        "a page with no notes yet gets them the usual way"
    );
}

#[test]
fn a_digest_never_carries_a_box_that_would_become_a_step() {
    let page = rewritten(
        "## Assistant notes\n\nOld.\n",
        "- [ ] still to do\n- [x] done",
        false,
    );

    assert!(!page.contains("\n- [ ]"), "{page}");
    assert!(!page.contains("\n- [x]"), "{page}");
    assert!(page.contains("still to do"), "{page}");
}

#[test]
fn what_lies_outside_the_notes_is_the_page_without_them() {
    let body = "## Notas\n\nLo mío.\n\n## Notas del asistente\n\nUna.\n\n## Al cerrarla\n\nBien.\n";

    assert_eq!(
        outside(body),
        "## Notas\n\nLo mío.\n\n## Al cerrarla\n\nBien.\n"
    );
    assert_eq!(
        outside(body),
        outside(&rewritten(body, "Otra.", true)),
        "writing the notes again changes nothing outside them"
    );
}

#[test]
fn the_notes_are_read_and_put_back_without_touching_the_rest_of_the_page() {
    let body = "## Notas\n\nLo mío.\n\n## Notas del asistente\n\nUna.\n\n## Al cerrarla\n\nBien.\n";

    let words = section(body).unwrap();
    assert!(words.contains("Una."), "{words:?}");
    assert!(
        !words.contains("Lo mío.") && !words.contains("Bien."),
        "{words:?}"
    );
    let digest = rewritten(body, "Resumen.", true);
    assert_eq!(put_back(&digest, words).as_deref(), Some(body));
    assert_eq!(section("## Notas\n\nLo mío.\n"), None);
    assert_eq!(put_back("## Notas\n\nLo mío.\n", "Una."), None);
}

#[test]
fn each_dated_entry_names_who_signed_it_and_words_before_the_first_are_unsigned() {
    let words = "\n**2026-11-10 09:30** · Claude\nUna.\n\n**2026-11-11 18:02** · Codex\nDos.\n";
    assert_eq!(
        signed(words),
        (vec!["Claude".to_string(), "Codex".to_string()], false)
    );

    let theirs = "\nMis palabras.\n\n**2026-11-10 09:30** · Claude\nUna.\n";
    assert_eq!(signed(theirs), (vec!["Claude".to_string()], true));

    assert_eq!(signed("**hoy** · Claude\nUna.\n"), (Vec::new(), true));
}
