use super::*;

fn desk() -> (tempfile::TempDir, Paths) {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    (room, paths)
}

fn a_paper(paths: &Paths, name: &str) {
    std::fs::write(paths.docs().join(format!("{name}.md")), b"# Algo").unwrap();
}

#[test]
fn a_paper_the_log_shed_is_taken_out_once_and_not_looked_for_again() {
    let (_room, paths) = desk();
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    let mut done = Already::default();

    assert_eq!(papers(&paths, &state.shed, None, &mut done), 1);
    assert!(!paths.docs().join("dev_a-0001.md").exists());
    assert!(done.papers.contains("dev_a-0001"));

    a_paper(&paths, "dev_a-0001");
    assert_eq!(
        papers(&paths, &state.shed, None, &mut done),
        0,
        "a set that only grows is not walked again"
    );
    assert!(
        paths.docs().join("dev_a-0001.md").exists(),
        "and a file written afterwards under a shed name is left alone"
    );

    state.shed.insert("dev_a-0002".into());
    a_paper(&paths, "dev_a-0002");
    assert_eq!(
        papers(&paths, &state.shed, None, &mut done),
        1,
        "only the new one"
    );
}

#[test]
fn nothing_retired_means_nobody_reads_every_document_to_find_out() {
    let (_room, paths) = desk();
    let state = State::default();
    let mut done = Already::default();
    let asked = std::cell::Cell::new(false);

    let gone = attachments(
        &paths,
        &state.retired,
        None,
        || {
            asked.set(true);
            Vec::new()
        },
        &mut done,
    );

    assert_eq!(gone, 0);
    assert!(
        !asked.get(),
        "reading every body to answer a question nobody asked is the whole cost"
    );
}

#[test]
fn an_attachment_something_still_names_is_left_and_asked_about_again() {
    let (_room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    let shelf = paths.data().join("attachments/ab");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join("una-a3f90001.png"), b"unos bytes").unwrap();

    let mut state = State::default();
    state.retired.insert(at.into());
    let mut done = Already::default();

    assert_eq!(
        attachments(
            &paths,
            &state.retired,
            None,
            || vec![at.to_string()],
            &mut done
        ),
        0
    );
    assert!(shelf.join("una-a3f90001.png").exists());
    assert!(
        done.attachments.is_empty(),
        "it was not taken out, so it is asked about again when the document goes"
    );

    assert_eq!(
        attachments(&paths, &state.retired, None, Vec::new, &mut done),
        1,
        "and once nothing names it, it goes"
    );
    assert!(done.attachments.contains(at));
}

fn cached(room: &tempfile::TempDir) -> crate::cache::Cache {
    crate::cache::Cache::open(&room.path().join("cache"))
        .unwrap()
        .expect("a cache opens")
}

#[test]
fn what_was_taken_out_is_remembered_across_a_reading() {
    let (room, paths) = desk();
    let cache = cached(&room);
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());

    let swept = all_of_it(&paths, &state, Some(&cache), None, true);
    assert_eq!(swept.papers, 1);
    assert!(swept.any());

    a_paper(&paths, "dev_a-0001");
    let again = all_of_it(&paths, &state, Some(&cache), None, false);
    assert_eq!(again.papers, 0, "the mark outlived the call");
    assert!(!again.any());
    assert!(paths.docs().join("dev_a-0001.md").exists());
}

#[test]
fn without_a_cache_it_still_sweeps_and_simply_forgets() {
    let (_room, paths) = desk();
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());

    assert_eq!(all_of_it(&paths, &state, None, None, false).papers, 1);
    a_paper(&paths, "dev_a-0001");
    assert_eq!(
        all_of_it(&paths, &state, None, None, false).papers,
        1,
        "nothing remembers, so it looks again"
    );
}

#[test]
fn a_body_that_arrived_saying_another_order_is_settled_in_one_batch() {
    let (_room, paths) = desk();
    let book = ulid::Ulid::generate();
    let mut state = State::default();
    let mut kept = |id, file: &str, order: &str, up| {
        state.docs.insert(
            id,
            crate::model::Kept {
                born_by: None,
                guest: false,
                made: None,
                made_by: None,
                wrote_by: None,
                by: None,
                title: None,
                bytes: None,
                wrote: None,
                tags: Vec::new(),
                id,
                file: file.into(),
                order: order.into(),
                folder: None,
                page_of: up,
                archived: false,
                locked: false,
                edited_by: None,
                flagged: None,
                folder_was: None,
            },
        );
    };
    kept(book, "dev_a-0001", "V", None);
    let one = ulid::Ulid::generate();
    let two = ulid::Ulid::generate();
    kept(one, "dev_a-0002", "V", Some(book));
    kept(two, "dev_a-0003", "W", Some(book));

    std::fs::write(
        paths.docs().join("dev_a-0001.md"),
        "# Libro

![dos](tisty:doc/dev_a-0003)

![uno](tisty:doc/dev_a-0002)
",
    )
    .unwrap();

    let told = settling_what_arrived(&paths, &state, &["dev_a-0001".to_string()]);
    assert_eq!(told.len(), 1, "only the one that has to move");

    assert!(
        settling_what_arrived(&paths, &state, &["dev_a-0002".to_string()]).is_empty(),
        "a page arriving moves nothing: the order lives in the book"
    );
    assert!(
        settling_what_arrived(&paths, &state, &[]).is_empty(),
        "and nothing arriving reads nothing"
    );
}

#[test]
fn a_file_that_would_not_go_is_looked_for_again_next_time() {
    let (_room, paths) = desk();
    let at = paths.docs().join("dev_a-0001.md");
    std::fs::create_dir_all(&at).unwrap();
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    let mut done = Already::default();

    assert_eq!(
        papers(&paths, &state.shed, None, &mut done),
        0,
        "it would not go"
    );
    assert!(
        done.papers.is_empty(),
        "so it is not written off: the next opening has to try again"
    );

    std::fs::remove_dir(&at).unwrap();
    std::fs::write(&at, b"# Algo").unwrap();
    assert_eq!(
        papers(&paths, &state.shed, None, &mut done),
        1,
        "and then it goes"
    );
    assert!(done.papers.contains("dev_a-0001"));
}

#[test]
fn a_shared_folder_that_is_not_there_is_never_mistaken_for_a_tidy_one() {
    let (room, paths) = desk();
    let away = room.path().join("nowhere");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    let mut done = Already::default();

    papers(&paths, &state.shed, Some(&away), &mut done);
    assert!(
        done.papers_up.is_empty(),
        "an unmounted drive looks exactly like an empty one, so nothing is written off"
    );

    std::fs::create_dir_all(away.join("docs")).unwrap();
    papers(&paths, &state.shed, Some(&away), &mut done);
    assert!(
        done.papers_up.contains("dev_a-0001"),
        "once it is there, it counts"
    );
}

#[test]
fn an_attachment_put_back_on_a_task_while_the_folders_are_walked_is_not_taken_out() {
    let (_room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    let shelf = paths.data().join("attachments/ab");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join("una-a3f90001.png"), b"unos bytes").unwrap();

    let mut state = State::default();
    state.retired.insert(at.into());

    let walked = Sweeping::of(&paths, &state, None, None, false).walk();

    let mut back = state.clone();
    let id = ulid::Ulid::generate();
    let mut task = crate::model::Task::new(id, "el plano", "a0");
    task.description = Some(format!("![una](<{at}>)"));
    back.tasks.insert(id, task);

    let (swept, done) = walked.with(&back);

    assert_eq!(swept.attachments, 0);
    assert!(
        shelf.join("una-a3f90001.png").exists(),
        "it was put back on a task while the folders were being walked"
    );
    assert!(done.attachments.is_empty());
}

#[test]
fn what_went_before_there_was_a_folder_still_reaches_the_folder_afterwards() {
    let (room, paths) = desk();
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    let mut done = Already::default();

    assert_eq!(papers(&paths, &state.shed, None, &mut done), 1);
    assert!(done.papers.contains("dev_a-0001"));

    let later = room.path().join("shared");
    std::fs::create_dir_all(later.join("docs")).unwrap();
    let theirs = later.join("docs").join("dev_a-0001.md");
    std::fs::write(&theirs, b"# Algo").unwrap();

    assert_eq!(
        papers(&paths, &state.shed, Some(&later), &mut done),
        1,
        "a folder set up afterwards never heard what was deleted before it"
    );
    assert!(!theirs.exists());
    assert!(done.papers_up.contains("dev_a-0001"));
}

#[test]
fn an_attachment_retired_before_there_was_a_folder_still_reaches_the_folder_afterwards() {
    let (room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    for root in [paths.data().to_path_buf(), room.path().join("shared")] {
        std::fs::create_dir_all(root.join("attachments/ab")).unwrap();
        std::fs::write(root.join(at), b"unos bytes").unwrap();
    }
    let mut state = State::default();
    state.retired.insert(at.into());
    let mut done = Already::default();

    assert_eq!(
        attachments(&paths, &state.retired, None, Vec::new, &mut done),
        1
    );
    assert!(done.attachments.contains(at));

    let later = room.path().join("shared");
    assert_eq!(
        attachments(&paths, &state.retired, Some(&later), Vec::new, &mut done),
        1,
        "a folder set up afterwards kept what this machine had already retired"
    );
    assert!(!later.join(at).exists());
    assert!(done.attachments_up.contains(at));
}

#[test]
fn a_store_with_no_folder_does_not_walk_the_same_deletions_every_time() {
    let (_room, paths) = desk();
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    state
        .retired
        .insert("attachments/ab/una-a3f90001.png".into());
    let mut done = Already::default();

    papers(&paths, &state.shed, None, &mut done);
    attachments(&paths, &state.retired, None, Vec::new, &mut done);

    let asked = std::cell::Cell::new(false);
    let gone = attachments(
        &paths,
        &state.retired,
        None,
        || {
            asked.set(true);
            Vec::new()
        },
        &mut done,
    );

    assert_eq!(gone, 0);
    assert_eq!(papers(&paths, &state.shed, None, &mut done), 0);
    assert!(
        !asked.get(),
        "with nowhere else to reach, what is gone from here is gone, and nobody reads the documents again"
    );
}

#[test]
fn an_attachment_only_a_document_names_survives_the_walk() {
    let (_room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    let shelf = paths.data().join("attachments/ab");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join("una-a3f90001.png"), b"unos bytes").unwrap();
    a_paper(&paths, "dev_a-0001");
    std::fs::write(
        paths.docs().join("dev_a-0001.md"),
        format!("# Algo\n\n![una]({at})\n"),
    )
    .unwrap();

    let mut state = State::default();
    state.retired.insert(at.into());
    let (swept, _) = Sweeping::of(&paths, &state, None, None, false)
        .walk()
        .with(&state);

    assert_eq!(swept.attachments, 0);
    assert!(
        shelf.join("una-a3f90001.png").is_file(),
        "the walk never read the documents, so what one of them names was taken out"
    );
}

#[test]
fn an_attachment_something_still_names_is_not_written_off_for_being_away() {
    let (_room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    let mut state = State::default();
    state.retired.insert(at.into());
    let mut done = Already::default();

    attachments(
        &paths,
        &state.retired,
        None,
        || vec![at.to_string()],
        &mut done,
    );

    assert!(
        done.attachments.is_empty(),
        "it is still named, so being away from this machine settles nothing"
    );
}

#[test]
fn a_deletion_that_reached_one_folder_is_still_owed_to_the_next_one() {
    let (room, paths) = desk();
    a_paper(&paths, "dev_a-0001");
    let mut state = State::default();
    state.shed.insert("dev_a-0001".into());
    let mut done = Already::default();

    let first = room.path().join("one");
    std::fs::create_dir_all(first.join("docs")).unwrap();
    std::fs::write(first.join("docs").join("dev_a-0001.md"), b"# Algo").unwrap();
    assert_eq!(papers(&paths, &state.shed, Some(&first), &mut done), 2);
    assert!(done.papers_up.contains("dev_a-0001"));

    let second = room.path().join("two");
    std::fs::create_dir_all(second.join("docs")).unwrap();
    let theirs = second.join("docs").join("dev_a-0001.md");
    std::fs::write(&theirs, b"# Algo").unwrap();
    done.facing(Some(&second));

    assert_eq!(
        papers(&paths, &state.shed, Some(&second), &mut done),
        1,
        "the next folder was told nothing about what the last one already took out"
    );
    assert!(!theirs.exists());
}

#[test]
fn a_folder_to_reach_never_makes_the_walk_forget_to_read_the_documents() {
    let (room, paths) = desk();
    let at = "attachments/ab/una-a3f90001.png";
    let shelf = paths.data().join("attachments/ab");
    std::fs::create_dir_all(&shelf).unwrap();
    let mut state = State::default();
    state.retired.insert(at.into());
    std::fs::write(shelf.join("una-a3f90001.png"), b"unos bytes").unwrap();
    a_paper(&paths, "dev_a-0001");
    std::fs::write(
        paths.docs().join("dev_a-0001.md"),
        format!("# Algo\n\n![una]({at})\n"),
    )
    .unwrap();

    let later = room.path().join("shared");
    std::fs::create_dir_all(&later).unwrap();
    let (swept, after) = Sweeping::of(&paths, &state, None, Some(&later), false)
        .walk()
        .with(&state);

    assert_eq!(swept.attachments, 0);
    assert!(
        shelf.join("una-a3f90001.png").is_file(),
        "the walk asked the folder about it without reading what still names it"
    );
    assert!(after.attachments_up.is_empty());
}
