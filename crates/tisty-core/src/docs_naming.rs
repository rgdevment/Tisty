use super::{Naming, name_at_end};

fn room() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn wrote(root: &std::path::Path, body: &str) -> String {
    super::create(root, &crate::event::DeviceId("dev_a".to_string()), body)
        .unwrap()
        .id
}

#[test]
fn naming_a_page_leaves_the_step_back_the_person_had() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\nlo que dije\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");
    super::edit(at.path(), at.path(), &book, "lo que dije", "otra cosa").unwrap();
    let kept = super::read_before(at.path(), &book).unwrap();
    let stood = super::before_left_at(at.path(), &book).unwrap();

    name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap();

    assert_eq!(
        super::read_before(at.path(), &book).unwrap(),
        kept,
        "what the person said before is still there"
    );
    assert_ne!(
        super::before_left_at(at.path(), &book).unwrap(),
        stood,
        "and it stands against the body the line is now part of"
    );
    let now = super::read(at.path(), &book).unwrap();
    assert_eq!(
        super::before_left_at(at.path(), &book).unwrap(),
        crate::attach::printed(now.as_bytes()),
        "so going back is still offered"
    );
}

#[test]
fn a_step_back_already_spent_is_not_brought_back_by_naming_a_page() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\nlo que dije\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");
    super::edit(at.path(), at.path(), &book, "lo que dije", "otra cosa").unwrap();
    // What the window's own save does: it writes, and keeps nothing beside the document.
    super::written(at.path(), &book, "# Libro\n\notra cosa\n\ny algo mio\n").unwrap();
    let spent = super::before_left_at(at.path(), &book).unwrap();

    name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap();

    assert_eq!(
        super::before_left_at(at.path(), &book).unwrap(),
        spent,
        "a step back nobody could take is not offered again"
    );
    let now = super::read(at.path(), &book).unwrap();
    assert_ne!(
        super::before_left_at(at.path(), &book).unwrap(),
        crate::attach::printed(now.as_bytes()),
        "so going back still refuses, and nothing of theirs is thrown away"
    );
}

#[test]
fn a_book_with_no_step_back_gains_none_from_being_named_in() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\nintro\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");

    name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap();

    assert!(super::before_left_at(at.path(), &book).is_none());
}

#[test]
fn a_page_is_named_at_the_end_with_the_title_it_carries() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\nintro\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");

    let said = name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap();

    let Naming::Wrote { named, whole } = said else {
        panic!("it had a place to write: {said:?}");
    };
    assert_eq!(named, vec![page.clone()]);
    assert!(
        whole.ends_with(&format!("![Enero](tisty:doc/{page})\n")),
        "{whole:?}"
    );
}

#[test]
fn the_same_page_asked_for_twice_is_named_once() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\nintro\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");

    let said = name_at_end(at.path(), at.path(), &book, &[page.as_str(), page.as_str()]).unwrap();

    let Naming::Wrote { named, whole } = said else {
        panic!("{said:?}");
    };
    assert_eq!(named.len(), 1);
    assert_eq!(whole.matches(&format!("tisty:doc/{page}")).count(), 1);
}

#[test]
fn a_book_ending_inside_a_fence_says_so_and_writes_nothing() {
    let at = room();
    let book = wrote(at.path(), "# Libro\n\n~~~sh\nabierta\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");

    assert_eq!(
        name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap(),
        Naming::Fenced
    );
}

#[test]
fn a_book_with_nothing_to_take_a_title_from_says_it_would_be_renamed() {
    let at = room();
    let book = wrote(at.path(), "\n\n");
    let page = wrote(at.path(), "# Enero\n\nx.\n");

    assert_eq!(
        name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap(),
        Naming::WouldRename
    );
}

#[test]
fn a_page_already_named_leaves_the_book_as_it_was() {
    let at = room();
    let page = wrote(at.path(), "# Enero\n\nx.\n");
    let book = wrote(
        at.path(),
        &format!("# Libro\n\n![Enero](tisty:doc/{page})\n"),
    );

    assert_eq!(
        name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap(),
        Naming::Nothing
    );
}

#[test]
fn a_page_named_only_inside_a_fence_is_named_for_real_as_well() {
    let at = room();
    let page = wrote(at.path(), "# Enero\n\nx.\n");
    let book = wrote(
        at.path(),
        &format!("# Libro\n\n~~~md\n![Enero](tisty:doc/{page})\n~~~\n"),
    );

    let said = name_at_end(at.path(), at.path(), &book, &[page.as_str()]).unwrap();

    let Naming::Wrote { whole, .. } = said else {
        panic!("an example is not a way in: {said:?}");
    };
    assert_eq!(whole.matches(&format!("tisty:doc/{page}")).count(), 2);
}
