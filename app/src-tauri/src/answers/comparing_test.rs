use std::path::Path;

use tisty_core::history::Writer;

use super::{Told, told};

const DOC: &str = "mac0-0001";
const UNO: &str = "# Acta\n\nEl gato come pescado fresco hoy.\n";
const DOS: &str = "# Acta\n\nEl gato come pollo fresco hoy.\n";

fn at(minutes: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + minutes * 60).unwrap()
}

fn written(data: &Path, bodies: &[&str]) {
    let docs = data.join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    for (turn, body) in bodies.iter().enumerate() {
        let by = Writer::at(data, None, at(40 * turn as i64));
        match turn {
            0 => {
                tisty_core::docs::create_named_by(&docs, &by, DOC, body).unwrap();
            }
            _ => {
                tisty_core::docs::write_by(&docs, &by, DOC, body).unwrap();
            }
        }
    }
}

fn print(body: &str) -> String {
    tisty_core::docs::print_of_text(body)
}

#[test]
fn the_window_gets_the_page_and_how_much_went_and_came() {
    let room = tempfile::tempdir().unwrap();
    written(room.path(), &[UNO, DOS]);

    let Told::Drawn { html, gone, came } = told(room.path(), DOC, &print(UNO), None).unwrap()
    else {
        panic!("a short document is drawn");
    };

    assert_eq!((gone, came), (1, 1));
    assert!(html.contains("<del>pescado</del>"), "{html}");
    assert!(html.contains("<ins>pollo</ins>"), "{html}");
}

#[test]
fn what_the_window_reads_is_named_in_its_own_letters() {
    let drawn = Told::Drawn {
        html: "<p>x</p>".into(),
        gone: 1,
        came: 2,
    };

    assert_eq!(
        serde_json::to_value(&drawn).unwrap(),
        serde_json::json!({ "kind": "drawn", "html": "<p>x</p>", "gone": 1, "came": 2 })
    );
    assert_eq!(
        serde_json::to_value(&Told::TooLong).unwrap(),
        serde_json::json!({ "kind": "tooLong" })
    );
}

#[test]
fn a_version_pruned_since_the_list_was_read_is_refused_by_name() {
    let room = tempfile::tempdir().unwrap();
    written(room.path(), &[UNO, DOS]);

    let refused = told(room.path(), DOC, &print("# Otra\n"), None).unwrap_err();

    assert_eq!(refused.code, "versionNotKept");
}

#[test]
fn two_versions_too_long_to_tell_apart_are_an_answer_and_not_a_refusal() {
    let room = tempfile::tempdir().unwrap();
    let long = |said: &str| {
        (0..4_100)
            .map(|at| format!("{said} {at}.\n\n"))
            .collect::<String>()
    };
    let (before, after) = (long("antes"), long("ahora"));
    written(room.path(), &[&before, &after]);

    assert_eq!(
        told(room.path(), DOC, &print(&before), None).unwrap(),
        Told::TooLong
    );
}

#[test]
fn a_document_that_is_gone_is_refused_as_no_such_document() {
    let room = tempfile::tempdir().unwrap();
    written(room.path(), &[UNO, DOS]);
    let docs = room.path().join("docs");
    std::fs::remove_file(tisty_core::docs::resolve(&docs, DOC).unwrap()).unwrap();

    let refused = told(room.path(), DOC, &print(UNO), None).unwrap_err();

    assert_eq!(refused.code, "noSuchDoc");
}
