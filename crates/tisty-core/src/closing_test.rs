use super::{THIS_TIME, said_in, summed, this_time};

const ROUND: &str = "## Para qué\n<!-- Por qué vuelve. -->\n\nRegar.\n\n## Esta vez\n<!-- ¿Algo distinto? -->\n\nLa maceta grande se secó.\n\n```\n## no es una sección\n```\n";

#[test]
fn a_round_s_own_words_leave_with_it_and_the_hint_stays() {
    let (said, cleared) = this_time(ROUND).unwrap();

    assert_eq!(
        said,
        "La maceta grande se secó.\n\n```\n## no es una sección\n```"
    );
    assert_eq!(
        cleared,
        "## Para qué\n<!-- Por qué vuelve. -->\n\nRegar.\n\n## Esta vez\n<!-- ¿Algo distinto? -->\n"
    );
    assert_eq!(said_in(&cleared, &THIS_TIME), "");
}

#[test]
fn a_round_with_nothing_written_this_time_leaves_the_page_alone() {
    assert_eq!(this_time("## Esta vez\n<!-- ¿Algo distinto? -->\n"), None);
    assert_eq!(this_time("## Para qué\n\nRegar.\n"), None);
}

#[test]
fn only_the_next_section_as_high_ends_this_one() {
    let body = "## Esta vez\n\nSe secó.\n\n### Detalle\n\nLa grande.\n\n## Después\n\nNada.\n";
    let (said, cleared) = this_time(body).unwrap();

    assert_eq!(said, "Se secó.\n\n### Detalle\n\nLa grande.");
    assert_eq!(cleared, "## Esta vez\n\n## Después\n\nNada.\n");
}

#[test]
fn how_it_went_goes_first_in_its_section() {
    let body = "## Pasos\n\n- [x] Llamar\n\n## Al cerrarla\n<!-- ¿Qué funcionó? -->\n";
    assert_eq!(
        summed(body, "Salió bien.\nLlamar temprano ayuda."),
        "## Pasos\n\n- [x] Llamar\n\n## Al cerrarla\n<!-- ¿Qué funcionó? -->\n\nSalió bien.\nLlamar temprano ayuda.\n"
    );
    let written = "## Al cerrarla\n<!-- ¿Qué funcionó? -->\n\nYa lo había pensado.\n\n## Otra\n";
    assert_eq!(
        summed(written, "Salió bien."),
        "## Al cerrarla\n<!-- ¿Qué funcionó? -->\n\nSalió bien.\n\nYa lo había pensado.\n\n## Otra\n"
    );
    assert_eq!(
        summed(
            "## Al cerrarla\n<!-- ¿Qué funcionó? -->\n\n## Otra\n",
            "Bien."
        ),
        "## Al cerrarla\n<!-- ¿Qué funcionó? -->\n\nBien.\n\n## Otra\n"
    );
}

#[test]
fn a_page_with_no_place_for_it_ends_with_how_it_went() {
    assert_eq!(
        summed("Notas sueltas.\n\n", "Bien."),
        "Notas sueltas.\n\nBien.\n"
    );
}
