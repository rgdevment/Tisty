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

#[test]
fn a_task_card_counts_the_steps_ticked_apart_from_those_an_agent_said_done() {
    use crate::Task;
    use crate::model::Step;

    let mut task = Task::new(ulid::Ulid::generate(), "Renew the certificate", "a0");
    task.steps = [(true, false), (false, true), (false, false)]
        .iter()
        .enumerate()
        .map(|(n, (done, said))| Step {
            id: ulid::Ulid::generate(),
            text: format!("step {n}"),
            done: *done,
            order: format!("a{n}"),
            by_agent: false,
            said: *said,
        })
        .collect();
    task.retally();

    let own = super::of_task(&task, Vec::new());

    assert_eq!(
        task.step_count(),
        crate::model::StepCount {
            done: 1,
            said: 1,
            all: 3
        },
        "the person ticked one, and an agent said one more done"
    );
    assert_eq!(
        own.facts.steps,
        Some([1, 3]),
        "only the ticked count as done"
    );
    assert_eq!(
        own.facts.steps_said,
        Some(1),
        "the agent's word is counted apart"
    );
    task.status = crate::model::Status::Dropped;
    task.retally();
    assert_eq!(
        super::of_task(&task, Vec::new()).facts.steps_said,
        None,
        "a closed task waits on nobody"
    );
    let bare = super::of_task(&Task::new(ulid::Ulid::generate(), "Bare", "a0"), Vec::new());
    assert_eq!(bare.facts.steps, None);
    assert_eq!(bare.facts.steps_said, None);
}
