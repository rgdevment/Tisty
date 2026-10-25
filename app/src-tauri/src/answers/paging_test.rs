use super::{DRAWN_BY, Diagrams, KEPT_AT_MOST, paged};

#[test]
fn a_page_is_drawn_once_and_then_read_from_the_shelf() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    let first = paged(&shelf, "# Title\n\nWords.\n", &Diagrams::default());
    assert!(first.contains("Title</h1>"));
    let kept: Vec<_> = std::fs::read_dir(shelf.join(DRAWN_BY))
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(kept.len(), 1);
    std::fs::write(kept[0].path(), "kept").unwrap();
    assert_eq!(
        paged(&shelf, "# Title\n\nWords.\n", &Diagrams::default()),
        "kept"
    );
}

#[test]
fn a_diagram_is_drawn_once_while_the_words_around_it_change() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    let fence = "graph TD\n  A --> B\n";
    let diagrams = Diagrams::default();
    diagrams.0.lock().unwrap().drawn.insert(
        tisty_core::attach::printed(fence.as_bytes()),
        Some("<svg>kept</svg>".into()),
    );
    for words in ["Before.", "After."] {
        let page = paged(
            &shelf,
            &format!("{words}\n\n```mermaid\n{fence}```\n"),
            &diagrams,
        );
        assert!(page.contains("kept"), "{page}");
    }
    assert_eq!(diagrams.0.lock().unwrap().drawn.len(), 1);
}

#[test]
fn the_diagrams_kept_are_the_latest_so_many() {
    let diagrams = Diagrams::default();
    for n in 0..super::DIAGRAMS_AT_MOST + 2 {
        diagrams.drawn(&format!("graph TD\n  A{n} --> B\n"));
    }
    let kept = diagrams.0.lock().unwrap();
    assert_eq!(kept.drawn.len(), super::DIAGRAMS_AT_MOST);
    assert_eq!(kept.order.len(), super::DIAGRAMS_AT_MOST);
    let latest = format!("graph TD\n  A{} --> B\n", super::DIAGRAMS_AT_MOST + 1);
    assert!(
        kept.drawn
            .contains_key(&tisty_core::attach::printed(latest.as_bytes()))
    );
    let first = tisty_core::attach::printed("graph TD\n  A0 --> B\n".as_bytes());
    assert!(!kept.drawn.contains_key(&first));
}

#[test]
fn pages_drawn_by_another_version_are_put_away() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    std::fs::create_dir_all(shelf.join("0.0.1")).unwrap();
    paged(&shelf, "Words.\n", &Diagrams::default());
    assert!(!shelf.join("0.0.1").exists());
    assert!(shelf.join(DRAWN_BY).exists());
}

#[test]
fn the_shelf_keeps_only_so_many_pages() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    for n in 0..KEPT_AT_MOST + 3 {
        paged(&shelf, &format!("Page {n}.\n"), &Diagrams::default());
    }
    let kept = std::fs::read_dir(shelf.join(DRAWN_BY)).unwrap().count();
    assert!(kept <= KEPT_AT_MOST);
}

#[test]
fn a_guide_nobody_answers_for_is_not_taken_as_the_guide() {
    assert_eq!(super::published("http://127.0.0.1:9/llms-full.txt"), None);
    assert!(
        super::GUIDE
            .iter()
            .all(|at| at.starts_with("https://markdown.tisty.app/"))
    );
}
