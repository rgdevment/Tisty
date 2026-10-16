use tisty_markdown::{html, read, reference};

fn heads(text: &str) -> Vec<usize> {
    text.lines()
        .filter(|line| line.starts_with('#'))
        .map(|line| line.chars().take_while(|one| *one == '#').count())
        .collect()
}

#[test]
fn both_languages_tell_the_same_language_section_by_section() {
    let english = reference(false);
    let spanish = reference(true);
    assert!(english.starts_with("# "));
    assert!(spanish.starts_with("# "));
    assert_eq!(heads(english), heads(spanish));
    assert!(heads(english).len() > 20);
}

#[test]
fn the_reference_reads_as_a_page_of_its_own() {
    for text in [reference(false), reference(true)] {
        let drawn = html(&read(text));
        assert!(drawn.contains("<h1"));
        assert!(drawn.contains("<table"));
        assert!(!drawn.contains("Markdown-Editor"));
    }
}
