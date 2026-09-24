use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.cli(&["algo mio"]);
        served.cli(&["agent", "--on"]);
        served
    }

    fn command(&self) -> Command {
        let root = self.home.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_tisty"));
        command
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL");
        command
    }

    fn cli(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn call(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let said = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        self.talk(&[
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": "2025-06-18", "capabilities": {},
                            "clientInfo": { "name": "test", "version": "1" } },
            })
            .to_string(),
            &said,
        ])
        .into_iter()
        .find(|one| one["id"] == 2)
        .unwrap()
    }

    fn talk(&self, said: &[&str]) -> Vec<serde_json::Value> {
        let mut child = self
            .command()
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        {
            let mut asks = child.stdin.take().unwrap();
            for one in said {
                writeln!(asks, "{one}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    fn wrote(&self, body: &str, page_of: Option<&str>) -> String {
        let args = match page_of {
            Some(up) => serde_json::json!({ "body": body, "page_of": up }),
            None => serde_json::json!({ "body": body }),
        };
        self.call("write_doc", args)["result"]["structuredContent"]["doc"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn body_of(&self, doc: &str) -> String {
        self.call("read_doc", serde_json::json!({ "doc": doc }))["result"]["structuredContent"]
            ["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    fn print_of(&self, doc: &str) -> String {
        self.call("read_doc", serde_json::json!({ "doc": doc }))["result"]["structuredContent"]
            ["print"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    fn pages_of(&self, doc: &str) -> Vec<String> {
        let said = self.call("read_doc", serde_json::json!({ "doc": doc }));
        said["result"]["structuredContent"]["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one.as_str().unwrap().to_string())
            .collect()
    }

    fn data(&self) -> std::path::PathBuf {
        self.home.path().join("data")
    }
}

fn copied(from: &std::path::Path, into: &std::path::Path) {
    std::fs::create_dir_all(into).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let at = into.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copied(&entry.path(), &at);
        } else {
            std::fs::copy(entry.path(), at).unwrap();
        }
    }
}

#[test]
fn a_page_written_by_an_assistant_is_named_at_the_end_of_its_document() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este año.", None);
    let page = served.wrote("# Marzo\n\nlo que se dijo.", Some(&book));

    let body = served.body_of(&book);
    assert!(
        body.contains(&format!("![Marzo](tisty:doc/{page})")),
        "the document has to say where the page goes: {body}"
    );
    assert!(
        body.starts_with("# Actas\n\nlas de este año."),
        "and nothing that was written before may be touched: {body}"
    );
}

#[test]
fn a_page_titled_with_brackets_still_sits_where_its_document_names_it() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este ano.", None);
    let one = served.wrote("# Capitulo 1 [borrador]", Some(&book));
    let two = served.wrote("# Capitulo 2", Some(&book));

    assert_eq!(
        served.pages_of(&book),
        vec![one, two],
        "a title with brackets must not shove the page to the end"
    );
}

#[test]
fn moving_the_line_that_names_a_page_moves_the_page() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let two = served.wrote("# Abril", Some(&book));

    assert_eq!(served.pages_of(&book), vec![one.clone(), two.clone()]);

    let said = served.call(
        "edit_doc",
        serde_json::json!({
            "doc": book,
            "old": format!("![Marzo](tisty:doc/{one})\n\n![Abril](tisty:doc/{two})"),
            "new": format!("![Abril](tisty:doc/{two})\n\n![Marzo](tisty:doc/{one})"),
        }),
    );

    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![two, one]);
}

#[test]
fn hanging_a_document_as_a_page_with_page_doc_lands_it_last_until_the_text_names_it() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let loose = served.wrote("# Suelto\n\nun documento aparte.", None);

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": loose, "page_of": book }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![one, loose.clone()]);

    let body = served.body_of(&book);
    assert!(
        !body.contains(&loose),
        "hanging it as a page does not by itself name it in the text: {body}"
    );
}

#[test]
fn naming_a_hung_page_in_the_text_with_edit_doc_moves_it_from_the_end_to_where_it_is_named() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let two = served.wrote("# Abril", Some(&book));
    let loose = served.wrote("# Enero\n\nun documento aparte.", None);

    served.call(
        "page_doc",
        serde_json::json!({ "doc": loose, "page_of": book }),
    );
    assert_eq!(
        served.pages_of(&book),
        vec![one.clone(), two.clone(), loose.clone()]
    );

    let old = format!("![Marzo](tisty:doc/{one})\n\n");
    let new = format!("![Enero](tisty:doc/{loose})\n\n{old}");
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": book, "old": old, "new": new }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![loose, one, two]);
}

#[test]
fn append_doc_settles_the_order_of_two_pages_that_were_never_named_before() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let a = served.wrote("# Suelto A\n\ncontenido.", None);
    let b = served.wrote("# Suelto B\n\ncontenido.", None);
    served.call("page_doc", serde_json::json!({ "doc": a, "page_of": book }));
    served.call("page_doc", serde_json::json!({ "doc": b, "page_of": book }));

    assert_eq!(
        served.pages_of(&book),
        vec![one.clone(), a.clone(), b.clone()]
    );

    let said = served.call(
        "append_doc",
        serde_json::json!({
            "doc": book,
            "body": format!("![B](tisty:doc/{b})\n\n![A](tisty:doc/{a})\n"),
        }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![one, b, a]);
}

#[test]
fn a_page_order_pulled_in_from_another_machine_settles_to_match_this_machines_own_text_on_the_next_write()
 {
    let here = Served::new();
    let book = here.wrote("# Actas\n\nde este ano.", None);
    let one = here.wrote("# Marzo", Some(&book));
    let two = here.wrote("# Abril", Some(&book));
    assert_eq!(here.pages_of(&book), vec![one.clone(), two.clone()]);

    // A second machine pulls this store, then swaps the two pages on its own, offline.
    let there = Served::new();
    copied(&here.data().join("docs"), &there.data().join("docs"));
    copied(&here.data().join("store"), &there.data().join("store"));
    let old = format!("![Marzo](tisty:doc/{one})\n\n![Abril](tisty:doc/{two})");
    let new = format!("![Abril](tisty:doc/{two})\n\n![Marzo](tisty:doc/{one})");
    let said = there.call(
        "edit_doc",
        serde_json::json!({ "doc": book, "old": old, "new": new }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(there.pages_of(&book), vec![two.clone(), one.clone()]);

    // Pulling that swap back into the first machine's own store, without touching its own copy
    // of the document's text, leaves the tree and the visible text disagreeing about the order.
    for device in std::fs::read_dir(there.data().join("store")).unwrap() {
        let device = device.unwrap();
        if !device.path().is_dir() {
            continue;
        }
        copied(
            &device.path(),
            &here.data().join("store").join(device.file_name()),
        );
    }
    assert_eq!(
        here.pages_of(&book),
        vec![two.clone(), one.clone()],
        "the log now carries the other machine's move"
    );
    assert_eq!(
        here.body_of(&book),
        format!("# Actas\n\nde este ano.\n\n{old}\n"),
        "but this machine's own file on disk still reads the way it always did"
    );

    // The next write on this machine settles the order back to what its own text says.
    let said = here.call(
        "append_doc",
        serde_json::json!({ "doc": book, "body": "Fin." }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(
        here.pages_of(&book),
        vec![one, two],
        "saving settles the order back to what the text in front of the person says"
    );
}

impl Served {
    fn bolt(&self, doc: &str) {
        let store = self.data().join("store");
        let events = tisty_core::store::read_all(&store).unwrap();
        let state = tisty_core::State::replay(&events);
        let kept = state.docs.values().find(|one| one.file == doc).unwrap();
        let device = events.last().unwrap().device.clone();
        let mut open = tisty_core::Store::open(&store, device).unwrap();
        open.append(tisty_core::Op::DocLock { id: kept.id })
            .unwrap();
    }

    fn complained(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!out.status.success(), "the command went through");
        String::from_utf8_lossy(&out.stderr).into_owned()
    }

    fn said(&self, name: &str, args: serde_json::Value) -> String {
        let told = self.call(name, args);
        assert!(told["result"]["isError"].as_bool() != Some(true), "{told}");
        told["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    fn refused(&self, name: &str, args: serde_json::Value) -> String {
        let said = self.call(name, args);
        assert_eq!(
            said["result"]["isError"].as_bool(),
            Some(true),
            "{name} went through: {said}"
        );
        said["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

#[test]
fn a_locked_document_turns_away_every_way_an_assistant_has_of_writing() {
    let served = Served::new();
    let book = served.wrote("# Minuta\n\nlo que dije", None);
    served.bolt(&book);

    for (name, args) in [
        (
            "append_doc",
            serde_json::json!({ "doc": &book, "body": "y algo mas" }),
        ),
        (
            "edit_doc",
            serde_json::json!({ "doc": &book, "old": "lo que dije", "new": "otra cosa" }),
        ),
    ] {
        let why = served.refused(name, args);
        assert!(why.contains("locked"), "{name} said: {why}");
    }
    assert_eq!(served.body_of(&book).trim_end(), "# Minuta\n\nlo que dije");
}

#[test]
fn a_locked_book_gains_no_page_and_keeps_the_ones_it_has() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let loose = served.wrote("# Suelto", None);
    served.bolt(&book);

    let why = served.refused(
        "write_doc",
        serde_json::json!({ "body": "# Clase dos", "page_of": &book }),
    );
    assert!(why.contains("locked"), "{why}");

    let why = served.refused(
        "page_doc",
        serde_json::json!({ "doc": &loose, "page_of": &book }),
    );
    assert!(why.contains("locked"), "{why}");

    let why = served.refused("page_doc", serde_json::json!({ "doc": &page }));
    assert!(why.contains("locked"), "{why}");
    assert_eq!(served.pages_of(&book), vec![page]);
}

#[test]
fn a_page_of_a_locked_book_is_shut_as_tightly_as_the_book() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let page = served.wrote("# Clase uno", Some(&book));
    served.bolt(&book);

    let why = served.refused(
        "append_doc",
        serde_json::json!({ "doc": &page, "body": "y algo mas" }),
    );

    assert!(why.contains("locked"), "{why}");
    assert!(
        served.call("read_doc", serde_json::json!({ "doc": &page }))["result"]["structuredContent"]
            ["locked"]
            .as_bool()
            .unwrap(),
        "read_doc has to say so before an assistant tries"
    );
}

#[test]
fn the_terminal_puts_no_file_into_a_locked_document_either() {
    let served = Served::new();
    let book = served.wrote(
        "# Minuta

lo que dije",
        None,
    );
    served.bolt(&book);
    let at = served.home.path().join("nota.txt");
    std::fs::write(&at, b"algo").unwrap();

    let why = served.complained(&["attach", &book, at.to_str().unwrap()]);

    assert!(why.contains("locked"), "{why}");
    assert_eq!(
        served.body_of(&book).trim_end(),
        "# Minuta

lo que dije"
    );
}

#[test]
fn rewriting_a_document_leaves_none_of_its_pages_with_nothing_pointing_at_it() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let print =
        served.call("read_doc", serde_json::json!({ "doc": &book }))["result"]["structuredContent"]
            ["print"]
            .as_str()
            .unwrap()
            .to_string();

    let said = served.call(
        "write_doc",
        serde_json::json!({ "doc": &book, "print": print, "body": "# Curso\n\notra cosa" }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![page.clone()]);
    let body = served.body_of(&book);
    assert!(
        body.contains(&format!("tisty:doc/{page}")),
        "the page was left with nothing pointing at it: {body}"
    );
    assert!(body.contains("otra cosa"), "what was sent is still there");
}

#[test]
fn an_edit_that_takes_out_the_line_naming_a_page_says_which_page_it_left_loose() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\n## Notas\n\nlo que hay", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let print = served.call("outline_doc", serde_json::json!({ "doc": &book }))["result"]
        ["structuredContent"]["print"]
        .as_str()
        .unwrap()
        .to_string();

    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &book, "section": 1, "new": "## Notas\n\notra cosa\n", "print": print }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let told = said["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        told.contains("Clase uno") && told.contains("nothing in this document points there now"),
        "an edit that leaves a page unnamed has to say so: {told}"
    );
    assert_eq!(
        said["result"]["structuredContent"]["loose"],
        serde_json::json!([page]),
        "{said}"
    );
    assert_eq!(
        served.pages_of(&book),
        vec![page],
        "the page itself is not lost, only the line naming it"
    );
}

#[test]
fn an_edit_that_moves_the_line_naming_a_page_leaves_nothing_loose() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nuno\n\ndos", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let card = format!("![Clase uno](tisty:doc/{page})");

    let said = served.call(
        "edit_doc",
        serde_json::json!({
            "doc": &book,
            "old": format!("uno\n\ndos\n\n{card}"),
            "new": format!("uno\n\n{card}\n\ndos"),
        }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let told = said["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        !told.contains("points there now"),
        "moving the line is not leaving it loose: {told}"
    );
    assert_eq!(
        said["result"]["structuredContent"]["loose"],
        serde_json::json!([]),
        "{said}"
    );
}

#[test]
fn a_page_is_moved_before_another_in_one_call_and_the_reading_order_follows() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    let one = served.wrote("# Clase uno", Some(&book));
    let two = served.wrote("# Clase dos", Some(&book));
    let three = served.wrote("# Clase tres", Some(&book));

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &three, "page_of": &book, "before": &one }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(
        served.pages_of(&book),
        vec![three.clone(), one.clone(), two.clone()],
        "the page did not move"
    );
    let body = served.body_of(&book);
    let at = |id: &str| body.find(&format!("tisty:doc/{id}")).unwrap();
    assert!(at(&three) < at(&one) && at(&one) < at(&two), "{body}");
    assert_eq!(
        body.matches(&format!("tisty:doc/{three}")).count(),
        1,
        "the line was copied rather than moved: {body}"
    );
}

#[test]
fn a_page_hung_and_placed_in_one_call_is_named_where_it_was_asked_for() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    let one = served.wrote("# Clase uno", Some(&book));
    let loose = served.wrote("# Clase suelta", None);

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &loose, "page_of": &book, "before": &one }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![loose.clone(), one.clone()]);
    assert!(
        served
            .body_of(&book)
            .contains(&format!("tisty:doc/{loose}")),
        "a page hung with `before` is named in the body"
    );
}

#[test]
fn a_page_whose_line_carries_words_of_their_own_is_not_moved_over_them() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    let one = served.wrote("# Clase uno", Some(&book));
    let two = served.wrote("# Clase dos", Some(&book));
    let print =
        served.call("read_doc", serde_json::json!({ "doc": &book }))["result"]["structuredContent"]
            ["print"]
            .as_str()
            .unwrap()
            .to_string();
    let mine = format!(
        "# Curso\n\n![Clase uno](tisty:doc/{one})\n\nLa puerta se ve en ![Clase dos](tisty:doc/{two}) y ahi se explica.\n"
    );
    served.call(
        "write_doc",
        serde_json::json!({ "doc": &book, "print": print, "body": &mine }),
    );

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &two, "page_of": &book, "before": &one }),
    );

    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let body = served.body_of(&book);
    assert!(
        body.contains("La puerta se ve en") && body.contains("y ahi se explica"),
        "the words around the line were carried off with it: {body}"
    );
}

fn shaped(served: &Served, body: &str) -> (String, Vec<String>) {
    let book = served.wrote("# Libro\n\nintro", None);
    let pages: Vec<String> = (0..3)
        .map(|n| served.wrote(&format!("# Cap {n}\n\nx."), Some(&book)))
        .collect();
    let print =
        served.call("read_doc", serde_json::json!({ "doc": &book }))["result"]["structuredContent"]
            ["print"]
            .as_str()
            .unwrap()
            .to_string();
    let mine = body
        .replace("{0}", &pages[0])
        .replace("{1}", &pages[1])
        .replace("{2}", &pages[2]);
    let said = served.call(
        "write_doc",
        serde_json::json!({ "doc": &book, "print": print, "body": mine }),
    );
    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    (book, pages)
}

#[test]
fn a_line_naming_two_pages_is_not_pulled_apart_to_move_one_of_them() {
    let served = Served::new();
    let (book, pages) = shaped(
        &served,
        "# Libro\n\nintro\n\n![A](tisty:doc/{0}) y ![B](tisty:doc/{1})\n\n![C](tisty:doc/{2})\n",
    );

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &pages[1], "page_of": &book, "after": &pages[2] }),
    );

    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let body = served.body_of(&book);
    assert!(
        body.contains(&format!("tisty:doc/{}", pages[0])) && body.contains(" y "),
        "the other page on that line went with it: {body}"
    );
}

#[test]
fn a_page_named_inside_a_table_is_not_a_place_to_write_beside() {
    let served = Served::new();
    let (book, pages) = shaped(
        &served,
        "# Libro\n\n| uno | dos |\n| --- | --- |\n| ![A](tisty:doc/{0}) | ok |\n| otra | ya |\n\n![B](tisty:doc/{1})\n\n![C](tisty:doc/{2})\n",
    );

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &pages[1], "page_of": &book, "after": &pages[0] }),
    );

    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert!(
        served.body_of(&book).contains("| otra | ya |"),
        "the table was broken in two: {}",
        served.body_of(&book)
    );
}

#[test]
fn a_page_named_the_other_ways_markdown_allows_is_moved_and_not_copied() {
    for body in [
        "# Libro\n\nintro\n\n![A](<tisty:doc/{0}>)\n\n![B](tisty:doc/{1})\n\n![C](tisty:doc/{2})\n",
        "# Libro\n\nintro\n\n![A](tisty:doc/{0} \"la clase\")\n\n![B](tisty:doc/{1})\n\n![C](tisty:doc/{2})\n",
    ] {
        let served = Served::new();
        let (book, pages) = shaped(&served, body);

        let said = served.call(
            "page_doc",
            serde_json::json!({ "doc": &pages[0], "page_of": &book, "after": &pages[2] }),
        );

        assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
        let now = served.body_of(&book);
        assert_eq!(
            now.matches(&format!("tisty:doc/{}", pages[0])).count(),
            1,
            "the line was copied rather than moved: {now}"
        );
        assert_eq!(
            served.pages_of(&book),
            vec![pages[1].clone(), pages[2].clone(), pages[0].clone()],
            "{now}"
        );
    }
}

#[test]
fn a_page_named_on_the_first_line_is_not_written_above() {
    let served = Served::new();
    let (book, pages) = shaped(
        &served,
        "![A](tisty:doc/{0})\n\n![B](tisty:doc/{1})\n\n![C](tisty:doc/{2})\n",
    );

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &pages[1], "page_of": &book, "before": &pages[0] }),
    );

    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert!(
        served
            .body_of(&book)
            .starts_with(&format!("![A](tisty:doc/{}", pages[0])),
        "the document would have been given a new title"
    );
}

#[test]
fn a_page_moved_after_another_lands_on_the_far_side_of_it() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let one = served.wrote("# Uno", Some(&book));
    let two = served.wrote("# Dos", Some(&book));
    let three = served.wrote("# Tres", Some(&book));

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &one, "page_of": &book, "after": &three }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![two, three, one]);
}

#[test]
fn a_book_whose_pages_no_line_names_is_put_in_order_a_page_at_a_time() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let loose: Vec<String> = (0..3)
        .map(|n| {
            let one = served.wrote(&format!("# Cap {n}\n\nx."), None);
            served.call(
                "page_doc",
                serde_json::json!({ "doc": &one, "page_of": &book }),
            );
            one
        })
        .collect();
    assert!(
        !served.body_of(&book).contains("tisty:doc/"),
        "no line names any of them to begin with"
    );

    for one in &loose {
        let said = served.call(
            "page_doc",
            serde_json::json!({ "doc": one, "page_of": &book, "at": "last" }),
        );
        assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    }

    assert_eq!(served.pages_of(&book), loose);
    let body = served.body_of(&book);
    for one in &loose {
        assert_eq!(
            body.matches(&format!("tisty:doc/{one}")).count(),
            1,
            "{body}"
        );
    }
    assert!(
        body.starts_with("# Libro"),
        "the title was left alone: {body}"
    );
}

#[test]
fn a_page_sent_first_goes_before_every_page_the_document_names() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let one = served.wrote("# Uno", Some(&book));
    let two = served.wrote("# Dos", Some(&book));
    let three = served.wrote("# Tres", Some(&book));

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &three, "page_of": &book, "at": "first" }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![three, one, two]);
}

#[test]
fn a_place_that_is_neither_first_nor_last_is_refused_and_so_are_two_at_once() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let one = served.wrote("# Uno", Some(&book));
    let two = served.wrote("# Dos", Some(&book));

    for args in [
        serde_json::json!({ "doc": &two, "page_of": &book, "at": "middle" }),
        serde_json::json!({ "doc": &two, "page_of": &book, "at": "last", "after": &one }),
    ] {
        let said = served.call("page_doc", args.clone());
        assert_eq!(
            said["result"]["isError"].as_bool(),
            Some(true),
            "this had to be refused: {args} gave {said}"
        );
    }
    assert_eq!(served.pages_of(&book), vec![one, two]);
}

#[test]
fn a_page_named_beside_nothing_is_refused_with_an_empty_name() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let one = served.wrote("# Uno", Some(&book));

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &one, "page_of": &book, "before": "" }),
    );

    assert_eq!(
        said["result"]["isError"].as_bool(),
        Some(true),
        "an empty name is a name that was sent: {said}"
    );
}

#[test]
fn placing_a_page_beside_one_that_is_not_a_page_of_the_same_document_is_refused() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let other = served.wrote("# Otro", None);
    let one = served.wrote("# Clase uno", Some(&book));

    for args in [
        serde_json::json!({ "doc": &one, "page_of": &book, "after": &other }),
        serde_json::json!({ "doc": &one, "page_of": &book, "after": &one }),
        serde_json::json!({ "doc": &one, "after": &one }),
        serde_json::json!({ "doc": [&one, &book], "page_of": &book, "after": &one }),
    ] {
        let said = served.call("page_doc", args.clone());
        assert_eq!(
            said["result"]["isError"].as_bool(),
            Some(true),
            "this had to be refused: {args} gave {said}"
        );
    }
    assert_eq!(served.pages_of(&book), vec![one]);
}

#[test]
fn a_body_that_names_its_pages_itself_is_written_exactly_as_it_was_sent() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let print =
        served.call("read_doc", serde_json::json!({ "doc": &book }))["result"]["structuredContent"]
            ["print"]
            .as_str()
            .unwrap()
            .to_string();
    let mine = format!("# Curso\n\n![Clase uno](tisty:doc/{page})\n\nal final");

    served.call(
        "write_doc",
        serde_json::json!({ "doc": &book, "print": print, "body": &mine }),
    );

    assert_eq!(served.body_of(&book).trim_end(), mine);
}

#[test]
fn a_document_put_away_by_an_agent_comes_back_the_same_way() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    let page = served.wrote("# Clase uno", Some(&book));

    let said = served.call("archive_doc", serde_json::json!({ "doc": &book }));
    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");

    let listed = served.call("docs", serde_json::json!({ "scope": "open" }));
    let open = serde_json::to_string(&listed).unwrap();
    assert!(!open.contains(&book), "it is still listed as open: {open}");
    assert!(!open.contains(&page), "its page stayed out: {open}");

    let back = served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": false }),
    );
    assert_ne!(back["result"]["isError"].as_bool(), Some(true), "{back}");
    let listed = served.call("docs", serde_json::json!({ "scope": "open" }));
    assert!(serde_json::to_string(&listed).unwrap().contains(&book));
    let body = served.body_of(&book);
    assert!(body.starts_with("# Curso"), "{body}");
    assert!(body.contains("lo que hay"), "{body}");
    assert!(body.contains(&format!("tisty:doc/{page}")), "{body}");
}

#[test]
fn a_page_is_put_away_on_its_own_and_the_book_stays_open() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let page = served.wrote("# Clase uno", Some(&book));
    let other = served.wrote("# Clase dos", Some(&book));

    let said = served.call("archive_doc", serde_json::json!({ "doc": &page }));
    assert!(said["result"]["isError"].is_null(), "{said}");

    let listed = served.call("docs", serde_json::json!({ "scope": "open" }));
    let here: Vec<String> = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["doc"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(here.contains(&book), "the book stays open: {here:?}");
    assert!(here.contains(&other), "so does the page nobody touched");
    assert!(!here.contains(&page), "only the one named went away");
}

fn on_disk(named: &str, body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join(named);
    std::fs::write(&at, body).unwrap();
    (dir, at)
}

#[test]
fn a_file_from_another_app_comes_in_tidied_and_says_what_changed() {
    let served = Served::new();
    let (_dir, at) = on_disk(
        "acta.md",
        "---\ntitle: Acta\n---\n\n# Acta\n\nUn <b>fuerte</b> y <!-- oculto --> y &amp; final.\n",
    );

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(body.contains("**fuerte**"), "{body}");
    assert!(!body.contains("<b>"), "{body}");
    assert!(!body.contains("oculto"), "{body}");
    assert!(body.contains("& final"), "{body}");
    let changed = said["result"]["structuredContent"]["changed"].to_string();
    assert!(changed.contains("front matter"), "{changed}");
    assert!(changed.contains("HTML comments"), "{changed}");
    assert_eq!(
        std::fs::read_to_string(&at).unwrap().lines().next(),
        Some("---"),
        "the file on disk is not touched"
    );
}

#[test]
fn a_file_with_no_title_of_its_own_is_named_after_itself() {
    let served = Served::new();
    let (_dir, at) = on_disk("Notas de la reunion.md", "lo que se dijo\n");

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    assert!(
        served.body_of(doc).starts_with("# Notas de la reunion"),
        "{}",
        served.body_of(doc)
    );
}

#[test]
fn nothing_that_is_not_markdown_comes_in_this_way() {
    let served = Served::new();
    let (_dir, at) = on_disk("clave.txt", "no soy markdown");

    let why = served.refused(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert!(why.contains("not markdown"), "{why}");
}

#[test]
fn a_file_outside_the_places_an_assistant_may_reach_stays_where_it_is() {
    let served = Served::new();
    let at = served.data().join("docs");
    std::fs::create_dir_all(&at).unwrap();
    let at = at.join("mio.md");
    std::fs::write(&at, "# Mio").unwrap();

    let why = served.refused(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert!(why.contains("may take files from"), "{why}");
}

#[test]
fn a_document_goes_out_to_a_folder_and_nothing_here_changes() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay", None);
    served.wrote("# Clase uno", Some(&book));
    let out = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();

    let said = served.call(
        "export_doc",
        serde_json::json!({ "doc": &book, "into": out.path().to_str().unwrap() }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let mut found = Vec::new();
    let mut walk = vec![out.path().to_path_buf()];
    while let Some(at) = walk.pop() {
        for one in std::fs::read_dir(&at).unwrap().flatten() {
            let path = one.path();
            if path.is_dir() {
                walk.push(path);
            } else {
                found.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }
    assert_eq!(
        found.iter().filter(|one| one.ends_with(".md")).count(),
        2,
        "the cover and its page: {found:?}"
    );
    assert!(served.body_of(&book).contains("lo que hay"));
}

#[test]
fn a_key_renamed_as_markdown_comes_in_under_a_warning() {
    let served = Served::new();
    let (_dir, at) = on_disk(
        "inocente.md",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n-----END RSA PRIVATE KEY-----\n",
    );

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(body.contains("[!CAUTION]"), "sin aviso: {body}");
    assert!(body.contains("BEGIN RSA PRIVATE KEY"), "no entro: {body}");
}

#[test]
fn a_guide_that_documents_its_environment_variables_comes_in() {
    let served = Served::new();
    let (_dir, at) = on_disk(
        "Que instalar.md",
        "# Que instalar

La configuracion queda asi:

```bash
FOO_URL=\"https://ejemplo.cl/x\"
FOO_VALOR=\"«REDACTADO»\"
FOO_CLIENT_KEY=\"$FOO_CLIENT_KEY\"
FOO_TOKEN=\"REDACTADO\"
```

| Variable | Valor |
| --- | --- |
| FOO_KEY | pendiente |
",
    );

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    assert!(served.body_of(doc).contains("FOO_CLIENT_KEY"));
}

#[test]
fn a_file_past_what_tisty_opens_is_turned_away_before_it_is_read() {
    let served = Served::new();
    let (_dir, at) = on_disk("enorme.md", &"a".repeat(600 * 1024));

    let why = served.refused(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert!(why.contains("past the 512000 a file may be"), "{why}");
}

#[test]
fn control_characters_do_not_walk_in_through_a_file() {
    let served = Served::new();
    let (_dir, at) = on_disk("escapes.md", "# Uno\n\nantes \u{1b}[31m rojo\n");

    let why = served.refused(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert!(why.contains("control characters"), "{why}");
}

#[test]
fn the_print_a_write_hands_back_is_the_one_the_document_reads_at() {
    let served = Served::new();
    let said = served.call(
        "write_doc",
        serde_json::json!({ "body": "# Sin salto\n\nsin newline final" }),
    );
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let told = said["result"]["structuredContent"]["print"]
        .as_str()
        .unwrap()
        .to_string();

    let back = served.call("read_doc", serde_json::json!({ "doc": doc }));
    assert_eq!(
        back["result"]["structuredContent"]["print"].as_str(),
        Some(told.as_str()),
        "a print that does not match disk turns the next write into a false conflict"
    );

    let again = served.call(
        "write_doc",
        serde_json::json!({ "doc": doc, "print": told, "body": "# Sin salto\n\notra cosa" }),
    );
    assert_ne!(again["result"]["isError"].as_bool(), Some(true), "{again}");
}

#[test]
fn archived_takes_true_or_false_and_says_so_when_it_is_neither() {
    let served = Served::new();
    let doc = served.wrote("# Actas", None);

    let why = served.refused(
        "archive_doc",
        serde_json::json!({ "doc": &doc, "archived": "false" }),
    );

    assert!(why.contains("true or false"), "{why}");
    let listed = served.call("docs", serde_json::json!({ "scope": "open" }));
    assert!(
        serde_json::to_string(&listed).unwrap().contains(&doc),
        "it must not have been put away on a word it did not understand"
    );
}

fn beside(dir: &std::path::Path, named: &str, bytes: &[u8]) {
    let at = dir.join(named);
    if let Some(up) = at.parent() {
        std::fs::create_dir_all(up).unwrap();
    }
    std::fs::write(at, bytes).unwrap();
}

const A_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
];

#[test]
fn a_picture_beside_the_file_comes_in_with_it() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "Risk Matrix/Untitled.png", A_PNG);
    let at = dir.path().join("Risk Matrix.md");
    std::fs::write(
        &at,
        "# Risk Matrix\n\n![Untitled](Risk%20Matrix/Untitled.png)\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(
        body.contains("attachments/"),
        "the picture was not brought in: {body}"
    );
    assert!(
        !body.contains("Risk%20Matrix"),
        "it still points outside Tisty: {body}"
    );
    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(1)
    );
}

#[test]
fn a_page_that_leaves_for_another_document_says_the_old_line_is_still_there() {
    let served = Served::new();
    let one = served.wrote("# Libro uno\n\nintro", None);
    let two = served.wrote("# Libro dos\n\nintro", None);
    let page = served.wrote("# Capitulo", Some(&one));

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": &page, "page_of": &two }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert!(
        said["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("is still written in"),
        "the book it left still names it and nothing said so: {said}"
    );
    assert!(
        served.body_of(&one).contains(&format!("tisty:doc/{page}")),
        "the old body is not rewritten by a call that did not name it"
    );
}

#[test]
fn a_document_taken_out_twice_to_the_same_place_says_what_is_already_there() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nlo que hay", None);
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let into = dir.path().to_str().unwrap();

    let first = served.call(
        "export_doc",
        serde_json::json!({ "doc": &doc, "into": into }),
    );
    assert_ne!(first["result"]["isError"].as_bool(), Some(true), "{first}");

    let why = served.refused(
        "export_doc",
        serde_json::json!({ "doc": &doc, "into": into }),
    );
    assert!(
        why.contains("is already there, and an export writes a new one"),
        "the second one has to say what is in the way, not an error number: {why}"
    );
}

#[test]
fn reading_a_document_says_which_of_its_pages_no_line_names() {
    let served = Served::new();
    let book = served.wrote("# Libro\n\nintro", None);
    let named = served.wrote("# Uno", Some(&book));
    let loose = served.wrote("# Dos", None);
    served.call(
        "page_doc",
        serde_json::json!({ "doc": &loose, "page_of": &book }),
    );

    let said = served.call("read_doc", serde_json::json!({ "doc": &book }));

    assert_eq!(
        said["result"]["structuredContent"]["pages_loose"],
        serde_json::json!([loose]),
        "{said}"
    );
    assert!(
        served
            .body_of(&book)
            .contains(&format!("tisty:doc/{named}")),
        "the one the body names is not loose"
    );
}

#[test]
fn a_file_the_document_does_not_keep_beside_it_is_left_where_it_is() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "elsewhere/private.png", A_PNG);
    beside(dir.path(), "notes/pictures/ours.png", A_PNG);
    let at = dir.path().join("notes/Notes.md");
    std::fs::write(
        &at,
        "# Notes\n\n![ours](pictures/ours.png)\n\n![theirs](../elsewhere/private.png)\n\n![theirs again](%2e%2e/elsewhere/private.png)\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_ne!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(1),
        "only what the document keeps beside it may come in: {said}"
    );
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(
        !body.contains("elsewhere"),
        "a file from another folder came in: {body}"
    );
    assert!(
        said["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("sits outside the folder the document came from"),
        "it did not say why the file was left behind: {said}"
    );
}

#[test]
fn a_file_outside_the_folder_reads_the_same_whether_it_is_there_or_not() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "elsewhere/here.png", A_PNG);
    let said = |named: &str| {
        let at = dir.path().join("notes/Notes.md");
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, format!("# Notes\n\n![one](../elsewhere/{named})\n")).unwrap();
        let out = served.call(
            "import_doc",
            serde_json::json!({ "path": at.to_str().unwrap() }),
        );
        out["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .replace(named, "<named>")
    };

    let there = said("here.png");
    let gone = said("nowhere.png");

    assert!(
        there.contains("sits outside the folder"),
        "a file that is there must not be taken: {there}"
    );
    assert_eq!(
        there
            .split("the words that named them are still in the text")
            .nth(1),
        gone.split("the words that named them are still in the text")
            .nth(1),
        "what it says tells whether the file exists, which is a way to read the disk"
    );
}

#[test]
fn a_key_beside_a_document_comes_in_as_a_copy_tisty_keeps() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(
        dir.path(),
        "secretos/server.key",
        b"-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n",
    );
    let at = dir.path().join("Notas.md");
    std::fs::write(
        &at,
        "# Notas\n\nLa [clave del servidor](secretos/server.key) y nada mas.\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(body.contains("attachments/"), "no se copio: {body}");
    assert!(body.contains("clave del servidor"), "{body}");
    let left = said["result"]["structuredContent"]["left_behind"].to_string();
    assert_eq!(left, "[]", "nada quedo fuera: {left}");
}

#[test]
fn a_file_that_is_not_there_leaves_words_and_not_a_broken_link() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join("Suelto.md");
    std::fs::write(&at, "# Suelto\n\n![foto](assets/no-esta.png)\n").unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(!body.contains("no-esta.png"), "{body}");
    assert!(body.contains("foto"), "{body}");
}

#[test]
fn another_markdown_file_comes_in_and_is_named_as_one_to_import_too() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "Sub/Otra.md", b"# Otra\n");
    let at = dir.path().join("Padre.md");
    std::fs::write(&at, "# Padre\n\nVer [Otra](Sub/Otra.md).\n").unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let named = said["result"]["structuredContent"]["names_markdown"].to_string();
    assert!(named.contains("Otra.md"), "{named}");
    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(1),
        "it still comes in, so the link does not point outside Tisty"
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(body.contains("attachments/"), "{body}");
    assert!(!body.contains("Sub/Otra.md"), "{body}");
}

#[test]
fn a_path_on_this_machine_is_never_left_in_the_text() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join("Rutas.md");
    std::fs::write(
        &at,
        "# Rutas\n\nUno [passwd](/etc/passwd), dos [ini](file:///C:/Windows/win.ini), tres [share](//servidor/x/y.png) y un [sitio](https://example.com) de verdad.\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    for gone in ["/etc/passwd", "file:///", "//servidor"] {
        assert!(!body.contains(gone), "{gone} stayed in: {body}");
    }
    assert!(
        body.contains("https://example.com"),
        "a link to the web is not a path: {body}"
    );
    for kept in ["passwd", "ini", "share"] {
        assert!(body.contains(kept), "the words were lost: {body}");
    }
}

#[test]
fn a_link_shown_as_an_example_inside_code_is_not_followed() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "real.png", A_PNG);
    let at = dir.path().join("Ejemplo.md");
    std::fs::write(
        &at,
        "# Ejemplo\n\nAsi se escribe: `![alt](real.png)`\n\n```md\n![alt](real.png)\n```\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(0),
        "an example is not a file to keep"
    );
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert_eq!(
        body.matches("![alt](real.png)").count(),
        2,
        "both examples read as they were written: {body}"
    );
}

#[test]
fn a_label_with_brackets_of_its_own_keeps_every_character() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join("Corchetes.md");
    std::fs::write(
        &at,
        "# Corchetes\n\nAntes [a [b] c](no-existe.png) despues.\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(body.contains("a [b] c"), "a bracket was eaten: {body}");
    assert!(!body.contains("no-existe.png"), "{body}");
}

#[test]
fn nothing_is_copied_in_when_the_document_itself_is_turned_away() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "real.png", A_PNG);
    let at = dir.path().join("Enorme.md");
    let body = format!(
        "# Enorme\n\n![foto](real.png)\n\n{}\n",
        "palabra ".repeat(9000)
    );
    std::fs::write(&at, body).unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
    let shelf = served.data().join("attachments");
    let left = std::fs::read_dir(&shelf)
        .map(|one| one.count())
        .unwrap_or(0);
    assert_eq!(
        left, 0,
        "a refused import must leave no file no document names"
    );
}

#[test]
fn a_drawing_a_page_and_a_data_file_come_in_like_any_other() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(
        dir.path(),
        "assets/dibujo.svg",
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"4\"/></svg>",
    );
    beside(
        dir.path(),
        "assets/pagina.html",
        b"<!doctype html><p>hola</p>",
    );
    beside(
        dir.path(),
        "assets/datos.xml",
        b"<?xml version=\"1.0\"?><a/>",
    );
    let at = dir.path().join("Export.md");
    std::fs::write(
        &at,
        "# Export\n\n![dibujo](assets/dibujo.svg)\n\n[pagina](assets/pagina.html)\n\n[datos](assets/datos.xml)\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(3),
        "{said}"
    );
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    for kind in ["svg", "html", "xml"] {
        assert!(
            body.contains("attachments/") && body.contains(kind),
            "{kind} did not come in: {body}"
        );
    }
    assert!(!body.contains("assets/"), "still pointing outside: {body}");
}

#[test]
fn a_target_with_spaces_nobody_encoded_is_still_the_target() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    beside(dir.path(), "Risk Matrix/Untitled.png", A_PNG);
    let at = dir.path().join("Crudo.md");
    std::fs::write(
        &at,
        "# Crudo\n\n![b](Risk Matrix/Untitled.png)\n\n![c](Risk%20Matrix/Untitled.png \"con titulo\")\n",
    )
    .unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    assert_eq!(
        said["result"]["structuredContent"]["files"].as_u64(),
        Some(2),
        "the one written with spaces came in like the one written with %20: {said}"
    );
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);
    assert!(!body.contains("Risk Matrix/"), "{body}");
    assert!(!body.contains("Risk%20Matrix/"), "{body}");
    assert!(body.contains("con titulo"), "the title is kept: {body}");
}

#[test]
fn an_exported_book_reads_as_a_book_outside_tisty() {
    let served = Served::new();
    let book = served.wrote("# Curso\n\nlo que hay antes", None);
    for one in ["Marzo", "Abril", "Mayo"] {
        served.wrote(&format!("# {one}\n\ntexto de {one}"), Some(&book));
    }
    let out = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();

    let said = served.call(
        "export_doc",
        serde_json::json!({ "doc": &book, "into": out.path().to_str().unwrap() }),
    );

    assert_eq!(
        said["result"]["structuredContent"]["pages_out"].as_u64(),
        Some(3),
        "{said}"
    );
    let mut found: Vec<String> = Vec::new();
    let mut walk = vec![out.path().to_path_buf()];
    while let Some(at) = walk.pop() {
        for one in std::fs::read_dir(&at).unwrap().flatten() {
            let path = one.path();
            if path.is_dir() {
                walk.push(path);
            } else {
                found.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }
    found.sort();
    assert_eq!(
        found,
        vec![
            "01 Marzo.md".to_string(),
            "02 Abril.md".to_string(),
            "03 Mayo.md".to_string(),
            "Curso.md".to_string(),
        ],
        "the pages come out numbered in reading order"
    );

    let cover = std::fs::read_to_string(
        std::fs::read_dir(out.path())
            .unwrap()
            .flatten()
            .find(|one| one.path().is_dir())
            .unwrap()
            .path()
            .join("Curso.md"),
    )
    .unwrap();
    for one in ["[Marzo](<01 Marzo.md>)", "[Abril](<02 Abril.md>)"] {
        assert!(
            cover.contains(one),
            "the cover points at its pages: {cover}"
        );
    }
    assert!(
        !cover.contains("tisty:doc/"),
        "nothing outside Tisty can follow that: {cover}"
    );
}

#[test]
fn a_token_an_assistant_sends_comes_in_under_a_warning_the_person_will_see() {
    let served = Served::new();

    let said = served.call(
        "write_doc",
        serde_json::json!({
            "body": "# Despliegue\n\nEl token es GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a\n"
        }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);

    assert!(body.contains("[!CAUTION]"), "sin aviso: {body}");
    assert!(body.contains("GITHUB_TOKEN"), "no se guardo: {body}");
    assert!(
        body.starts_with("# Despliegue"),
        "el aviso se puso antes del titulo: {body}"
    );
}

#[test]
fn a_document_that_only_names_its_variables_gets_no_warning() {
    let served = Served::new();

    let said = served.call(
        "write_doc",
        serde_json::json!({
            "body": "# Que instalar\n\n```bash\nFOO_URL=\"https://ejemplo.cl/x\"\nFOO_CLIENT_KEY=\"REDACTADO\"\n```\n"
        }),
    );

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);

    assert!(!body.contains("[!CAUTION]"), "aviso de mas: {body}");
}

#[test]
fn many_secrets_get_one_warning_at_the_top_and_leave_the_body_alone() {
    let served = Served::new();
    let sent = "# Despliegue\n\nDB_PASSWORD=Tr0ub4dor3xK\n\nY mas abajo:\n\nDB_PASSWORD=Tr0ub4dor3xK\nSMTP_PASSWORD=h4nsel4ndGretel\n";

    let said = served.call("write_doc", serde_json::json!({ "body": sent }));

    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    let body = served.body_of(doc);

    assert_eq!(
        body.matches("[!CAUTION]").count(),
        1,
        "mas de un aviso: {body}"
    );
    assert_eq!(
        body.matches("DB_PASSWORD").count(),
        3,
        "el nombre se repitio en el aviso o se perdio del cuerpo: {body}"
    );
    assert!(
        body.ends_with("SMTP_PASSWORD=h4nsel4ndGretel\n"),
        "el cuerpo cambio: {body}"
    );
}

const A_SECRET: &str = "GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a";

fn written(served: &Served, body: &str) -> String {
    let said = served.call("write_doc", serde_json::json!({ "body": body }));
    let doc = said["result"]["structuredContent"]["doc"].as_str().unwrap();
    served.body_of(doc)
}

#[test]
fn the_warning_is_a_block_of_its_own_with_blank_lines_around_it() {
    let served = Served::new();
    for sent in [
        format!("# Despliegue\nEl entorno usa esto:\n\n{A_SECRET}\n"),
        format!("| a | b |\n| --- | --- |\n| 1 | 2 |\n\n{A_SECRET}\n"),
        format!("Notas sueltas\n{A_SECRET}\n"),
    ] {
        let body = written(&served, &sent);
        let lines: Vec<&str> = body.lines().collect();
        let at = lines
            .iter()
            .position(|one| one.contains("[!CAUTION]"))
            .unwrap_or_else(|| panic!("sin aviso: {body}"));
        let shut = lines[at..]
            .iter()
            .position(|one| !one.trim_start().starts_with('>'))
            .map(|one| at + one)
            .unwrap_or(lines.len());

        assert!(
            at == 0 || lines[at - 1].trim().is_empty(),
            "sin hueco antes del aviso: {body}"
        );
        assert!(
            shut >= lines.len() || lines[shut].trim().is_empty(),
            "sin hueco despues del aviso, se traga lo de abajo: {body}"
        );
    }
}

#[test]
fn a_table_at_the_top_is_still_a_table() {
    let served = Served::new();
    let body = written(
        &served,
        &format!("| a | b |\n| --- | --- |\n| 1 | 2 |\n\n{A_SECRET}\n"),
    );

    for row in ["| a | b |", "| --- | --- |", "| 1 | 2 |"] {
        assert!(
            body.lines().any(|one| one.trim() == row),
            "la fila {row:?} dejo de ser fila: {body}"
        );
    }
    let rows = body
        .lines()
        .position(|one| one.trim() == "| a | b |")
        .expect("la tabla sigue");
    let mark = body
        .lines()
        .position(|one| one.contains("[!CAUTION]"))
        .expect("hay aviso");
    assert!(mark > rows, "el aviso partio la tabla por arriba: {body}");
}

#[test]
fn writing_the_same_body_again_does_not_stack_warnings() {
    let served = Served::new();
    let sent = format!("# Despliegue\n\n{A_SECRET}\n");
    let said = served.call("write_doc", serde_json::json!({ "body": sent }));
    let doc = said["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    let mut body = served.body_of(&doc);
    assert!(body.contains("[!CAUTION]"), "no hubo aviso: {body}");

    for round in 0..3 {
        let print = tisty_core::attach::printed(body.as_bytes());
        let again = served.call(
            "write_doc",
            serde_json::json!({ "doc": doc, "print": print, "body": body.clone() }),
        );
        assert!(
            again["result"]["isError"] != serde_json::json!(true),
            "la reescritura {round} fallo: {again}"
        );
        body = served.body_of(&doc);
    }

    assert_eq!(
        body.matches("[!CAUTION]").count(),
        1,
        "se apilaron avisos: {body}"
    );
}

#[test]
fn adding_to_a_document_again_does_not_stack_warnings() {
    let served = Served::new();
    let said = served.call(
        "write_doc",
        serde_json::json!({ "body": "# Notas\n\nnada por aqui\n" }),
    );
    let doc = said["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    for _ in 0..3 {
        served.call(
            "append_doc",
            serde_json::json!({ "doc": doc, "body": format!("\n{A_SECRET}\n") }),
        );
    }

    let body = served.body_of(&doc);
    assert_eq!(
        body.matches("[!CAUTION]").count(),
        1,
        "se apilaron avisos: {body}"
    );
}

#[test]
fn a_warning_never_lands_inside_a_fence() {
    let served = Served::new();
    let said = served.call(
        "write_doc",
        serde_json::json!({ "body": "# Abierto\n\n```sh\necho hola\n" }),
    );
    let doc = said["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    served.call(
        "append_doc",
        serde_json::json!({ "doc": doc, "body": format!("{A_SECRET}\n") }),
    );

    let body = served.body_of(&doc);
    let fences = body.matches("```").count();
    assert!(
        !body.contains("[!CAUTION]") || fences.is_multiple_of(2),
        "el aviso quedo dentro del codigo: {body}"
    );
}

#[test]
fn the_warning_never_becomes_the_title() {
    let served = Served::new();
    for sent in [
        format!("Notas de despliegue\n\n{A_SECRET}\n"),
        format!("\n\n# Con hueco arriba\n\n{A_SECRET}\n"),
        format!("## Solo un subtitulo\n\n{A_SECRET}\n"),
        format!("| a | b |\n| --- | --- |\n\n{A_SECRET}\n"),
        format!("> Una cita\n>\n> {A_SECRET}\n"),
        format!("\u{feff}# Con marca de orden\n\n{A_SECRET}\n"),
    ] {
        let said = served.call("write_doc", serde_json::json!({ "body": sent }));
        let title = said["result"]["structuredContent"]["title"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(
            !title.contains("credential") && !title.contains("credencial"),
            "el aviso se quedo de titulo: {title:?} para {sent:?}"
        );
    }
}

#[test]
fn a_markdown_file_written_on_windows_comes_in() {
    let served = Served::new();
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join("Windows.md");
    std::fs::write(&at, "# Notas de Windows\r\n\r\nUna linea cualquiera.\r\n").unwrap();

    let said = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_str().unwrap() }),
    );

    let doc = said["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap_or_else(|| panic!("no entro: {said}"));
    let body = served.body_of(doc);
    assert!(!body.contains('\r'), "quedaron retornos de carro: {body:?}");
    assert!(body.contains("Una linea cualquiera."));
}

#[test]
fn a_body_the_editor_could_not_keep_is_never_written_by_the_warning() {
    let served = Served::new();
    for sent in [
        format!("# Titulo\n\n{A_SECRET}\n"),
        format!("> [!NOTE]\n> Una nota del principio\n\n{A_SECRET}\n"),
        format!("```math\nx = 1\n```\n\n{A_SECRET}\n"),
        format!("- una lista\n- con puntos\n\n{A_SECRET}\n"),
    ] {
        let said = served.call("write_doc", serde_json::json!({ "body": sent }));
        let doc = said["result"]["structuredContent"]["doc"]
            .as_str()
            .unwrap_or_else(|| panic!("no entro: {said} para {sent:?}"));
        let body = served.body_of(doc);

        let again = served.call("write_doc", serde_json::json!({ "body": body.clone() }));
        assert!(
            again["result"]["isError"] != serde_json::json!(true),
            "lo que se guardo ya no se puede volver a escribir: {again} para {body:?}"
        );
    }
}

/// Every one of these once spliced the tail of the document back on as a whole second copy.
#[test]
fn an_edit_that_reaches_either_end_leaves_no_copy_of_the_document_behind() {
    let served = Served::new();
    let whole = "Repro\n\n## S\n\nx";

    let told = |doc: &str, what: &str| {
        let now = served.body_of(doc);
        assert!(now.len() < whole.len() * 2, "{what} duplicated it: {now:?}");
        assert!(
            now.matches("Repro").count() <= 1,
            "{what} duplicated it: {now:?}"
        );
    };

    let doc = served.wrote(whole, None);
    let said = served.call(
        "append_doc",
        serde_json::json!({ "doc": &doc, "under": "S", "body": "Z" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    told(&doc, "append_doc under the last heading");

    let doc = served.wrote(whole, None);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "section": 0, "print": served.print_of(&doc), "new": "## S2" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    told(&doc, "edit_doc over the last section");

    let doc = served.wrote(whole, None);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "from": 3, "to": 5, "print": served.print_of(&doc), "new": "y" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    told(&doc, "edit_doc to the last line");

    let doc = served.wrote(whole, None);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "from": 3, "print": served.print_of(&doc), "new": "y" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    told(&doc, "edit_doc with no `to` at all");

    let doc = served.wrote(whole, None);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "from": 1, "to": 2, "print": served.print_of(&doc), "new": "Otro" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    let now = served.body_of(&doc);
    assert_eq!(
        now.matches("## S").count(),
        1,
        "edit_doc from the first line duplicated it: {now:?}"
    );
    assert!(
        !now.contains("Repro"),
        "the first line was replaced, not kept: {now:?}"
    );

    let doc = served.wrote(whole, None);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "from": 3, "to": 4, "print": served.print_of(&doc), "new": "y" }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    told(&doc, "edit_doc that touches neither end");
}

#[test]
fn what_a_write_replaced_can_be_put_back_and_putting_it_back_undoes_itself() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nLo que se dijo.", None);
    let print = served.print_of(&doc);

    served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Lo que se dijo.", "new": "Otra cosa.", "print": print }),
    );
    assert!(served.body_of(&doc).contains("Otra cosa"));

    let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert!(
        served.body_of(&doc).contains("Lo que se dijo"),
        "what the edit replaced came back: {}",
        served.body_of(&doc)
    );

    let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert!(
        served.body_of(&doc).contains("Otra cosa"),
        "going back twice lands where it started, so trying it is safe: {}",
        served.body_of(&doc)
    );
}

#[test]
fn a_document_nothing_has_been_written_over_has_nothing_to_go_back_to() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nLo que se dijo.", None);

    let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
    assert_eq!(said["result"]["isError"].as_bool(), Some(true), "{said}");
}

#[test]
fn every_write_says_how_much_the_document_grew_by() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nUno.", None);

    let said = served.call(
        "append_doc",
        serde_json::json!({ "doc": &doc, "body": "Dos." }),
    );
    let grew = said["result"]["structuredContent"]["grew"]
        .as_i64()
        .unwrap();
    assert_eq!(grew, "\n\nDos.".chars().count() as i64, "{said}");

    let print = served.print_of(&doc);
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Dos.", "new": "X", "print": print }),
    );
    assert_eq!(
        said["result"]["structuredContent"]["grew"].as_i64(),
        Some(-3),
        "an edit that takes text out says so with a minus: {said}"
    );
}

#[test]
fn an_edit_can_hand_back_what_it_reads_like_now_instead_of_being_read_again() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nUno.\n\nDos.\n\nTres.\n\nCuatro.\n\nCinco.", None);
    let print = served.print_of(&doc);

    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Tres.", "new": "Trece.", "print": print, "echo": true }),
    );
    let around = &said["result"]["structuredContent"]["around"];
    assert!(
        around["body"].as_str().unwrap().contains("Trece."),
        "the change is in what came back: {said}"
    );
    assert!(around["from"].as_u64().unwrap() >= 1, "{said}");

    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Cuatro.", "new": "Catorce.", "print": served.print_of(&doc) }),
    );
    assert!(
        said["result"]["structuredContent"]["around"].is_null(),
        "nothing is handed back unless it was asked for: {said}"
    );
}

#[test]
fn putting_a_document_away_says_what_is_left_pointing_at_it() {
    let served = Served::new();
    let cited = served.wrote("# Citado\n\nAqui.", None);
    let pointing = served.wrote(
        &format!("# Apunta\n\nMira [esto](tisty:doc/{cited})."),
        None,
    );

    let said = served.call("archive_doc", serde_json::json!({ "doc": &cited }));
    let told = said["result"]["structuredContent"]["pointed_at"]
        .as_array()
        .unwrap();
    assert_eq!(told, &[serde_json::json!(pointing)], "{said}");
}

#[test]
fn a_run_of_lines_is_held_to_the_same_budget_as_every_other_way_of_reading() {
    let served = Served::new();
    let long = (1..=400)
        .map(|n| {
            format!("Linea {n} con bastante texto detras para que ocupe lo suyo en el presupuesto.")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let doc = served.wrote(&format!("# Largo\n\n{long}"), None);

    let said = served.call(
        "read_doc",
        serde_json::json!({ "doc": &doc, "from": 1, "to": 900 }),
    );
    let kept = &said["result"]["structuredContent"];
    assert!(
        kept["next"].as_u64().is_some(),
        "a run too wide to fit says where to carry on: {}",
        kept["next"]
    );
    assert!(
        kept["body"].as_str().unwrap().chars().count() < long.chars().count(),
        "naming a run was the way round the budget"
    );
}

/// Adding to the end keeps no body of its own, and neither does the window's save. Either one
/// leaves what is kept beside the document further back than a single step.
#[test]
fn going_back_is_refused_when_it_would_undo_more_than_the_write_that_was_kept() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nUno.", None);

    served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Uno.", "new": "Dos.", "print": served.print_of(&doc) }),
    );
    served.call(
        "append_doc",
        serde_json::json!({ "doc": &doc, "body": "Acuerdo importante." }),
    );

    let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
    assert_eq!(
        said["result"]["isError"].as_bool(),
        Some(true),
        "going back here would have taken the appended line with it: {said}"
    );
    assert!(
        served.body_of(&doc).contains("Acuerdo importante"),
        "and nothing was touched: {}",
        served.body_of(&doc)
    );
}

#[test]
fn a_section_is_held_to_the_same_budget_as_a_run_of_lines() {
    let served = Served::new();
    let long = (1..=400)
        .map(|n| {
            format!("Linea {n} con bastante texto detras para que ocupe lo suyo en el presupuesto.")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let doc = served.wrote(&format!("# Largo\n\n{long}"), None);

    let said = served.call("read_doc", serde_json::json!({ "doc": &doc, "section": 0 }));
    let kept = &said["result"]["structuredContent"];
    assert!(
        kept["next"].as_u64().is_some(),
        "one heading over the whole document was the way round the budget: {}",
        kept["next"]
    );
}

#[test]
fn indented_code_is_not_mistaken_for_a_paragraph_somebody_wrapped() {
    let served = Served::new();
    let doc = served.wrote("# Codigo\n\nLo de abajo.", None);

    let said = served.call(
        "append_doc",
        serde_json::json!({
            "doc": &doc,
            "body": "    let one = something_with_a_fairly_long_name(argument, another);\n    let two = something_with_a_fairly_long_name(argument, another);\n    let three = something_with_long_name(argument, another_one_here);",
        }),
    );
    let told = said["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(
        !told.contains("wrapped"),
        "indented code is not prose anybody wrapped: {told}"
    );
}

#[test]
fn going_back_over_more_than_one_write_is_asked_for_and_is_itself_undone() {
    let served = Served::new();
    let doc = served.wrote("# Acta\n\nUno.", None);

    served.call(
        "edit_doc",
        serde_json::json!({ "doc": &doc, "old": "Uno.", "new": "Dos.", "print": served.print_of(&doc) }),
    );
    served.call(
        "append_doc",
        serde_json::json!({ "doc": &doc, "body": "Acuerdo importante." }),
    );
    let stood = served.body_of(&doc);

    let said = served.call(
        "restore_doc",
        serde_json::json!({ "doc": &doc, "even_if_more": true }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(
        said["result"]["structuredContent"]["over_more_than_one_write"],
        serde_json::json!(true),
        "it says it went further than one write: {said}"
    );
    let now = served.body_of(&doc);
    assert!(now.contains("Uno."), "it went all the way back: {now:?}");
    assert!(!now.contains("Acuerdo importante"), "{now:?}");

    // What it wrote over is kept in turn, so the same call again undoes the whole thing.
    let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
    assert!(
        said["result"]["isError"].as_bool() != Some(true),
        "going back over more than one write left something that could be put back: {said}"
    );
    assert_eq!(
        served.body_of(&doc),
        stood,
        "everything that was undone came back, without needing the flag again"
    );
}

/// A body handed in without a closing newline is written with one. Hashing the one that was handed
/// in rather than the one that reached the disk refused every step back the moment it was taken.
#[test]
fn a_body_written_over_can_be_put_back_whether_or_not_it_ended_in_a_newline() {
    for body in ["# Acta\n\nUno.", "# Acta\n\nUno.\n"] {
        let served = Served::new();
        let doc = served.wrote(body, None);

        let said = served.call(
            "write_doc",
            serde_json::json!({ "doc": &doc, "body": "# Acta\n\nDos.", "print": served.print_of(&doc) }),
        );
        assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

        let said = served.call("restore_doc", serde_json::json!({ "doc": &doc }));
        assert!(
            said["result"]["isError"].as_bool() != Some(true),
            "nothing was written in between, so this is one step back: {said}"
        );
        assert!(
            said["result"]["structuredContent"]["over_more_than_one_write"].is_null(),
            "and it did not have to go further than one write: {said}"
        );
        assert!(
            served.body_of(&doc).contains("Uno."),
            "{}",
            served.body_of(&doc)
        );
    }
}

#[test]
fn looking_inside_a_document_takes_words_in_any_order_and_accents_decide_nothing() {
    let served = Served::new();
    let doc = served.wrote(
        "# Plan\n\nintro\n\n## El riego\n\nLa migración a Windows y macOS va en mayo.\n\n## El portón\n\nsin fecha\n",
        None,
    );
    let inside = |query: &str| {
        served.call("find", serde_json::json!({ "doc": &doc, "query": query }))["result"]
            ["structuredContent"]
            .clone()
    };

    let kept = inside("macos windows");
    assert_eq!(kept["total"], 1, "two words in the other order: {kept}");
    assert_eq!(kept["lines"][0]["line"], 7);
    assert_eq!(kept["lines"][0]["section"]["at"], 1);
    assert_eq!(kept["lines"][0]["section"]["title"], "El riego");
    assert_eq!(
        kept["lines"][0]["around"], "\nLa migración a Windows y macOS va en mayo.\n",
        "the line before and the line after, no more"
    );

    let kept = inside("migracion");
    assert_eq!(kept["total"], 1, "typed without its accent: {kept}");

    let kept = inside("porton");
    assert_eq!(kept["lines"][0]["line"], 9);
    assert_eq!(
        kept["lines"][0]["section"]["at"], 2,
        "a hit on the heading line itself belongs to that section: {kept}"
    );

    assert_eq!(
        inside("\"macos va\"")["total"],
        1,
        "a phrase in quotes, as written"
    );
    assert_eq!(
        inside("\"va macos\"")["total"],
        0,
        "a phrase in quotes is looked for whole"
    );
    assert_eq!(inside("va macos")["total"], 1, "the same two words loose");
    assert!(
        inside("\"\"")["error"].is_object() || inside("\"\"").get("total").is_none(),
        "nothing to look for is refused"
    );

    let bare = served.wrote(
        "Sin titulo con almohadilla\n\nuna linea que dice riego\n",
        None,
    );
    let kept = served.call(
        "find",
        serde_json::json!({ "doc": &bare, "query": "riego" }),
    )["result"]["structuredContent"]
        .clone();
    assert_eq!(kept["total"], 1);
    assert!(
        kept["lines"][0].get("section").is_none(),
        "no heading, so no section to sit in: {kept}"
    );
    let told = served.call(
        "find",
        serde_json::json!({ "doc": &bare, "query": "riego" }),
    )["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(told.contains("line 3 — una linea"), "{told}");
    let told = served.call(
        "find",
        serde_json::json!({ "doc": &doc, "query": "porton" }),
    )["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(told.contains("line 9 (section 2, El portón) — "), "{told}");
}

#[test]
fn the_outline_of_a_book_names_each_page_with_what_it_holds() {
    let served = Served::new();
    let book = served.wrote(
        "# Actas\n\n## Enero\n\nlo de enero\n\n## Febrero\n\nlo de febrero\n",
        None,
    );
    let first = served.wrote(
        "# Acta del 3\n\n## Riego\n\nuno dos tres #riego\n",
        Some(&book),
    );
    let second = served.wrote("# Acta del 10\n\ncuatro cinco\n", Some(&book));
    let third = served.wrote("seis siete ocho\n", Some(&book));
    served.call(
        "sum_up",
        serde_json::json!({ "doc": &second, "summary": "the tenth, in short", "notes": "not for the list" }),
    );

    let said = served.call("outline_doc", serde_json::json!({ "doc": &book }));
    let kept = &said["result"]["structuredContent"];
    let pages = kept["pages"].as_array().expect("a row per page");
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0]["doc"], first);
    assert_eq!(pages[0]["title"], "Acta del 3");
    assert_eq!(pages[0]["sections"], 2);
    assert_eq!(pages[0]["words"], 10);
    assert_eq!(
        pages[0]["about"],
        serde_json::json!(["riego"]),
        "{}",
        pages[0]
    );
    assert_eq!(pages[1]["doc"], second);
    assert_eq!(pages[1]["title"], "Acta del 10");
    assert_eq!(pages[1]["sections"], 1, "the title is a heading too");
    assert_eq!(pages[1]["gist"]["summary"], "the tenth, in short");
    assert!(
        pages[1]["gist"].get("notes").is_none(),
        "notes are not for a list: {}",
        pages[1]
    );
    assert_eq!(pages[2]["doc"], third);
    assert_eq!(pages[2]["words"], 3);
    assert!(
        pages[2].get("sections").is_none(),
        "no heading at all: {}",
        pages[2]
    );
    let told = said["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        told.contains("Pages, in the order they are read (3):"),
        "{told}"
    );
    assert!(
        told.contains(&format!("{first} — Acta del 3 (10 words, 2 sections)")),
        "{told}"
    );
    assert!(
        told.contains(&format!("{third} — seis siete ocho (3 words)")),
        "{told}"
    );
    assert!(told.contains("Enero  ·  lines 3-5, 22 chars"), "{told}");

    let enero = &kept["outline"][1];
    assert_eq!(enero["title"], "Enero");
    assert_eq!(enero["line"], 3);
    assert_eq!(enero["to"], 5, "the blank line before Febrero is nobody's");
    assert_eq!(enero["chars"], "## Enero\n\nlo de enero\n".chars().count());

    let said = served.call("outline_doc", serde_json::json!({ "doc": &first }));
    assert_eq!(
        said["result"]["structuredContent"]["page_of"], book,
        "a page's outline says whose page it is"
    );
}

#[test]
fn a_long_document_read_whole_comes_back_as_an_outline_that_weighs_each_part() {
    let served = Served::new();
    let long = (1..=200)
        .map(|n| format!("Linea {n} con bastante texto detras para pasar del presupuesto."))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!("# Largo\n\n## Uno\n\n{long}\n\n## Dos\n\nfin\n");
    let doc = served.wrote(&body, None);

    let said = served.call("read_doc", serde_json::json!({ "doc": &doc }));
    let kept = &said["result"]["structuredContent"];
    assert_eq!(kept["whole"], false);
    let uno = &kept["outline"][1];
    assert_eq!(uno["title"], "Uno");
    assert_eq!(uno["line"], 3);
    assert_eq!(uno["to"], 3 + 2 + 200 - 1);
    let held: usize = body
        .lines()
        .skip(2)
        .take(2 + 200)
        .map(|one| one.chars().count() + 1)
        .sum();
    assert_eq!(uno["chars"], held, "what reading section 1 would cost");
}

#[test]
fn a_page_the_archive_holds_is_sent_back_to_its_document_and_not_to_a_folder() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este año.", None);
    let page = served.wrote("# Marzo", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    let why = served.refused(
        "archive_doc",
        serde_json::json!({ "doc": &page, "archived": false }),
    );

    assert!(
        why.contains("document that holds it") && why.contains(&book),
        "a page hangs from a document, and sending the person to a folder sends them nowhere: {why}"
    );
    assert!(!why.contains("  "), "{why}");
}

#[test]
fn what_a_document_takes_to_the_archive_is_what_it_brings_back() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let one = served.wrote("# Marzo", Some(&book));
    let two = served.wrote("# Abril", Some(&book));

    let went = served.said(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );
    assert!(
        went.contains("Its pages went with it") && went.contains(&one) && went.contains(&two),
        "{went}"
    );

    let back = served.said(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": false }),
    );
    assert!(
        back.contains("Its pages came back with it"),
        "coming back is not going away, and the answer has to read like what happened: {back}"
    );

    let again = served.said(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": false }),
    );
    assert!(
        again.contains("already out of the archive") && !again.contains("pages"),
        "asking twice changes nothing and takes no page anywhere: {again}"
    );
}

#[test]
fn a_page_in_the_archive_is_not_pulled_out_of_the_document_that_holds_it() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let page = served.wrote("# Marzo", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    let why = served.refused("page_doc", serde_json::json!({ "doc": &page }));

    assert!(
        why.contains("in the archive"),
        "taking it out would leave it awake outside the archive with nobody's hand on it: {why}"
    );
    assert!(!why.contains("  "), "{why}");
    assert_eq!(served.pages_of(&book), vec![page]);
}

#[test]
fn a_task_pointing_at_a_page_of_a_book_in_the_archive_says_it_is_put_away() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let page = served.wrote("# Marzo", Some(&book));
    served.call(
        "propose",
        serde_json::json!({
            "title": "read what March says",
            "description": format!("lo dejado en [Marzo](tisty:doc/{page})"),
            "source": "test#1",
        }),
    );

    let listed = served.cli(&["ls", "all"]);
    let number = listed
        .lines()
        .find(|line| line.contains("read what March says"))
        .and_then(|line| line.split_whitespace().next())
        .expect("the task is listed")
        .trim_end_matches('.')
        .to_string();

    let before = served.cli(&["story", &number]);
    assert!(!before.contains("(put away)"), "{before}");

    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    let after = served.cli(&["story", &number]);
    assert!(
        after.contains("(put away)"),
        "the page went to the archive inside its document, and the task has to say so: {after}"
    );
}

#[test]
fn a_count_of_pages_says_how_many_of_them_are_not_awake() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let one = served.wrote("# Marzo", Some(&book));
    served.wrote("# Abril", Some(&book));
    let old = served.wrote("# Enero", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &old, "archived": true }),
    );
    served.call(
        "flag_doc",
        serde_json::json!({ "doc": &one, "body": "it has had its day" }),
    );

    let listed = served.call("docs", serde_json::json!({ "folders": false }));
    let row = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["doc"] == serde_json::json!(book))
        .expect("the document that holds them is listed");
    assert_eq!(row["pages"], 3);
    assert_eq!(row["pages_archived"], 1, "three pages is not three to read");
    assert_eq!(row["pages_flagged"], 1);

    let read = served.call("read_doc", serde_json::json!({ "doc": &book }));
    assert_eq!(
        read["result"]["structuredContent"]["pages_archived"],
        serde_json::json!([old]),
        "which one it is, not only how many"
    );
    assert_eq!(
        read["result"]["structuredContent"]["pages_flagged"],
        serde_json::json!([one])
    );

    let outline = served.call("outline_doc", serde_json::json!({ "doc": &book }));
    let said = outline["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        said.contains("1 of them put away on their own")
            && said.contains("1 an agent gave up for old"),
        "the prose has to say what the rows say: {said}"
    );
    assert!(
        said.contains("in the archive on its own, read-only"),
        "{said}"
    );
}

#[test]
fn what_a_document_is_called_is_what_the_person_is_told() {
    let served = Served::new();
    let book = served.wrote("# Actas de enero\n\nlo que se dijo.", None);

    let said = served.said(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    assert!(
        said.contains("\"Actas de enero\""),
        "an id is not a name they can look up in the window: {said}"
    );
    assert!(
        said.contains(&book),
        "and the id still has to be there for the next call: {said}"
    );
}

#[test]
fn a_page_the_document_covers_is_not_woken_behind_its_back() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let page = served.wrote("# Marzo", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &page, "archived": true }),
    );
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    let why = served.refused(
        "archive_doc",
        serde_json::json!({ "doc": &page, "archived": false }),
    );
    assert!(
        why.contains("document that holds it"),
        "nothing here takes it out while the document holds it: {why}"
    );

    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": false }),
    );
    let read = served.call("read_doc", serde_json::json!({ "doc": &page }));
    assert_eq!(
        read["result"]["structuredContent"]["archived"], true,
        "the page was apart before, and it stays apart: {read}"
    );
}

#[test]
fn a_page_put_away_on_its_own_can_still_become_a_document_of_its_own() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let page = served.wrote("# Marzo", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &page, "archived": true }),
    );

    let said = served.said("page_doc", serde_json::json!({ "doc": &page }));

    assert!(said.contains("document of its own"), "{said}");
    let read = served.call("read_doc", serde_json::json!({ "doc": &page }));
    assert_eq!(
        read["result"]["structuredContent"]["archived"], true,
        "reorganising is not taking it out of the archive: {read}"
    );
    let up = served.call("read_doc", serde_json::json!({ "doc": &book }));
    assert!(
        up["result"]["structuredContent"]["pages"].is_null(),
        "and the book no longer holds it: {up}"
    );
}

#[test]
fn a_listing_says_which_pages_answer_for_themselves_and_which_are_only_covered() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let apart = served.wrote("# Marzo", Some(&book));
    served.wrote("# Abril", Some(&book));
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &apart, "archived": true }),
    );
    served.call(
        "archive_doc",
        serde_json::json!({ "doc": &book, "archived": true }),
    );

    let listed = served.call("docs", serde_json::json!({ "page_of": &book }));
    let rows = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap();
    let mine = rows
        .iter()
        .find(|row| row["doc"] == serde_json::json!(apart))
        .expect("the page is listed");
    let other = rows
        .iter()
        .find(|row| row["doc"] != serde_json::json!(apart))
        .unwrap();

    assert_eq!(mine["archived"], true);
    assert_eq!(
        mine["apart"], true,
        "an agent has to know which one archive_doc will refuse: {listed}"
    );
    assert_eq!(other["archived"], true);
    assert!(other["apart"].is_null(), "{listed}");
}

#[test]
fn asking_for_the_pages_of_nothing_is_not_asking_for_everything() {
    let served = Served::new();
    served.wrote("# Suelto", None);

    let why = served.refused("docs", serde_json::json!({ "page_of": "" }));
    assert!(why.contains("`page_of`"), "{why}");

    let all = served.call(
        "docs",
        serde_json::json!({ "folder": serde_json::Value::Null }),
    );
    assert!(
        all["result"]["isError"].as_bool() != Some(true),
        "a client that sends an absent option as null is not naming a folder: {all}"
    );
}

#[test]
fn twelve_pages_are_reorganised_in_one_call_and_not_twelve() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let loose: Vec<String> = (1..=3)
        .map(|n| served.wrote(&format!("# Capitulo {n}"), None))
        .collect();

    let said = served.said(
        "page_doc",
        serde_json::json!({ "doc": loose.clone(), "page_of": &book }),
    );

    assert!(said.contains("are now pages of"), "{said}");
    assert_eq!(
        served.pages_of(&book),
        loose,
        "all three, in the order asked"
    );

    let out = served.said("page_doc", serde_json::json!({ "doc": loose.clone() }));
    assert!(out.contains("documents of their own"), "{out}");
    let up = served.call("read_doc", serde_json::json!({ "doc": &book }));
    assert!(up["result"]["structuredContent"]["pages"].is_null(), "{up}");
}

#[test]
fn a_list_that_one_name_spoils_moves_nobody() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("folder", serde_json::json!({ "name": "Proyectos" }));
    let one = served.wrote("# Uno", None);
    let two = served.wrote("# Dos", None);

    let why = served.refused(
        "file_doc",
        serde_json::json!({ "doc": [&one, "no-existe", &two], "folder": "Proyectos" }),
    );

    assert!(why.contains("no-existe"), "{why}");
    for which in [&one, &two] {
        let read = served.call("read_doc", serde_json::json!({ "doc": which }));
        assert!(
            read["result"]["structuredContent"]["folder"].is_null(),
            "one intention is one move or none: {read}"
        );
    }
}

#[test]
fn what_points_at_a_document_is_there_to_be_asked_before_putting_it_away() {
    let served = Served::new();
    let one = served.wrote("# Acta de enero", None);
    let two = served.wrote(
        &format!("# Resumen\n\nlo dejado en [enero](tisty:doc/{one})"),
        None,
    );

    let said = served.said("outline_doc", serde_json::json!({ "doc": &one }));

    assert!(
        said.contains("Pointing at it") && said.contains(&two),
        "asking before archiving is the only moment it helps: {said}"
    );
    let told = served.call("outline_doc", serde_json::json!({ "doc": &one }));
    assert_eq!(
        told["result"]["structuredContent"]["pointed_at"],
        serde_json::json!([two])
    );
}

#[test]
fn pages_taken_out_together_land_one_after_another_and_not_all_at_once() {
    let served = Served::new();
    let book = served.wrote("# Actas", None);
    let pages: Vec<String> = (1..=3)
        .map(|n| served.wrote(&format!("# Capitulo {n}"), Some(&book)))
        .collect();

    served.said("page_doc", serde_json::json!({ "doc": pages.clone() }));

    let listed = served.call("docs", serde_json::json!({ "limit": 10 }));
    let loose: Vec<String> = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["doc"] != serde_json::json!(book))
        .map(|row| row["title"].as_str().unwrap_or_default().to_string())
        .collect();

    assert!(
        loose.contains(&"Capitulo 1".to_string()) && loose.len() == 3,
        "{listed}"
    );
    let apart: Vec<String> = pages
        .iter()
        .map(|which| {
            served.call("read_doc", serde_json::json!({ "doc": which }))["result"]
                ["structuredContent"]["page_of"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        apart,
        vec!["", "", ""],
        "none of them hangs from anything now"
    );

    let again = served.said(
        "page_doc",
        serde_json::json!({ "doc": pages.clone(), "page_of": &book }),
    );
    assert!(again.contains("in that order"), "{again}");
    assert_eq!(
        served.pages_of(&book),
        pages,
        "and putting them back keeps the order they were named in"
    );
}
