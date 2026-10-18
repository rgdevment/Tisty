use tisty_markdown::front::{title, titled_as};

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
