use tisty_markdown::front::{self, title, titled_as};

#[test]
fn a_title_with_both_quotes_is_written_as_yaml_and_reads_back_as_written() {
    let name = "Mario's \"cosas\"";
    let out = titled_as("---\ntitle: Viejo\n---\n", name).unwrap();
    assert_eq!(out, "---\ntitle: 'Mario''s \"cosas\"'\n---\n");
    assert_eq!(title(&out).as_deref(), Some(name));
}

#[test]
fn a_backslash_in_a_title_is_kept_literal_as_yaml_reads_it() {
    let name = r"C:\new: docs";
    let out = titled_as("---\ntitle: Viejo\n---\n", name).unwrap();
    assert_eq!(out, "---\ntitle: 'C:\\new: docs'\n---\n");
    assert_eq!(title(&out).as_deref(), Some(name));
}

#[test]
fn a_language_is_read_in_its_usual_case() {
    assert_eq!(
        front::lang("---\nlang: ES\n---\n# a").as_deref(),
        Some("es")
    );
    assert_eq!(
        front::lang("---\nlang: pt_br\n---\n# a").as_deref(),
        Some("pt-BR")
    );
    assert_eq!(
        front::lang("---\nLang: \"zh-hant-tw\"\n---\n# a").as_deref(),
        Some("zh-Hant-TW")
    );
    assert_eq!(
        front::lang("---\nlang: es-419\n---\n# a").as_deref(),
        Some("es-419")
    );
}

#[test]
fn what_is_not_a_language_names_none() {
    for said in ["Spanish", "e", "es-", "es-CL-x", "1a", "es CL"] {
        assert_eq!(
            front::lang(&format!("---\nlang: {said}\n---\n# a")),
            None,
            "{said}"
        );
    }
    assert_eq!(front::lang("# lang: es"), None);
}

#[test]
fn setting_a_property_rewrites_its_line_or_adds_it_last_and_leaves_the_rest() {
    assert_eq!(
        front::with_field("---\nlang: es\npaper: letter\n---\n# a\n", "lang", "en"),
        "---\nlang: en\npaper: letter\n---\n# a\n"
    );
    assert_eq!(
        front::with_field("---\npaper: letter\n---\n# a\n", "lang", "en"),
        "---\npaper: letter\nlang: en\n---\n# a\n"
    );
    assert_eq!(
        front::with_field("# a\n", "lang", "en"),
        "---\nlang: en\n---\n# a\n"
    );
    assert_eq!(
        front::with_field("\u{feff}# a\n", "lang", "en"),
        "\u{feff}---\nlang: en\n---\n# a\n"
    );
}

#[test]
fn taking_a_property_out_leaves_the_rest_and_the_block_goes_with_its_last() {
    assert_eq!(
        front::without_field("---\nlang: es\npaper: letter\n---\n# a\n", "paper"),
        "---\nlang: es\n---\n# a\n"
    );
    assert_eq!(
        front::without_field("---\nsheets: on\n---\n\n# a\n", "sheets"),
        "# a\n"
    );
    assert_eq!(front::without_field("# a\n", "paper"), "# a\n");
}

#[test]
fn a_property_taken_out_takes_its_listed_values_with_it() {
    assert_eq!(
        front::without_field(
            "---\ntags: [a]\npaper:\n  - letter\nlang: es\n---\n# a\n",
            "paper"
        ),
        "---\ntags: [a]\nlang: es\n---\n# a\n"
    );
}

#[test]
fn the_paper_and_the_sheets_are_read_as_the_window_reads_them() {
    let laid = front::laid("---\npaper: Letter\nsheets: on\norientation: landscape\n---\n# a");
    assert_eq!(
        laid,
        front::Laid {
            paper: Some("letter"),
            sheets: true,
            landscape: true,
        }
    );
    // Lying across only means something while the sheets are laid, and a paper the window lacks is no paper.
    let loose = front::laid("---\npaper: A0\norientation: landscape\n---\n# a");
    assert_eq!(loose, front::Laid::default());
}

fn limit(name: &str) -> usize {
    let contract: serde_json::Value =
        serde_json::from_str(tisty_contracts::CONTRACT).expect("the contract is JSON");
    contract["contracts"]["front_matter"]["values"][name]
        .as_u64()
        .expect("a limit the contract names") as usize
}

#[test]
fn a_block_is_read_up_to_the_bytes_the_contract_gives_it() {
    let frame = "---\ntitle: a\n#\n---\n".len();
    let sized = |bytes: usize| format!("---\ntitle: a\n#{}\n---\n", "x".repeat(bytes - frame));
    let at_most = limit("block_bytes_at_most");
    assert_eq!(sized(at_most).len(), at_most);
    assert_eq!(title(&sized(at_most)).as_deref(), Some("a"));
    assert_eq!(title(&sized(at_most + 1)), None);
    assert_eq!(title(sized(at_most + 1).trim_end()).as_deref(), Some("a"));
    assert_eq!(title(sized(at_most + 2).trim_end()), None);
}

#[test]
fn a_block_is_read_up_to_the_properties_the_contract_gives_it() {
    let keyed = |keys: usize| {
        let others: String = (1..keys).map(|one| format!("k{one}: v\n")).collect();
        format!("---\ntitle: a\n{others}---\n")
    };
    let at_most = limit("properties_at_most");
    assert_eq!(title(&keyed(at_most)).as_deref(), Some("a"));
    assert_eq!(title(&keyed(at_most + 1)), None);
}

#[test]
fn a_value_is_read_up_to_the_characters_the_contract_gives_it() {
    // One emoji is one character but more than one byte, so counting bytes reads less than the contract says.
    let said = |characters: usize| format!("---\ntitle: 😀{}\n---\n", "x".repeat(characters - 1));
    let at_most = limit("value_characters_at_most");
    assert_eq!(
        title(&said(at_most)).map(|one| one.chars().count()),
        Some(at_most)
    );
    assert_eq!(title(&said(at_most + 1)), None);
}
