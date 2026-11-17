use super::sections;

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
