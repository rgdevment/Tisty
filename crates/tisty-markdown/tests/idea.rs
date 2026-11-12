use tisty_markdown::{IdeaHead, html, idea_head, printed, read};

fn page(text: &str) -> String {
    html(&read(text))
}

#[test]
fn an_idea_tells_its_time_hue_and_side_and_shows_only_its_words() {
    let drawn = page("> [!IDEA] 2026-11-12 09:12 · blue · right\n> Probar con harina integral\n");

    assert!(drawn.contains("data-callout=\"idea\""));
    assert!(drawn.contains("data-when=\"2026-11-12 09:12\""));
    assert!(drawn.contains("data-hue=\"blue\""));
    assert!(drawn.contains("data-side=\"right\""));
    assert!(drawn.contains("Probar con harina integral"));
    assert!(!drawn.contains("[!IDEA]"));
    assert!(
        !drawn.contains("· blue"),
        "the head line is data, not words: {drawn}"
    );
}

#[test]
fn an_idea_without_a_hue_or_a_side_is_a_plain_note_in_the_flow() {
    let drawn = page("> [!idea] 2026-11-12\n> Sin color\n");

    assert!(drawn.contains("data-callout=\"idea\""));
    assert!(drawn.contains("data-when=\"2026-11-12\""));
    assert!(!drawn.contains("data-hue"));
    assert!(!drawn.contains("data-side"));
}

#[test]
fn the_head_takes_its_parts_in_any_order_and_leaves_what_it_does_not_know() {
    let head = idea_head("> [!IDEA] left · Coral · 2026-11-12 18:40 · plaid");

    assert_eq!(
        head,
        IdeaHead {
            when: Some("2026-11-12 18:40"),
            hue: Some("coral"),
            side: Some("left"),
        }
    );
    assert_eq!(
        idea_head("> [!NOTE] 2026-11-12 · blue"),
        IdeaHead::default()
    );
    assert_eq!(idea_head("> [!IDEA]"), IdeaHead::default());
}

#[test]
fn folio_reads_an_idea_as_its_own_kind_of_callout() {
    let json = printed(&read("> [!IDEA] 2026-11-12 · pink\n> Una idea\n"));

    let said = json.to_string();
    assert!(said.contains("\"kind\":\"idea\""), "{json}");
    assert!(said.contains("\"hue\":\"pink\""), "{json}");
    assert!(said.contains("\"when\":\"2026-11-12\""), "{json}");
}
