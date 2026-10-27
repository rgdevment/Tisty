use super::{DRAWN_BY, KEPT_AT_MOST, paged, table_edit};
use tisty_markdown::grid::Edit;

#[test]
fn a_table_is_edited_where_the_page_drew_it_and_nowhere_else() {
    let body = "Antes.\n\n| a |\n| --- |\n| 1 |\n";
    let asked = || Edit::Cell {
        row: 0,
        column: 0,
        said: "2".to_string(),
    };
    assert_eq!(
        table_edit(body.to_string(), (8, 28), asked()),
        Ok("Antes.\n\n| a |\n| --- |\n| 2 |\n".to_string())
    );
    assert_eq!(
        table_edit(body.to_string(), (0, 6), asked()),
        Err("moved".to_string())
    );
}

#[test]
fn a_page_is_drawn_once_and_then_read_from_the_shelf() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    let first = paged(&shelf, "# Title\n\nWords.\n");
    assert!(first.contains("Title</h1>"));
    let kept: Vec<_> = std::fs::read_dir(shelf.join(DRAWN_BY))
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(kept.len(), 1);
    std::fs::write(kept[0].path(), "kept").unwrap();
    assert_eq!(paged(&shelf, "# Title\n\nWords.\n"), "kept");
}

#[test]
fn pages_drawn_by_another_version_are_put_away() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    std::fs::create_dir_all(shelf.join("0.0.1")).unwrap();
    paged(&shelf, "Words.\n");
    assert!(!shelf.join("0.0.1").exists());
    assert!(shelf.join(DRAWN_BY).exists());
}

#[test]
fn the_shelf_keeps_only_so_many_pages() {
    let tmp = tempfile::tempdir().unwrap();
    let shelf = tmp.path().join("pages");
    for n in 0..KEPT_AT_MOST + 3 {
        paged(&shelf, &format!("Page {n}.\n"));
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
