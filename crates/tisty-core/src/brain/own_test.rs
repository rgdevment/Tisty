use super::sections;

#[test]
fn a_section_is_placed_by_the_file_s_own_lines_past_properties_code_and_tags() {
    let body = "---\ntitle: Viaje\nlang: es\n---\n# Viaje\n\n#viajes\n\n## Hoteles\n\n```sh\n\
                # not a heading\n```\n\nCerca del mar.\n\n### Precios\n\n100 €\n\n## Comida\nFin\n";

    let all = sections(body);

    let placed: Vec<(&str, usize, [usize; 2])> = all
        .iter()
        .map(|one| (one.title.as_str(), one.level, one.lines))
        .collect();
    assert_eq!(
        placed,
        [
            ("Viaje", 1, [5, 22]),
            ("Hoteles", 2, [9, 19]),
            ("Precios", 3, [17, 19]),
            ("Comida", 2, [21, 22]),
        ],
        "properties count as lines, and neither the fence nor the tag opens a section"
    );
    for one in &all {
        let [from, to] = one.bytes;
        assert_eq!(
            &body[from..to],
            crate::docs::lines_between(body, one.lines[0], one.lines[1]),
            "{} reads the same by bytes and by lines",
            one.title
        );
        assert!(
            body[from..].starts_with('#'),
            "{} starts at its heading",
            one.title
        );
    }
    let [from, to] = all[1].bytes;
    assert!(body[from..to].contains("# not a heading\n```\n\nCerca del mar."));
}

#[test]
fn a_section_s_bytes_cut_the_same_text_as_its_lines() {
    let body =
        "# Viaje\r\n\r\nAñadir café.\r\n\r\n## Día uno\r\nLlegué tarde.\r\n\r\n## Día dos\nFin";

    let all = sections(body);

    let titles: Vec<&str> = all.iter().map(|one| one.title.as_str()).collect();
    assert_eq!(titles, ["Viaje", "Día uno", "Día dos"]);
    for one in &all {
        let [from, to] = one.bytes;
        assert_eq!(
            &body[from..to],
            crate::docs::lines_between(body, one.lines[0], one.lines[1]),
            "{} reads the same by bytes and by lines",
            one.title
        );
    }
    assert_eq!(
        all[2].bytes[1],
        body.len(),
        "the last section runs to the end"
    );
}
