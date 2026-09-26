use super::*;

#[test]
fn a_card_says_what_is_in_a_body_without_anybody_writing_it_down() {
    let card = Card::read_from(
        "# Acta del comite\n\n## El riego\n\nEl riego queda para mayo. El riego es lo de \
         siempre.\n\n![la foto](foto.png)\n\n[el plano](plano.pdf)\n\n```sh\n# not a \
         heading\n```\n",
    );

    assert_eq!(card.title, "Acta del comite");
    assert_eq!(
        card.outline.len(),
        2,
        "the fenced one is code: {:?}",
        card.outline
    );
    assert_eq!(card.outline[1].title, "El riego");
    assert_eq!(card.pictures, 1);
    assert_eq!(card.links, 1);
    assert!(card.words > 10);
    assert!(
        card.keywords.iter().any(|one| one == "riego"),
        "what it leans on: {:?}",
        card.keywords
    );
}

#[test]
fn a_link_is_counted_only_where_a_target_follows_its_label() {
    let card = Card::read_from("- [ ] tarea\n- [x] hecha\n\nver [el plano](tisty:doc/abc)\n");
    assert_eq!(card.links, 1, "a checklist box is not a link");
    assert_eq!(card.pictures, 0);

    let card = Card::read_from("[x] y [z](u) y ![foto](f.png)\n");
    assert_eq!(card.links, 1);
    assert_eq!(card.pictures, 1);

    let card = Card::read_from("[a](b)\n");
    assert_eq!(card.links, 1, "a link on the very first byte");
}

#[test]
fn a_heading_says_where_its_section_ends_and_how_much_it_holds() {
    let card = Card::read_from("# Acta\n\nintro\n\n## Riego\n\nuno\ndos\n\n\n## Porton\n\ntres\n");
    let riego = &card.outline[1];
    assert_eq!(
        (riego.line, riego.to),
        (5, 8),
        "the blank lines before Porton are nobody's"
    );
    assert_eq!(riego.chars, "## Riego\n\nuno\ndos\n".chars().count());
    let porton = &card.outline[2];
    assert_eq!((porton.line, porton.to), (11, 13));
    assert_eq!(
        section_lines(
            "# Acta\n\nintro\n\n## Riego\n\nuno\ndos\n\n\n## Porton\n\ntres\n",
            1
        ),
        Some((5, 8)),
        "the outline and a section read agree on where it ends"
    );
}

#[test]
fn a_card_is_read_again_once_the_file_it_described_is_written() {
    let room = tempfile::tempdir().unwrap();
    let cache = crate::cache::Cache::open(&room.path().join("cache"))
        .unwrap()
        .expect("a cache to remember in");
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("mac0-0001.md"), "# Acta\n\nel riego.\n").unwrap();

    let first = card_of(&docs, Some(&cache), "mac0-0001").unwrap();
    assert_eq!(first.title, "Acta");
    assert_eq!(
        card_of(&docs, Some(&cache), "mac0-0001").unwrap(),
        first,
        "read twice, the same card comes back"
    );

    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(docs.join("mac0-0001.md"), "# Otra acta\n\nel porton.\n").unwrap();
    let now = card_of(&docs, Some(&cache), "mac0-0001").unwrap();
    assert_eq!(
        now.title, "Otra acta",
        "the remembered one was not handed back"
    );
    assert_ne!(now.print, first.print);
}

#[test]
fn asking_for_one_page_of_them_does_not_forget_the_rest() {
    let room = tempfile::tempdir().unwrap();
    let cache = crate::cache::Cache::open(&room.path().join("cache"))
        .unwrap()
        .expect("a cache to remember in");
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    for n in 1..=3 {
        std::fs::write(
            docs.join(format!("mac0-000{n}.md")),
            format!("# Acta {n}\n\nlo que se hablo.\n"),
        )
        .unwrap();
    }
    let all: Vec<String> = (1..=3).map(|n| format!("mac0-000{n}")).collect();
    cards_of(&docs, Some(&cache), &all);

    // A window onto the list, as `docs` hands one back.
    cards_of(&docs, Some(&cache), &all[..1]);

    let stamp = |id: &str| {
        let told = std::fs::metadata(docs.join(format!("{id}.md"))).unwrap();
        (
            told.len(),
            told.modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        )
    };
    assert!(
        cache.card("mac0-0003", stamp("mac0-0003")).is_some(),
        "what was not on the page is still remembered"
    );
}

fn a_room() -> (tempfile::TempDir, std::path::PathBuf, crate::cache::Cache) {
    let room = tempfile::tempdir().unwrap();
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let cache = crate::cache::Cache::open(&room.path().join("cache"))
        .unwrap()
        .expect("a cache to remember in");
    (room, docs, cache)
}

#[test]
fn the_words_of_a_document_are_kept_so_searching_opens_nothing() {
    let (_room, docs, cache) = a_room();
    std::fs::write(
        docs.join("mac0-0001.md"),
        "# Acta del riego\n\nEl riego del patio queda para mayo.\n",
    )
    .unwrap();
    std::fs::write(
        docs.join("mac0-0002.md"),
        "# El porton\n\nSe cambia en abril.\n",
    )
    .unwrap();

    let all: Vec<String> = ["mac0-0001", "mac0-0002"].map(String::from).to_vec();
    cards_of(&docs, Some(&cache), &all);

    let found = sighted(&docs, Some(&cache), "riego", 20, |_| true).expect("the cache answers");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].id, "mac0-0001");

    // Accents typed or not typed decide nothing, the same as walking the files.
    let found = sighted(&docs, Some(&cache), "porton", 20, |_| true).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");

    let none = sighted(&docs, Some(&cache), "camion", 20, |_| true).unwrap();
    assert!(none.is_empty(), "{none:?}");
}

#[test]
fn a_document_the_cache_never_read_is_taken_in_rather_than_left_out() {
    let (_room, docs, cache) = a_room();
    std::fs::write(docs.join("mac0-0001.md"), "# Acta\n\nel riego.\n").unwrap();
    cards_of(&docs, Some(&cache), &["mac0-0001".to_string()]);

    // Another machine's round drops a document in; nothing has read it here yet.
    std::fs::write(docs.join("mac0-0002.md"), "# Otra\n\nel porton.\n").unwrap();

    let found =
        sighted(&docs, Some(&cache), "porton", 20, |_| true).expect("the search still answers");
    assert_eq!(
        found.len(),
        1,
        "the new one is read in rather than missed: {found:?}"
    );
    assert_eq!(found[0].id, "mac0-0002");

    let at = docs.join("mac0-0002.md");
    assert!(
        cache.card("mac0-0002", stamped(&at).unwrap()).is_some(),
        "and it is remembered, so the next search opens nothing"
    );
}

#[test]
fn a_card_kept_before_the_words_were_is_read_again_rather_than_trusted() {
    let (_room, docs, cache) = a_room();
    std::fs::write(docs.join("mac0-0001.md"), "# Acta\n\nel riego.\n").unwrap();
    let at = docs.join("mac0-0001.md");
    let stamp = stamped(&at).unwrap();
    let card = Card::read_from(&read(&docs, "mac0-0001").unwrap());

    cache.note_card("mac0-0001", stamp, &card, "");

    assert!(
        cache.card("mac0-0001", stamp).is_none(),
        "a card with no words behind it is no card at all"
    );
    assert!(card_of(&docs, Some(&cache), "mac0-0001").is_some());
    assert!(
        cache.card("mac0-0001", stamp).is_some(),
        "and it is kept whole"
    );
}

#[test]
fn a_document_that_is_gone_stops_being_remembered() {
    let room = tempfile::tempdir().unwrap();
    let cache = crate::cache::Cache::open(&room.path().join("cache"))
        .unwrap()
        .expect("a cache to remember in");
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("mac0-0001.md"), "# Acta\n\nel riego.\n").unwrap();
    std::fs::write(docs.join("mac0-0002.md"), "# Otra\n\nel porton.\n").unwrap();

    let told = std::fs::metadata(docs.join("mac0-0002.md")).unwrap();
    let stamp = (
        told.len(),
        told.modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let both = ["mac0-0001".to_string(), "mac0-0002".to_string()];
    assert_eq!(cards_of(&docs, Some(&cache), &both).len(), 2);
    assert!(
        cache.card("mac0-0002", stamp).is_some(),
        "it was remembered to begin with"
    );

    std::fs::remove_file(docs.join("mac0-0002.md")).unwrap();
    forget_stray_cards(&docs, Some(&cache));

    assert_eq!(cards_of(&docs, Some(&cache), &both).len(), 1);
    assert!(
        cache.card("mac0-0002", stamp).is_none(),
        "and what it said is forgotten"
    );
}
