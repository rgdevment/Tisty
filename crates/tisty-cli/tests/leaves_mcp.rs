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
fn a_page_is_not_put_away_on_its_own() {
    let served = Served::new();
    let book = served.wrote("# Curso", None);
    let page = served.wrote("# Clase uno", Some(&book));

    let why = served.refused("archive_doc", serde_json::json!({ "doc": &page }));

    assert!(why.contains("page of"), "{why}");
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

    assert!(why.contains("past the"), "{why}");
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
