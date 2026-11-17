use super::*;

fn card(target: &str) -> Link {
    Link {
        target: target.into(),
        card: true,
    }
}

fn mention(target: &str) -> Link {
    Link {
        target: target.into(),
        card: false,
    }
}

#[test]
fn a_card_and_a_mention_are_told_apart() {
    let body = "![Trips](tisty:doc/a-0001)\n\nSee [the bank](tisty:task/01ABC) today.";

    assert_eq!(
        links(body),
        vec![card("tisty:doc/a-0001"), mention("tisty:task/01ABC")]
    );
}

#[test]
fn a_link_inside_code_is_an_example() {
    let body =
        "```\n![Trips](tisty:doc/a-0001)\n```\n\n`[x](tisty:doc/a-0002)` and [y](tisty:doc/a-0003)";

    assert_eq!(links(body), vec![mention("tisty:doc/a-0003")]);
}

#[test]
fn a_name_in_double_brackets_is_no_link() {
    assert!(links("Read [[Trips]] first.").is_empty());
}

#[test]
fn the_same_link_twice_is_kept_once() {
    let body = "[a](tisty:doc/a-0001) and [b](tisty:doc/a-0001)";

    assert_eq!(links(body), vec![mention("tisty:doc/a-0001")]);
}

#[test]
fn each_link_is_told_with_where_it_starts_each_time_it_is_written() {
    let body = "# Trips

[a](tisty:doc/a-0001)
```
[x](tisty:doc/a-0002)
```
![b](tisty:doc/a-0001) and [[Bank]]";

    assert_eq!(
        link_spots(body),
        vec![
            (mention("tisty:doc/a-0001"), body.find("[a]").unwrap()),
            (card("tisty:doc/a-0001"), body.find("[b]").unwrap())
        ]
    );
}

#[test]
fn a_link_to_a_document_is_told_apart_from_the_heading_it_lands_on() {
    assert_eq!(doc_aimed("tisty:doc/a-0001"), Some(("a-0001", None)));
    assert_eq!(
        doc_aimed("tisty:doc/a-0001#the-vet"),
        Some(("a-0001", Some("the-vet")))
    );
    assert_eq!(doc_aimed("tisty:doc/a-0001#"), Some(("a-0001", None)));
    assert_eq!(doc_aimed("tisty:task/01ABC"), None);
}
