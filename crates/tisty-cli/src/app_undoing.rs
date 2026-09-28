use super::App;
use tisty_core::{Op, Paths, event::Filed, order};
use ulid::Ulid;

fn desk() -> (tempfile::TempDir, Paths) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    (tmp, paths)
}

#[test]
fn settling_a_book_is_not_what_undo_reaches_for() {
    let (_tmp, paths) = desk();
    let mut app = App::at(paths).unwrap();

    let list = Ulid::generate();
    app.commit(Op::ListAdd {
        id: list,
        d: tisty_core::event::ListAdd {
            name: "Casa".into(),
            order: order::first(),
            color: None,
        },
    })
    .unwrap();

    let book = Ulid::generate();
    let pages: Vec<Ulid> = (0..2).map(|_| Ulid::generate()).collect();
    app.commit(Op::DocAdd {
        id: book,
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: None,
            said: None,
            file: "dev_a-0001".into(),
            order: order::first(),
            folder: None,
            page_of: None,
        },
    })
    .unwrap();
    for (n, id) in pages.iter().enumerate() {
        app.commit(Op::DocAdd {
            id: *id,
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                said: None,
                made: None,
                by: None,
                file: format!("dev_a-000{}", n + 2),
                order: order::first(),
                folder: None,
                page_of: Some(book),
            },
        })
        .unwrap();
    }

    app.commit_all(
        pages
            .iter()
            .map(|id| Op::DocMove {
                id: *id,
                d: Filed {
                    folder: None,
                    page_of: None,
                    order: Some(order::first()),
                },
            })
            .collect(),
    )
    .unwrap();

    let reached = app.last_own_change().unwrap();
    assert_eq!(reached.len(), 1, "one event, not the settling batch");
    assert!(
        matches!(reached[0].0.op, Op::DocAdd { .. }),
        "undo reaches the last thing the person did, not the order it settled: {:?}",
        reached[0].0.op
    );
}
