use super::*;

#[test]
fn every_page_has_its_twin_in_the_other_language() {
    assert_eq!(EN.len(), ES.len());
    for (mine, theirs) in [(EN, ES), (ES, EN)] {
        for one in mine {
            assert!(
                theirs
                    .iter()
                    .any(|other| other.page == one.twin && other.twin == one.page),
                "{} has no twin",
                one.page
            );
        }
    }
}

#[test]
fn every_page_says_something_under_its_own_title() {
    for one in EN.iter().chain(ES) {
        assert!(!one.title().is_empty(), "{} has no title", one.page);
        assert!(
            one.readable(false).trim().len() > 80,
            "{} says almost nothing",
            one.page
        );
    }
}

#[test]
fn a_page_is_named_as_tisty_app_names_it() {
    assert_eq!(slugify("Instalación"), "instalacion");
    assert_eq!(
        slugify("Qué-puede-hacer-un-asistente"),
        "que-puede-hacer-un-asistente"
    );
    assert_eq!(slugify("Keyboard-Shortcuts"), "keyboard-shortcuts");
    assert_eq!(slugify("Páginas-y-PDF"), "paginas-y-pdf");
    assert_eq!(found(true, "").map(|one| one.page), Some("Inicio"));
    assert_eq!(
        found(false, "sync-and-backup").map(|one| one.page),
        Some("Sync-and-Backup")
    );
}

#[test]
fn every_link_between_pages_lands_on_one() {
    for spanish in [false, true] {
        for one in pages(spanish) {
            let read = one.readable(spanish);
            assert!(!read.contains("[["), "{} keeps a wiki link", one.page);
            for link in read.split(SCHEME).skip(1) {
                let slug = link.split([')', '#']).next().unwrap_or_default();
                assert!(
                    found(spanish, slug).is_some(),
                    "{} links to {slug:?}",
                    one.page
                );
            }
        }
    }
}

#[test]
fn a_picture_is_kept_as_one_the_window_can_show_or_stand_in_for() {
    let read = readable(
        "# Title\n\n<img src=\"https://example.invalid/a.png\" alt=\"Tisty\"/>\n",
        false,
    );
    assert_eq!(read.trim(), "![Tisty](https://example.invalid/a.png)");
    let all = pictures(true);
    assert!(!all.is_empty());
    assert!(all.iter().all(|url| url.starts_with("https://")));
    let weightless: Vec<&String> = all
        .iter()
        .filter(|url| weight(url).is_none_or(|bytes| bytes == 0))
        .collect();
    assert!(weightless.is_empty(), "no weight for {weightless:?}");
}
