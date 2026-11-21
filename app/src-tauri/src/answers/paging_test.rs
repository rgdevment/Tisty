use super::{Check, DRAWN_BY, KEPT_AT_MOST, Placing, canon, paged, placed_near, table_edit};

use tisty_markdown::grid::Edit;

fn check(text: &str, hash: u32) -> Check {
    Check {
        bytes: text.len(),
        hash: [hash, !hash],
    }
}

fn key(was: Check, from: usize, to: usize, insert: &str) -> super::Edit {
    super::Edit {
        was,
        from,
        to,
        insert: insert.to_string(),
    }
}

#[test]
fn what_the_page_sends_is_read_as_a_key_or_a_whole_body() {
    let edit = r#"{"edit":{"from":3,"to":3,"insert":"s","was":{"bytes":3,"hash":[1,2]}}}"#;
    let Ok(super::Sent::Edit(key)) = serde_json::from_str::<super::Sent>(edit) else {
        panic!("{edit} is not read as a key");
    };
    assert_eq!((key.from, key.to, key.insert.as_str()), (3, 3, "s"));
    assert_eq!(
        key.was,
        Check {
            bytes: 3,
            hash: [1, 2]
        }
    );
    let body = r#"{"body":"Unos"}"#;
    assert!(matches!(
        serde_json::from_str::<super::Sent>(body),
        Ok(super::Sent::Body(text)) if text == "Unos"
    ));
}

#[test]
fn what_the_page_is_answered_names_the_count_the_sum_and_the_blocks_drawn() {
    let doc = tisty_markdown::read("One.\n\nTwo.\n\nThree.\n\nFour.\n");
    let near = placed_near(&doc, 7, 8, Some(doc.labels()));
    let said = serde_json::to_value(&near).unwrap_or_default();
    let mut keys: Vec<&str> = said
        .as_object()
        .map(|one| one.keys().map(String::as_str).collect())
        .unwrap_or_default();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["count", "from", "heads", "html", "sum", "to", "whole"]
    );
    assert!(said["heads"].is_null());
    assert_eq!(said["whole"], false);
    assert_eq!(
        (&said["count"], &said["from"], &said["to"]),
        (&4.into(), &0.into(), &3.into())
    );
    let last = &doc.blocks()[3].at;
    assert_eq!(
        said["sum"],
        serde_json::json!(tisty_markdown::spans_sum([[last.start, last.end]]))
    );
    let headed = tisty_markdown::read("# Uno\n\nA.\n\nB.\n\n# Uno\n");
    let said =
        serde_json::to_value(placed_near(&headed, 2, 3, Some(headed.labels()))).unwrap_or_default();
    assert_eq!(
        said["heads"],
        serde_json::json!(tisty_markdown::names_sum(["uno-1"]))
    );
}

#[test]
fn keys_sent_alone_draw_the_page_as_the_whole_text_would() {
    let placing = Placing::default();
    let mut text = "# Uno\n\nAño con ñandú 😀.\n\n- a\n- b\r\n\n```\nfn x() {}\n```\n".to_string();
    let mut was = check(&text, 1);
    placing.read("main", &text, was, false, |_, _| ());
    let keys = [
        ("Uno", "Unos"),
        ("😀", "😁"),
        ("ñandú ", ""),
        ("- b", "- b\n- c"),
        ("\r\n", "\n"),
        ("fn x", "fn y"),
        ("# ", ""),
    ];
    for (hash, (gone, insert)) in (2..).zip(keys) {
        let from = text
            .find(gone)
            .unwrap_or_else(|| panic!("{gone:?} in {text:?}"));
        let next = format!("{}{insert}{}", &text[..from], &text[from + gone.len()..]);
        let now = check(&next, hash);
        let typed = key(was, from, from + gone.len(), insert);
        let drawn =
            |doc: &tisty_markdown::Doc, shown| placed_near(doc, from, from + insert.len(), shown);
        let shown = Some(tisty_markdown::read(&text).labels());
        let whole = drawn(&tisty_markdown::read(&next), shown);
        assert_eq!(placing.edited("main", &typed, now, drawn), Some(whole));
        text = next;
        was = now;
    }
}

#[test]
fn a_key_typed_on_another_text_is_not_applied_and_the_held_text_stays() {
    let placing = Placing::default();
    let text = "Año.\n";
    let was = check(text, 7);
    placing.read("main", text, was, false, |_, _| ());
    let now = check("Año!.\n", 8);
    let at = text.find('.').unwrap_or_default();
    let typed = |was: Check, from: usize, insert: &str| key(was, from, from, insert);
    let said = |doc: &tisty_markdown::Doc, _| doc.text().to_string();
    assert_eq!(
        placing.edited("main", &typed(check(text, 9), at, "!"), now, said),
        None
    );
    assert_eq!(
        placing.edited("quick", &typed(was, at, "!"), now, said),
        None
    );
    assert_eq!(placing.edited("main", &typed(was, 2, "!"), now, said), None);
    assert_eq!(
        placing.edited("main", &typed(was, at, "!!"), now, said),
        None
    );
    assert_eq!(
        placing.edited("main", &typed(was, at, "!"), now, said),
        Some("Año!.\n".to_string())
    );
    placing.forget("main");
    assert_eq!(
        placing.edited("main", &typed(now, at, "?"), now, said),
        None
    );
}

#[test]
fn a_key_that_defines_a_link_far_from_the_blocks_it_draws_asks_for_the_whole_page() {
    let placing = Placing::default();
    let text = "Ver [x].\n\nA.\n\nB.\n\nC.\n\n";
    let was = check(text, 1);
    placing.read("main", text, was, false, |_, _| ());
    let (end, insert) = (text.len(), "[x]: https://a.b\n");
    let next = format!("{text}{insert}");
    let now = check(&next, 2);
    let drawn = |doc: &tisty_markdown::Doc, shown| placed_near(doc, end, next.len(), shown);
    let defined = placing.edited("main", &key(was, end, end, insert), now, drawn);
    assert!(defined.is_some_and(|near| near.whole && near.from > 0));
    let at = next.find("B.").unwrap_or_default();
    let typed = |doc: &tisty_markdown::Doc, shown| placed_near(doc, at, at + 1, shown).whole;
    let after = check(&format!("{}s{}", &next[..at], &next[at..]), 3);
    let laid = placing.edited("main", &key(now, at, at, "s"), after, typed);
    assert_eq!(laid, Some(false));
}

#[test]
fn a_whole_body_with_no_page_held_to_weigh_it_against_asks_for_the_whole_page() {
    let placing = Placing::default();
    let text = "Uno.\n\nDos.\n";
    let drawn = |doc: &tisty_markdown::Doc, shown| placed_near(doc, 0, 1, shown).whole;
    assert!(placing.read("main", text, check(text, 1), true, drawn));
    assert!(!placing.read("main", text, check(text, 2), true, drawn));
}

#[test]
fn a_whole_body_that_defines_a_link_asks_for_the_whole_page() {
    let placing = Placing::default();
    let text = "Ver [x].\n\nA.\n\nB.\n";
    placing.read("main", text, check(text, 1), false, |_, _| ());
    let defined = format!("{text}\n[x]: https://a.b\n");
    let end = text.len();
    let drawn =
        |doc: &tisty_markdown::Doc, shown| placed_near(doc, end, defined.len(), shown).whole;
    assert!(placing.read("main", &defined, check(&defined, 2), true, drawn));
    let typed = defined.replacen("A.", "As.", 1);
    let at = typed.find("As.").unwrap_or_default() + 1;
    let laid = |doc: &tisty_markdown::Doc, shown| placed_near(doc, at, at + 1, shown).whole;
    assert!(!placing.read("main", &typed, check(&typed, 3), true, laid));
}

fn falls_over(_: &tisty_markdown::Doc, _: Option<[u32; 2]>) -> String {
    panic!("the drawing fell over")
}

#[test]
fn a_page_that_breaks_while_drawn_is_refused_and_the_next_key_sends_the_whole_body() {
    let placing = Placing::default();
    let text = "Uno.\n";
    let was = check(text, 1);
    placing.read("main", text, was, false, |_, _| ());
    let next = "Unos.\n";
    let now = check(next, 2);
    let keyed = crate::caught("doc_placed_near", || {
        placing.edited("main", &key(was, 3, 3, "s"), now, falls_over)
    });
    assert!(keyed.is_err_and(|refused| refused.code == "internal"));
    let said = |doc: &tisty_markdown::Doc, _| doc.text().to_string();
    let typed = key(now, 4, 4, "!");
    let after = check("Unos!.\n", 3);
    assert_eq!(placing.edited("main", &typed, after, said), None);
    let whole = crate::caught("doc_placed_near", || {
        placing.read("main", next, now, true, falls_over)
    });
    assert!(whole.is_err_and(|refused| refused.code == "internal"));
    assert_eq!(placing.edited("main", &typed, after, said), None);
    assert_eq!(placing.read("main", next, now, true, said), next);
    assert_eq!(
        placing.edited("main", &typed, after, said),
        Some("Unos!.\n".to_string())
    );
}

#[test]
fn every_order_that_draws_answers_even_when_its_drawing_breaks() {
    let said = std::fs::read_to_string("src/answers/paging.rs").expect("the orders that draw");
    let orders: Vec<&str> = said.split("#[tauri::command(async)]").skip(1).collect();
    let loose: Vec<&str> = orders
        .iter()
        .map(|order| order.split("\n}\n").next().unwrap_or_default())
        .filter(|order| !order.contains("caught(") && !order.contains("tabled("))
        .filter_map(|order| order.lines().find(|line| line.starts_with("pub fn ")))
        .collect();

    assert!(orders.len() > 10, "the orders that draw moved elsewhere");
    assert!(
        loose.is_empty(),
        "an order that draws leaves the page waiting forever if it breaks: {loose:?}"
    );
}

#[test]
fn no_order_anywhere_hands_back_a_drawing_that_cannot_say_it_broke() {
    let mut bare = Vec::new();
    for entry in std::fs::read_dir("src/answers").expect("the answers") {
        let path = entry.expect("an answer").path();
        if path.extension().is_none_or(|one| one != "rs") {
            continue;
        }
        let said = std::fs::read_to_string(&path).expect("an answer's orders");
        for order in said.split("#[tauri::command(async)]").skip(1) {
            let head = order.split('{').next().unwrap_or_default();
            if head.contains("-> tauri::ipc::Response") {
                let name = head.lines().find(|line| line.starts_with("pub fn "));
                bare.push(format!("{}: {}", path.display(), name.unwrap_or_default()));
            }
        }
    }
    assert!(
        bare.is_empty(),
        "an order that draws leaves the page waiting forever if it breaks: {bare:?}"
    );
}

#[test]
fn a_date_typed_as_it_is_said_lands_as_its_day() {
    let now: jiff::Zoned = "2026-10-28T10:00:00[America/Santiago]".parse().unwrap();
    assert_eq!(canon("date", "2026-11-14", &now, "es"), "2026-11-14");
    assert_eq!(canon("date", "mañana", &now, "es"), "2026-10-29");
    assert_eq!(canon("date", "tomorrow", &now, "en"), "2026-10-29");
    assert_eq!(canon("date", "pronto", &now, "es"), "pronto");
    assert_eq!(canon("money CLP", "$6.490", &now, "es"), "6490");
}

#[test]
fn a_table_is_edited_where_the_page_drew_it_and_nowhere_else() {
    let body = "Antes.\n\n| a |\n| --- |\n| 1 |\n";
    let asked = || Edit::Cell {
        row: 0,
        column: 0,
        said: "2".to_string(),
    };
    assert_eq!(
        table_edit(body.to_string(), (8, 28), vec![asked()]),
        Ok("Antes.\n\n| a |\n| --- |\n| 2 |\n".to_string())
    );
    assert_eq!(
        table_edit(body.to_string(), (0, 6), vec![asked()]),
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
