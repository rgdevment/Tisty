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
