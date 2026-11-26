fn page(markdown: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(markdown))
}

#[test]
fn an_escaped_emoji_code_stays_as_the_code_that_was_copied() {
    let shown = page(r"Listo \:tada: y \:x: y \::star\::");
    assert!(shown.contains("Listo :tada: y :x: y ::star::"), "{shown:?}");
    assert!(!shown.contains('🎉') && !shown.contains('❌'), "{shown:?}");
}

#[test]
fn an_unescaped_emoji_code_still_draws_its_glyph() {
    let shown = page("Listo :tada: y :x:");
    assert!(shown.contains('🎉') && shown.contains('❌'), "{shown:?}");
}
