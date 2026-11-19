use std::path::{Path, PathBuf};

use super::{Compared, compared};
use crate::history::Writer;

const DOC: &str = "mac0-0001";
const UNO: &str = "# Acta\n\nuno\n";
const DOS: &str = "# Acta\n\ndos\n";
const TRES: &str = "# Acta\n\ntres\n";

fn person(data: &Path, minutes: i64) -> Writer<'_> {
    let at = jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap();
    Writer::at(data, None, at)
}

fn print(body: &str) -> String {
    crate::docs::print_of_text(body)
}

fn desk(bodies: &[&str]) -> (tempfile::TempDir, PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    for (turn, body) in bodies.iter().enumerate() {
        let by = person(room.path(), 40 * turn as i64);
        match turn {
            0 => {
                crate::docs::create_named_by(&docs, &by, DOC, body).unwrap();
            }
            _ => {
                crate::docs::write_by(&docs, &by, DOC, body).unwrap();
            }
        }
    }
    (room, docs)
}

fn told(data: &Path, was: &str, now: Option<&str>) -> tisty_markdown::Compared {
    match compared(data, DOC, &print(was), now.map(print).as_deref()).unwrap() {
        Compared::Told(said) => said,
        other => panic!("a short document is told, and this said {other:?}"),
    }
}

#[test]
fn a_version_is_compared_with_what_the_document_says_today() {
    let (room, _) = desk(&[UNO, DOS, TRES]);

    let said = told(room.path(), UNO, None);

    assert_eq!((said.gone, said.came), (1, 1));
    assert!(
        said.html.contains("data-change=\"gone\">uno</p>"),
        "{}",
        said.html
    );
    assert!(
        said.html.contains("data-change=\"came\">tres</p>"),
        "{}",
        said.html
    );
}

#[test]
fn two_versions_are_compared_with_each_other() {
    let (room, _) = desk(&[UNO, DOS, TRES]);

    let said = told(room.path(), UNO, Some(DOS));

    assert!(
        said.html.contains("data-change=\"gone\">uno</p>"),
        "{}",
        said.html
    );
    assert!(
        said.html.contains("data-change=\"came\">dos</p>"),
        "{}",
        said.html
    );
    assert!(!said.html.contains("tres"), "{}", said.html);
}

#[test]
fn a_version_compared_with_itself_shows_the_page_unmarked() {
    let (room, _) = desk(&[UNO, DOS, TRES]);

    let said = told(room.path(), DOS, Some(DOS));

    assert_eq!((said.gone, said.came), (0, 0));
    assert!(!said.html.contains("data-change"), "{}", said.html);
}

#[test]
fn a_version_this_machine_does_not_keep_is_said_not_kept() {
    let (room, _) = desk(&[UNO, DOS, TRES]);
    let data = room.path();
    let elsewhere = print("# Acta\n\ncuatro\n");

    assert_eq!(
        compared(data, DOC, &elsewhere, None).unwrap(),
        Compared::NotKept
    );
    assert_eq!(
        compared(data, DOC, &print(UNO), Some(&elsewhere)).unwrap(),
        Compared::NotKept
    );
    assert_eq!(
        compared(data, DOC, "../state", None).unwrap(),
        Compared::NotKept
    );
}

#[test]
fn two_versions_too_long_to_tell_apart_say_so() {
    let long = |said: &str| {
        (0..4_100)
            .map(|at| format!("{said} {at}.\n\n"))
            .collect::<String>()
    };
    let (before, after) = (long("antes"), long("ahora"));
    let (room, _) = desk(&[&before, &after]);

    assert_eq!(
        compared(room.path(), DOC, &print(&before), None).unwrap(),
        Compared::TooLong
    );
}

#[test]
fn a_document_that_is_gone_is_an_error_and_not_a_comparison() {
    let (room, docs) = desk(&[UNO, DOS, TRES]);
    std::fs::remove_file(crate::docs::resolve(&docs, DOC).unwrap()).unwrap();

    assert!(compared(room.path(), DOC, &print(UNO), None).is_err());
}

#[test]
fn a_name_that_is_no_document_s_is_an_error_before_any_version_is_read() {
    let (room, _) = desk(&[UNO, DOS, TRES]);

    assert!(compared(room.path(), "../docs/mac0-0001", &print(UNO), None).is_err());
}
