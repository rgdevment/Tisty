use super::*;

const ERRAND: &str = "01M7NC27JDW2R8VEASK8NYP628";
const OTHER: &str = "01M7NC27JDW2R8VEASK8NYP629";

fn known() -> Known {
    let mut known = Known::default();
    for (task, title, id) in [
        (true, "Call the bank", ERRAND),
        (true, "Pack the books", OTHER),
        (false, "Brainsty: decisiones", "4nnctnme-0033"),
        (false, "Trips", "trips-0001"),
        (false, "Vet visits", "vet-0001"),
        (false, "Vet visits", "vet-0002"),
    ] {
        known.add(Target {
            task,
            title: title.into(),
            id: id.into(),
        });
    }
    known
}

fn link(text: &str) -> (String, Heard) {
    let mut heard = Heard::default();
    let said = linked(text, &known(), None, &BTreeSet::new(), &mut heard);
    (said, heard)
}

fn linked_ids(heard: &Heard) -> Vec<&str> {
    heard
        .linked
        .iter()
        .map(|(_, one)| one.id.as_str())
        .collect()
}

#[test]
fn a_bare_id_becomes_a_link_named_by_its_title() {
    let (said, heard) = link(&format!("Call {ERRAND} and read 4nnctnme-0033 first."));

    assert_eq!(
        said,
        format!(
            "Call [Call the bank](tisty:task/{ERRAND}) and read \
             [Brainsty: decisiones](tisty:doc/4nnctnme-0033) first."
        )
    );
    assert_eq!(linked_ids(&heard), [ERRAND, "4nnctnme-0033"]);
}

#[test]
fn a_task_id_alone_in_its_paragraph_becomes_its_card() {
    let (said, _) = link(&format!("Next:\n\n{ERRAND}\n\nThen rest."));

    assert_eq!(
        said,
        format!("Next:\n\n![Call the bank](tisty:task/{ERRAND})\n\nThen rest.")
    );
}

#[test]
fn an_exact_title_between_quotes_or_double_brackets_is_linked_where_it_is_written() {
    let (said, heard) = link(&format!(
        "As «Brainsty: decisiones» says, «Brainsty: decisiones» again, [[Brainsty: decisiones]] \
         too, and «Call the bank», {ERRAND}."
    ));

    assert_eq!(
        said,
        format!(
            "As «[Brainsty: decisiones](tisty:doc/4nnctnme-0033)» says, «Brainsty: decisiones» \
             again, [Brainsty: decisiones](tisty:doc/4nnctnme-0033) too, and \
             «[Call the bank](tisty:task/{ERRAND})», [Call the bank](tisty:task/{ERRAND})."
        ),
        "words between quotes are linked where first named; an id or [[…]] every time"
    );
    assert_eq!(linked_ids(&heard), ["4nnctnme-0033", ERRAND]);
}

#[test]
fn a_double_bracket_title_names_only_documents() {
    let (said, heard) = link("See [[Call the bank]] and [[Trips]].");

    assert_eq!(
        said,
        "See [[Call the bank]] and [Trips](tisty:doc/trips-0001)."
    );
    assert_eq!(linked_ids(&heard), ["trips-0001"]);
    assert_eq!(heard.maybe.len(), 1, "a task named that way is proposed");
    assert_eq!(heard.maybe[0].1[0].id, ERRAND);
}

#[test]
fn a_whole_document_s_first_line_is_its_title_and_is_kept() {
    let body = "Plans for «Brainsty: decisiones»\n\nMore.\n";

    assert_eq!(
        title_line(&json!({ "body": body }), "body").as_deref(),
        Some("Plans for «Brainsty: decisiones»")
    );
    assert_eq!(
        title_line(
            &json!({ "doc": "x-0001", "print": "p", "body": body }),
            "body"
        )
        .as_deref(),
        Some("Plans for «Brainsty: decisiones»")
    );
    for adding in [
        json!({ "doc": "x-0001", "body": body }),
        json!({ "task": ERRAND, "body": body }),
    ] {
        assert_eq!(title_line(&adding, "body"), None, "{adding}");
    }
}

#[test]
fn a_title_two_things_carry_is_proposed_and_left_as_written() {
    let text = "Book «Vet visits» soon.";

    let (said, heard) = link(text);

    assert_eq!(said, text);
    assert_eq!(heard.maybe.len(), 1);
    assert_eq!(heard.maybe[0].0, "Vet visits");
    let ids: Vec<&str> = heard.maybe[0].1.iter().map(|one| one.id.as_str()).collect();
    assert_eq!(ids, ["vet-0001", "vet-0002"]);
}

#[test]
fn a_title_matched_only_loosely_or_a_single_word_between_quotes_is_proposed() {
    let text = "Read «brainsty:  Decisiones» and «Trips».";

    let (said, heard) = link(text);

    assert_eq!(said, text);
    let doubts: Vec<&str> = heard.maybe.iter().map(|(said, _)| said.as_str()).collect();
    assert_eq!(doubts, ["brainsty:  Decisiones", "Trips"]);
}

#[test]
fn headings_tables_and_lines_already_there_are_left_alone() {
    let text = format!(
        "# About «Brainsty: decisiones»\n\nTitle {ERRAND}\n===\n\n| {ERRAND} | x |\n\nKept {ERRAND} line.\n"
    );
    let mut heard = Heard::default();
    let kept_line = format!("Kept {ERRAND} line.");
    let kept: BTreeSet<&str> = [kept_line.as_str()].into();

    let said = linked(&text, &known(), None, &kept, &mut heard);

    assert_eq!(said, text);
    assert!(heard.linked.is_empty());
}

#[test]
fn what_is_being_written_never_links_to_itself() {
    let mut heard = Heard::default();

    let said = linked(
        &format!("This is {ERRAND}, see «Call the bank»."),
        &known(),
        Some(ERRAND),
        &BTreeSet::new(),
        &mut heard,
    );

    assert_eq!(said, format!("This is {ERRAND}, see «Call the bank»."));
    assert!(heard.linked.is_empty());
}

#[test]
fn linking_twice_changes_nothing_more() {
    let (once, _) = link(&format!(
        "Call {ERRAND}, read «Brainsty: decisiones» and [[Trips]]."
    ));

    let (twice, heard) = link(&once);

    assert_eq!(twice, once);
    assert!(heard.linked.is_empty() && heard.maybe.is_empty());
}

#[test]
fn a_title_with_brackets_is_written_escaped() {
    let mut known = known();
    known.add(Target {
        task: false,
        title: "Notes [old]".into(),
        id: "notes-0001".into(),
    });
    let mut heard = Heard::default();

    let said = linked(
        "See notes-0001.",
        &known,
        None,
        &BTreeSet::new(),
        &mut heard,
    );

    assert_eq!(said, "See [Notes \\[old\\]](tisty:doc/notes-0001).");
}

#[test]
fn a_new_picture_whose_words_are_its_file_name_is_asked_about() {
    let text = "![IMG_2034.PNG](img-2034-3f9a1c2b.png)\n\n![Rina on her chair](rina-3f9a1c2b.jpg)\n\n\
                ![](form-1a2b3c4d.pdf)\n\n![](old-9d02e4b7.png)\n";
    let kept: BTreeSet<&str> = ["![](old-9d02e4b7.png)"].into();
    let mut heard = Heard::default();

    linked(text, &known(), None, &kept, &mut heard);

    assert_eq!(heard.unshown, ["img-2034-3f9a1c2b.png"]);
}

#[test]
fn the_answer_carries_what_was_linked_what_is_doubtful_and_asks_about_a_picture_last() {
    let (_, heard) = link(&format!(
        "Call {ERRAND} about «Vet visits».\n\n![](scan-3f9a1c2b.png)\n"
    ));
    let answer = json!({
        "content": [{ "type": "text", "text": "Noted." }],
        "structuredContent": { "id": "x" },
    });

    let told = heard.told(answer);

    let shown = &told["structuredContent"];
    assert_eq!(
        shown["references"]["linked"],
        json!([{ "task": ERRAND, "title": "Call the bank", "said": ERRAND }])
    );
    assert_eq!(shown["references"]["maybe"][0]["said"], json!("Vet visits"));
    assert_eq!(shown["ask"]["pictures"], json!(["scan-3f9a1c2b.png"]));
    let text = told["content"][0]["text"].as_str().unwrap();
    assert!(
        text.starts_with("Noted.\n\nTisty linked what this names"),
        "{text}"
    );
    assert!(
        text.contains("Not linked, for you to judge: \"Vet visits\" could be"),
        "{text}"
    );
    assert!(
        text.ends_with("as ![What it shows](scan-3f9a1c2b.png)."),
        "the question about a picture goes last: {text}"
    );
}

#[test]
fn nothing_named_leaves_the_answer_as_it_was() {
    let (said, heard) = link("Plain words, nothing named.");
    let answer =
        json!({ "content": [{ "type": "text", "text": "Noted." }], "structuredContent": {} });

    assert_eq!(said, "Plain words, nothing named.");
    assert_eq!(heard.told(answer.clone()), answer);
}
