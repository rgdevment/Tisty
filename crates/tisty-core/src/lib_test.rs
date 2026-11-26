use super::*;

fn under_home(rest: &str) -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/home/someone".into());
    std::path::Path::new(&home).join(rest).display().to_string()
}

fn fact_named<'a>(facts: &'a [(&'static str, witness::Fact)], key: &str) -> &'a witness::Fact {
    &facts.iter().find(|(name, _)| *name == key).expect(key).1
}

#[test]
fn a_broken_segment_names_its_file_without_naming_the_account() {
    let at = under_home("tisty/store/dev_a3f1/000001.tisty");
    let error = Error::MalformedEvent {
        file: at.clone(),
        line: 35,
        source: serde_json::from_str::<u8>("nonsense").unwrap_err(),
    };

    let facts = error.told();

    assert!(
        matches!(fact_named(&facts, "file"), witness::Fact::Path(_)),
        "a path carried as an id skips the redaction and takes the home directory with it"
    );
    assert!(matches!(
        fact_named(&facts, "line"),
        witness::Fact::Count(35)
    ));
}

#[test]
fn a_torn_segment_names_its_file_the_same_way() {
    let error = Error::TruncatedSegment {
        file: under_home("tisty/store/dev_a3f1/000001.tisty"),
        found: 12,
        declared: Some(20),
    };

    assert!(matches!(
        fact_named(&error.told(), "file"),
        witness::Fact::Path(_)
    ));
}

#[test]
fn every_error_says_which_one_it_is() {
    let facts = Error::NoHomeDirectory.told();

    assert!(matches!(
        fact_named(&facts, "code"),
        witness::Fact::Code("noHomeDirectory")
    ));
}

#[test]
fn a_newer_history_says_whose_it_is() {
    let facts = Error::UnsupportedVersion {
        version: 17,
        device: "dev_f0ztyvwj".into(),
    }
    .told();

    assert!(matches!(
        fact_named(&facts, "device"),
        witness::Fact::Id(id) if id == "dev_f0ztyvwj"
    ));
    assert!(matches!(
        fact_named(&facts, "version"),
        witness::Fact::Count(17)
    ));
}

#[test]
fn an_error_that_is_not_a_held_file_is_not_held_open() {
    let gone = Error::Io(std::io::Error::from(std::io::ErrorKind::NotFound));

    assert!(!gone.held_open());
    assert!(!Error::AlreadyRunning.held_open());
}

#[cfg(windows)]
#[test]
fn a_document_another_program_holds_open_says_so_when_it_cannot_be_written() {
    use std::os::windows::fs::OpenOptionsExt;

    let room = tempfile::tempdir().unwrap();
    let data = room.path();
    let docs = data.join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let id = "mac0-0001";
    let by = history::Writer::now(data, None);
    docs::create_named_by(&docs, &by, id, "# Acta\n\nuno\n").unwrap();
    let file = docs::resolve(&docs, id).unwrap();
    let _held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file)
        .unwrap();

    let refused = docs::write_by(&docs, &by, id, "# Acta\n\ndos\n").unwrap_err();

    assert!(
        refused.held_open(),
        "a file held open by another program was not told apart: {refused:?}"
    );
}

#[cfg(windows)]
fn held_while_sharing(docs: &std::path::Path, by: &history::Writer, share: u32) -> std::fs::File {
    use std::os::windows::fs::OpenOptionsExt;

    docs::create_named_by(
        docs,
        by,
        "mac0-0001",
        "# Acta

uno
",
    )
    .unwrap();
    let file = docs::resolve(docs, "mac0-0001").unwrap();
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(share)
        .open(&file)
        .unwrap()
}

#[cfg(windows)]
#[test]
fn a_document_another_program_shares_for_reading_cannot_be_replaced_and_says_it_is_held_open() {
    for share in [1u32, 3] {
        let room = tempfile::tempdir().unwrap();
        let docs = room.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        let by = history::Writer::now(room.path(), None);
        let _held = held_while_sharing(&docs, &by, share);
        assert_eq!(
            docs::read(&docs, "mac0-0001").unwrap(),
            "# Acta

uno
"
        );

        let by_write = docs::write_by(
            &docs,
            &by,
            "mac0-0001",
            "# Acta

dos
",
        )
        .unwrap_err();
        let print = docs::print_of_text(
            "# Acta

uno
",
        );
        let over = docs::write_over(
            &docs,
            &by,
            "mac0-0001",
            "# Acta

dos
",
            &print,
        )
        .unwrap_err();

        assert!(by_write.held_open(), "share {share}: {by_write:?}");
        assert!(over.held_open(), "share {share}: {over:?}");
    }
}

#[test]
fn access_denied_counts_as_held_open_on_windows_only() {
    let denied = Error::Io(std::io::Error::from_raw_os_error(5));

    assert_eq!(denied.held_open(), cfg!(windows));
}
