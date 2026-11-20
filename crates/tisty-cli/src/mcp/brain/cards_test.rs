use super::*;

fn near(n: usize) -> Value {
    json!({ "id": format!("dev_person-{n:04}"), "kind": "doc", "title": format!("Doc {n}") })
}

#[test]
fn a_card_keeps_eight_of_each_neighbour_and_counts_the_rest() {
    let mut said = Map::new();
    said.insert(
        "cited_by".into(),
        json!((0..20).map(near).collect::<Vec<_>>()),
    );
    said.insert("cites".into(), json!((0..8).map(near).collect::<Vec<_>>()));
    said.insert(
        "decisions".into(),
        json!((0..15).map(|n| json!({ "line": n })).collect::<Vec<_>>()),
    );
    said.insert("decisions_more".into(), json!(48));

    assert!(capped(&mut said));
    assert_eq!(said["cited_by"].as_array().unwrap().len(), NEAR_SHOWN);
    assert_eq!(said["cited_by_more"], 12);
    assert_eq!(said["cites"].as_array().unwrap().len(), 8);
    assert!(said.get("cites_more").is_none());
    assert_eq!(said["decisions"].as_array().unwrap().len(), DECISIONS_SHOWN);
    assert_eq!(said["decisions_more"], 51);

    let mut few = Map::new();
    few.insert("tags".into(), json!(["casa"]));
    assert!(!capped(&mut few));
}

#[test]
fn a_neighbour_is_told_by_title_id_and_state() {
    assert_eq!(
        near_told(&json!({ "id": "01X", "title": "Call the bank", "state": "open" })),
        "\"Call the bank\" [01X] (open)"
    );
    assert_eq!(
        near_told(&json!({ "file": "attachments/a.png", "shows": "a cat" })),
        "\"attachments/a.png\" []"
    );
    assert_eq!(near_told(&json!("casa")), "casa");
    assert_eq!(near_told(&json!(3)), "3");
}

#[test]
fn each_kind_of_link_names_its_own_neighbours() {
    assert_eq!(of_kind("cite"), ["cites", "cited_by"]);
    assert_eq!(of_kind("mention"), ["mentions", "mentioned_by"]);
    assert_eq!(of_kind("tag"), ["tags"]);
    assert_eq!(of_kind("uses"), ["files"]);
    assert!(of_kind("structure").contains(&"part_of"));
    assert!(!of_kind("structure").contains(&"cites"));
    assert_eq!(of_kind("all").len(), 15);
}

#[test]
fn neighbours_are_counted_in_a_line_for_the_text() {
    let mut said = Map::new();
    said.insert("cited_by".into(), json!([near(1), near(2)]));
    said.insert("tags".into(), json!([]));
    assert_eq!(neighbours_told(&said), "\n\nNeighbours: cited by 2.");
    assert_eq!(neighbours_told(&Map::new()), "");
}

#[test]
fn a_guess_is_told_apart_with_the_words_both_share() {
    let mut said = Map::new();
    said.insert(
        "guessed".into(),
        json!([
            { "id": "01A", "title": "Gastos de diciembre", "state": "open", "score": 0.42, "shared": ["presupuesto", "familiar"] },
            { "id": "dev_person-0001", "title": "Budget", "score": 0.91 },
        ]),
    );

    assert_eq!(
        guessed_told(&said),
        "
Guessed alike by Tisty, linked by no one: \"Gastos de diciembre\" [01A] (open) (shares presupuesto, familiar); \"Budget\" [dev_person-0001]."
    );
    assert_eq!(neighbours_told(&said), "", "a guess is no neighbour");
    assert_eq!(guessed_told(&Map::new()), "");
}

#[test]
fn a_guess_by_spelling_or_translation_says_so_instead_of_words_shared() {
    let mut said = Map::new();
    said.insert(
        "guessed".into(),
        json!([
            { "id": "01A", "title": "Pagar la luz", "score": 0.8, "shared": ["pagar≈pay"], "method": "dictionary" },
            { "id": "01B", "title": "Vacunas", "score": 0.7, "shared": ["vacuna≈vacunas"], "method": "typo" },
            { "id": "01C", "title": "Gastos", "score": 0.6, "shared": ["gastos"], "method": "words" },
        ]),
    );

    let told = guessed_told(&said);

    assert!(
        told.contains("[01A] (alike by translation: pagar≈pay)"),
        "{told}"
    );
    assert!(
        told.contains("[01B] (spelled almost the same: vacuna≈vacunas)"),
        "{told}"
    );
    assert!(told.contains("[01C] (shares gastos)"), "{told}");
}
