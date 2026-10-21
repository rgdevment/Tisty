use super::{mislabeled, near, task_ids};

#[test]
fn titles_sharing_most_of_their_words_are_alike() {
    assert!(near(
        "tisty nube por api",
        "nube por api el camino al 10 10 2026 otra version"
    ));
    assert!(near("login", "login"));
    assert!(!near("nube por api", "recetas de pan"));
    assert!(!near("tisty arquitectura", "tisty nube por api"));
}

#[test]
fn ids_of_tasks_are_read_out_of_the_text() {
    let found = task_ids("see 01M4WM41YSB2Z4BFH6JYE436TP and N-64, not 01m4wm41ysb2z4bfh6jye436tp");

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].to_string(), "01M4WM41YSB2Z4BFH6JYE436TP");
}

#[test]
fn a_page_line_showing_another_name_than_its_title_is_told() {
    let whole =
        "# Plan\n\n![Old name](tisty:doc/aaaa-0001)\n\n![Notes \\[draft\\]](tisty:doc/aaaa-0002)\n";
    let renamed = tisty_core::docs::Card::read_from("# New name\n\nbody\n");
    let kept = tisty_core::docs::Card::read_from("# Notes [draft]\n\nbody\n");
    let found = mislabeled(whole, &[("aaaa-0001", &renamed), ("aaaa-0002", &kept)]);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["doc"], "aaaa-0001");
    assert_eq!(found[0]["shows"], "Old name");
    assert_eq!(found[0]["title"], "New name");
}
