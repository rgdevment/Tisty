use super::noted;

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
