use super::*;

const TASK: &str = "01M7NC27JDW2R8VEASK8NYP628";

fn named(text: &str) -> Vec<(Named, &str)> {
    spots(text)
        .into_iter()
        .map(|one| {
            let said = &text[one.at.clone()];
            (one.named, said)
        })
        .collect()
}

#[test]
fn a_bare_task_id_and_a_document_id_are_named() {
    let text = format!("See {TASK} and 4nnctnme-0033, then rest.");

    assert_eq!(
        named(&text),
        vec![
            (Named::Id(TASK.into()), TASK),
            (Named::Id("4nnctnme-0033".into()), "4nnctnme-0033"),
        ]
    );
}

#[test]
fn words_between_quotes_are_named_without_the_quotes() {
    let text = "Read «Brainsty: decisiones», “Trips” and \"The vet\".";

    assert_eq!(
        named(text),
        vec![
            (
                Named::Quoted("Brainsty: decisiones".into()),
                "Brainsty: decisiones"
            ),
            (Named::Quoted("Trips".into()), "Trips"),
            (Named::Quoted("The vet".into()), "The vet"),
        ]
    );
}

#[test]
fn spaces_inside_the_quotes_stay_out_of_what_is_named() {
    assert_eq!(
        named("Lee «  Viajes » hoy."),
        vec![(Named::Quoted("Viajes".into()), "Viajes")]
    );
}

#[test]
fn a_double_bracket_is_named_whole() {
    assert_eq!(
        named("As [[Brainsty: decisiones]] says."),
        vec![(
            Named::Wiki("Brainsty: decisiones".into()),
            "[[Brainsty: decisiones]]"
        )]
    );
}

#[test]
fn code_links_front_matter_and_addresses_name_nothing() {
    let text = format!(
        "---\ntitle: \"Trips\"\n---\n\n`{TASK}` and [«Trips»](tisty:doc/a-0001) and \
         [x](tisty:task/{TASK}) at https://example.com/4nnctnme-0033 and <a href=\"«x»\">.\n\n\
         ```\n«Trips» {TASK}\n```\n"
    );

    assert_eq!(named(&text), vec![]);
}

#[test]
fn the_words_of_a_link_by_reference_name_nothing_but_plain_brackets_do() {
    let text = "See [«Trips» notes][1] and [«Vet»].\n\n[1]: https://example.com\n";

    assert_eq!(named(text), vec![(Named::Quoted("Vet".into()), "Vet")]);
}

#[test]
fn a_comment_names_nothing_and_an_unclosed_double_bracket_hides_nothing_after_it() {
    let text = format!("<!-- «Trips» -->\nUse [[ later\n\nCall {TASK}.\n\nSee [[Vet]]");

    assert_eq!(
        named(&text),
        vec![
            (Named::Id(TASK.into()), TASK),
            (Named::Wiki("Vet".into()), "[[Vet]]"),
        ]
    );
}

#[test]
fn a_step_named_by_its_number_is_no_title() {
    assert_eq!(
        named("Do [[#2]] next, then [[Trips]]."),
        vec![(Named::Wiki("Trips".into()), "[[Trips]]")]
    );
}

#[test]
fn an_id_glued_to_a_path_or_a_file_name_is_no_id() {
    let text = format!("docs/{TASK} and 4nnctnme-0033.md and x{TASK} and v1.4nnctnme-0033");

    assert_eq!(named(&text), vec![]);
}

#[test]
fn an_id_between_quotes_is_named_as_an_id() {
    let text = format!("The task «{TASK}» moved.");

    assert_eq!(named(&text), vec![(Named::Id(TASK.into()), TASK)]);
}

#[test]
fn quotes_that_hold_marks_or_run_past_the_line_are_not_a_name() {
    let text = "a «[x]» b «open\nclose» c 5\" screen";

    assert_eq!(named(text), vec![]);
}

#[test]
fn an_apostrophe_word_quote_is_not_opened_inside_a_word() {
    assert_eq!(named("say\"so\" now"), vec![]);
}

#[test]
fn lowercase_or_short_runs_are_not_task_ids() {
    let text = "01m7nc27jdw2r8veasknyp628x and 81M7NC27JDW2R8VEASK8NYP628 and covid-19";

    assert_eq!(named(text), vec![]);
}
