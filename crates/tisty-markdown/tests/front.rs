use tisty_markdown::front::{title, titled_as};

#[test]
fn a_title_with_both_quotes_is_written_as_yaml_and_reads_back_as_written() {
    let name = "Mario's \"cosas\"";
    let out = titled_as("---\ntitle: Viejo\n---\n", name).unwrap();
    assert_eq!(out, "---\ntitle: 'Mario''s \"cosas\"'\n---\n");
    assert_eq!(title(&out).as_deref(), Some(name));
}
