use super::*;

fn targets(text: &str) -> Vec<String> {
    extract(text).into_iter().map(|one| one.target).collect()
}

#[test]
fn the_prose_around_a_reference_is_what_gives_it_meaning() {
    let found = extract(
        "se corrigió en el ticket [[CUSLEG-3465]], MR en [gitlab](https://gl.example/mr/7)",
    );

    assert_eq!(
        found,
        vec![
            Ref {
                kind: Kind::Doc,
                target: "CUSLEG-3465".into(),
                label: None
            },
            Ref {
                kind: Kind::Link,
                target: "https://gl.example/mr/7".into(),
                label: Some("gitlab".into())
            },
        ]
    );
}

#[test]
fn a_link_is_read_once_and_not_again_as_a_bare_address() {
    assert_eq!(
        targets("[the ticket](https://x.example/1)"),
        ["https://x.example/1"]
    );
}

#[test]
fn an_address_written_plainly_still_counts() {
    assert_eq!(
        targets("mirar https://x.example/1 antes"),
        ["https://x.example/1"]
    );
}

#[test]
fn prose_punctuation_is_not_part_of_the_address() {
    assert_eq!(
        targets("está en https://x.example/1."),
        ["https://x.example/1"]
    );
    assert_eq!(
        targets("(ver https://x.example/1)"),
        ["https://x.example/1"]
    );
    assert_eq!(targets("¿en https://x.example/1?"), ["https://x.example/1"]);
}

#[test]
fn a_bracket_the_address_opened_belongs_to_it() {
    assert_eq!(
        targets("https://en.example.org/wiki/Foo_(bar)"),
        ["https://en.example.org/wiki/Foo_(bar)"]
    );
}

#[test]
fn a_scheme_pointing_nowhere_is_not_a_reference() {
    assert!(targets("escribir https:// y ya").is_empty());
}

#[test]
fn code_is_read_as_text_because_that_is_what_backticks_mean() {
    assert!(targets("usa `[[algo]]` para enlazar").is_empty());
    assert!(targets("``` \n [[algo]] \n ```").is_empty());
    assert_eq!(targets("`sin cerrar [[algo]]"), ["algo"]);
}

#[test]
fn the_same_target_twice_is_one_reference() {
    assert_eq!(targets("[[A]] y otra vez [[A]]"), ["A"]);
}

#[test]
fn a_reference_that_never_closes_is_not_one() {
    assert!(targets("[[sin cerrar").is_empty());
    assert!(targets("[etiqueta](sin cerrar").is_empty());
    assert!(targets("[[]]").is_empty());
    assert!(targets("[etiqueta]()").is_empty());
}

#[test]
fn a_wrapped_destination_loses_its_brackets() {
    assert_eq!(
        targets("![shot](<attachments/ab/cd.png>)"),
        ["attachments/ab/cd.png"]
    );
    assert_eq!(
        targets("[clip](<C:/My Docs/clip (1).mkv>)"),
        ["C:/My Docs/clip (1).mkv"]
    );
}

#[test]
fn a_label_is_optional_and_a_title_is_not_the_target() {
    assert_eq!(
        extract("[](https://x.example/1) y [a](https://y.example/2 \"por qué\")"),
        vec![
            Ref {
                kind: Kind::Link,
                target: "https://x.example/1".into(),
                label: None
            },
            Ref {
                kind: Kind::Link,
                target: "https://y.example/2".into(),
                label: Some("a".into())
            },
        ]
    );
}

#[test]
fn the_documents_a_text_names_come_out_in_the_order_it_names_them() {
    assert_eq!(
        papers(
            "primero ![Uno](tisty:doc/mac0-0002)\n\nluego ![Dos](tisty:doc/mac0-0001)\n\ny https://x.example/"
        ),
        ["mac0-0002", "mac0-0001"]
    );
}

#[test]
fn a_document_named_twice_is_only_counted_where_it_is_first_named() {
    assert_eq!(
        papers("![A](tisty:doc/mac0-0001) ![B](tisty:doc/mac0-0002) ![A](tisty:doc/mac0-0001)"),
        ["mac0-0001", "mac0-0002"]
    );
}

#[test]
fn a_document_named_inside_code_is_not_named_at_all() {
    assert_eq!(papers("`![A](tisty:doc/mac0-0001)`"), Vec::<String>::new());
}

#[test]
fn a_title_with_brackets_is_still_read_back_from_the_card_written_for_it() {
    let said = card("mac0-0010", "Capitulo 1 [borrador]");

    assert_eq!(papers(&said), ["mac0-0010"], "{said}");
}

#[test]
fn what_an_aligned_paragraph_points_at_is_still_pointed_at() {
    let said = "<p style=\"text-align: center\">\
                <a href=\"attachments/ab/nota-1234.pdf\">el plano</a></p>";

    assert_eq!(targets(said), ["attachments/ab/nota-1234.pdf"]);
}

#[test]
fn a_page_linked_from_an_aligned_paragraph_is_reached_but_is_no_chapter() {
    let said = "<p style=\"text-align: center\"><a href=\"tisty:doc/mac0-0010\">Uno</a></p>";

    assert!(
        papers(said).is_empty(),
        "a link is not a card, however it is written"
    );
    assert_eq!(
        extract(said)
            .into_iter()
            .map(|one| one.target)
            .collect::<Vec<_>>(),
        ["tisty:doc/mac0-0010"]
    );
}

#[test]
fn an_angle_in_prose_does_not_swallow_what_comes_after_it() {
    let said = "si a < b mira [el plano](attachments/ab/plano-1234.pdf) y 5 > 3";

    assert_eq!(targets(said), ["attachments/ab/plano-1234.pdf"]);
}

#[test]
fn an_angle_between_two_cards_leaves_both_where_they_are() {
    let said = "![Uno](tisty:doc/mac0-0001)

si a < b

![Dos](tisty:doc/mac0-0002)";

    assert_eq!(papers(said), ["mac0-0001", "mac0-0002"]);
}

#[test]
fn an_address_written_between_angles_is_still_an_address() {
    assert_eq!(
        targets("<https://x.example/one>"),
        ["https://x.example/one"]
    );
}

#[test]
fn a_reference_inside_a_comment_is_still_a_reference() {
    assert_eq!(
        targets("<!-- [x](attachments/ab/x-1111.pdf) -->"),
        ["attachments/ab/x-1111.pdf"]
    );
}

#[test]
fn an_attribute_that_only_ends_in_href_names_nothing() {
    let said = "<img data-href=\"attachments/ab/fantasma-1111.pdf\" alt=\"x\">";

    assert!(targets(said).is_empty(), "{said}");
}

#[test]
fn an_ampersand_written_for_html_is_read_back_as_one() {
    let said = "<p><a href=\"https://x.example/a?one=1&amp;two=2\">x</a></p>";

    assert_eq!(targets(said), ["https://x.example/a?one=1&two=2"]);
}

#[test]
fn a_title_ending_in_a_slash_does_not_escape_the_bracket_that_closes_it() {
    let said = card("mac0-0010", "Rutas C:\\ y mas");

    assert_eq!(papers(&said), ["mac0-0010"], "{said}");
}

#[test]
fn a_label_that_ends_in_a_slash_still_closes_where_it_looks_closed() {
    assert_eq!(
        targets("[C:\\](attachments/ab/f-1111.png)"),
        ["attachments/ab/f-1111.png"]
    );
}

#[test]
fn a_label_holding_a_link_is_still_no_label_at_all() {
    let found = extract("[uno [dos](https://x.example/2)](https://y.example/1)");
    let outer = found
        .iter()
        .find(|one| one.target.contains("y.example"))
        .unwrap();

    assert_eq!(outer.label, None, "the outer brackets label nothing");
}

#[test]
fn accents_do_not_shift_the_scan() {
    assert_eq!(
        targets("añadir ñandú über 🎉 [[mañana]] y https://x.example/ñ"),
        ["mañana", "https://x.example/ñ"]
    );
}
