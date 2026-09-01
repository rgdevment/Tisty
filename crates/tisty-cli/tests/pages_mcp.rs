use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::json;
use tempfile::TempDir;
use ulid::Ulid;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.cli(&["algo mio"]);
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
            let mut pipe = child.stdin.take().unwrap();
            for one in said {
                writeln!(pipe, "{one}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        assert!(
            out.stderr.is_empty(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect(line))
            .collect()
    }

    fn call(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        self.talk(&[&asked]).remove(0)
    }
}

impl Served {
    fn put_away(&self, doc: &str) {
        let paths = tisty_core::Paths::new(
            self.home.path().join("data"),
            self.home.path().join("config"),
        );
        let state = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
        let id = state.docs.values().find(|one| one.file == doc).unwrap().id;
        let who = tisty_core::Config::load_or_init(&paths)
            .unwrap()
            .agent_id
            .unwrap();
        tisty_core::Store::open(paths.store(), who)
            .unwrap()
            .append(tisty_core::Op::DocArchive { id })
            .unwrap();
    }

    /// Seeds documents straight into the event log, bypassing `docs::create`, so approaching the
    /// five-hundred-document ceiling does not cost one subprocess per filler document.
    fn flood_docs(&self, how_many: usize) {
        let paths = tisty_core::Paths::new(
            self.home.path().join("data"),
            self.home.path().join("config"),
        );
        let who = tisty_core::Config::load_or_init(&paths)
            .unwrap()
            .agent_id
            .unwrap();
        let mut store = tisty_core::Store::open(paths.store(), who).unwrap();
        for n in 0..how_many {
            store
                .append(tisty_core::Op::DocAdd {
                    id: Ulid::generate(),
                    d: tisty_core::event::DocAdd {
                        file: format!("filler-{n}"),
                        order: format!("{n:06}"),
                        folder: None,
                        page_of: None,
                    },
                })
                .unwrap();
        }
    }
}

fn wrote_paper(served: &Served, body: &str) -> String {
    served.call("write_doc", json!({ "body": body }))["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string()
}

fn wrote_page(served: &Served, body: &str, page_of: &str) -> String {
    served.call("write_doc", json!({ "body": body, "page_of": page_of }))["result"]
        ["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string()
}

fn body_of(served: &Served, doc: &str) -> String {
    served.call("read_doc", json!({ "doc": doc }))["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap()
        .to_string()
}

fn walked(at: &std::path::Path) -> Box<dyn Iterator<Item = std::path::PathBuf>> {
    let Ok(entries) = std::fs::read_dir(at) else {
        return Box::new(std::iter::empty());
    };
    Box::new(entries.filter_map(Result::ok).flat_map(|one| {
        let path = one.path();
        if path.is_dir() {
            walked(&path)
        } else {
            Box::new(std::iter::once(path))
        }
    }))
}

fn docs_on_disk(served: &Served) -> usize {
    let at = served.home.path().join("data/docs");
    let Ok(entries) = std::fs::read_dir(&at) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|one| one.path().extension().and_then(|e| e.to_str()) == Some("md"))
        .count()
}

#[test]
fn ten_pages_are_written_under_one_document_and_each_reads_back_its_own_body() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");

    let pages: Vec<String> = (0..10)
        .map(|n| {
            wrote_page(
                &served,
                &format!("# Mes {n}\n\nlo que pasó el mes {n}."),
                &parent,
            )
        })
        .collect();

    let listed = served.call("docs", json!({}));
    let all = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap();
    let parent_row = all.iter().find(|one| one["doc"] == parent).unwrap();
    assert_eq!(parent_row["pages"], 10, "{listed}");
    assert!(parent_row["page_of"].is_null());

    for (n, page) in pages.iter().enumerate() {
        let row = all
            .iter()
            .find(|one| one["doc"] == *page)
            .unwrap_or_else(|| panic!("page {n} is not listed: {listed}"));
        assert_eq!(row["page_of"], parent, "{listed}");

        let read = served.call("read_doc", json!({ "doc": page }));
        let body = read["result"]["structuredContent"]["body"]
            .as_str()
            .unwrap();
        assert!(
            body.contains(&format!("lo que pasó el mes {n}.")),
            "page {n} came back with the wrong body: {body}"
        );
    }

    let parent_read = served.call("read_doc", json!({ "doc": parent }));
    let listed_pages: Vec<&str> = parent_read["result"]["structuredContent"]["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one.as_str().unwrap())
        .collect();
    for page in &pages {
        assert!(
            listed_pages.contains(&page.as_str()),
            "{page} is missing from the parent's own list: {listed_pages:?}"
        );
    }
    assert_eq!(listed_pages.len(), 10);
}

#[test]
fn a_document_toggles_between_being_a_page_and_being_its_own_document_back_to_back() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("folder", json!({ "name": "Trabajo" }));
    let a = wrote_paper(&served, "# A\n\nsuelto.");
    let b = served.call(
        "write_doc",
        json!({ "body": "# B\n\notro suelto.", "folder": "Trabajo" }),
    );
    let b = b["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    let hung = served.call("page_doc", json!({ "doc": a, "page_of": b }));
    assert_eq!(hung["result"]["structuredContent"]["page_of"], b, "{hung}");
    let row = &served.call("docs", json!({}))["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["doc"] == a)
        .unwrap()
        .clone();
    assert_eq!(
        row["folder"], "Trabajo",
        "a page takes its document's folder: {row}"
    );

    let out = served.call("page_doc", json!({ "doc": a }));
    assert!(
        out["result"]["structuredContent"]["page_of"].is_null(),
        "{out}"
    );
    let row = served.call("docs", json!({}))["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["doc"] == a)
        .unwrap()
        .clone();
    assert_eq!(
        row["folder"], "Trabajo",
        "taken out on its own, it keeps the folder it had as a page: {row}"
    );

    let hung_again = served.call("page_doc", json!({ "doc": a, "page_of": b }));
    assert_eq!(
        hung_again["result"]["structuredContent"]["page_of"], b,
        "{hung_again}"
    );
    let out_again = served.call("page_doc", json!({ "doc": a }));
    assert!(
        out_again["result"]["structuredContent"]["page_of"].is_null(),
        "toggling twice in a row still works: {out_again}"
    );
}

#[test]
fn page_doc_moves_a_page_straight_from_one_document_to_another() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let first = wrote_paper(&served, "# Primero\n\nuno.");
    let second = wrote_paper(&served, "# Segundo\n\ndos.");
    let page = wrote_page(&served, "# Página\n\nla que se mueve.", &first);

    let moved = served.call("page_doc", json!({ "doc": page, "page_of": second }));
    assert_eq!(
        moved["result"]["structuredContent"]["page_of"], second,
        "{moved}"
    );

    let first_pages =
        served.call("read_doc", json!({ "doc": first }))["result"]["structuredContent"]["pages"]
            .as_array()
            .unwrap()
            .clone();
    assert!(
        !first_pages.iter().any(|one| one == &page),
        "{first_pages:?}"
    );

    let second_pages =
        served.call("read_doc", json!({ "doc": second }))["result"]["structuredContent"]["pages"]
            .as_array()
            .unwrap()
            .clone();
    assert!(
        second_pages.iter().any(|one| one == &page),
        "{second_pages:?}"
    );
}

#[test]
fn an_attachment_on_a_page_is_recorded_in_the_pages_body_not_its_documents() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlo que se habló.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);

    let loose = served.home.path().join("plano.png");
    std::fs::write(&loose, b"\x89PNG\r\n\x1a\nthe drawing").unwrap();
    let said = served.call(
        "attach",
        json!({ "doc": page, "path": loose.to_string_lossy(), "label": "el plano" }),
    );
    assert!(said["result"]["isError"].is_null(), "{said}");

    let page_body = body_of(&served, &page);
    let parent_body = body_of(&served, &parent);
    assert!(
        page_body.contains("![el plano](<attachments/"),
        "{page_body}"
    );
    assert!(
        !parent_body.contains("attachments/"),
        "the file belongs to the page, not the document it hangs from: {parent_body}"
    );
}

#[test]
fn an_attachment_shared_by_a_page_and_another_document_is_copied_once_and_survives_the_page_leaving()
 {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlo que se habló.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);
    let other = wrote_paper(&served, "# Aparte\n\notro documento.");

    let loose = served.home.path().join("compartido.png");
    std::fs::write(&loose, b"\x89PNG\r\n\x1a\nshared drawing").unwrap();

    let onto_page = served.call(
        "attach",
        json!({ "doc": page, "path": loose.to_string_lossy(), "label": "compartido en la página" }),
    );
    assert!(onto_page["result"]["isError"].is_null(), "{onto_page}");
    let onto_other = served.call(
        "attach",
        json!({ "doc": other, "path": loose.to_string_lossy(), "label": "compartido en el otro" }),
    );
    assert!(onto_other["result"]["isError"].is_null(), "{onto_other}");

    let copies = walked(&served.home.path().join("data/attachments")).count();
    assert_eq!(
        copies, 1,
        "the same bytes are kept once, whoever links to them"
    );

    let taken_out = served.call("page_doc", json!({ "doc": page }));
    assert!(
        taken_out["result"]["structuredContent"]["page_of"].is_null(),
        "{taken_out}"
    );

    let page_body = body_of(&served, &page);
    let other_body = body_of(&served, &other);
    assert!(
        page_body.contains("![compartido en la página](<attachments/"),
        "becoming a document of its own does not touch the page's own body: {page_body}"
    );
    assert!(
        other_body.contains("![compartido en el otro](<attachments/"),
        "the other document's link never depended on the page staying a page: {other_body}"
    );
    assert_eq!(
        walked(&served.home.path().join("data/attachments")).count(),
        1,
        "still one copy after the page leaves"
    );
}

#[test]
#[ignore = "writes a ~60 MB file to disk. Run by name: \
            cargo test -p tisty-cli --test pages_mcp -- --ignored \
            a_page_accepts_a_file_the_task_limit_would_refuse_because_it_uses_the_documents_ceiling"]
fn a_page_accepts_a_file_the_task_limit_would_refuse_because_it_uses_the_documents_ceiling() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlo que se habló.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);

    let heavy = served.home.path().join("charla.mp4");
    let mut bytes = b"    ftypisom".to_vec();
    bytes.resize(60_000_000, 0);
    std::fs::write(&heavy, bytes).unwrap();

    let said = served.call(
        "attach",
        json!({ "doc": page, "path": heavy.to_string_lossy() }),
    );
    assert!(
        said["result"]["isError"].is_null(),
        "a page is a document as far as the attachment ceiling goes, not a task: {said}"
    );
    assert!(body_of(&served, &page).contains("charla.mp4"));
}

#[test]
#[ignore = "writes a ~501 MB file to disk. Run by name: \
            cargo test -p tisty-cli --test pages_mcp -- --ignored \
            an_attachment_past_the_documents_ceiling_is_refused_even_for_a_page"]
fn an_attachment_past_the_documents_ceiling_is_refused_even_for_a_page() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlo que se habló.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);

    let too_heavy = served.home.path().join("pelicula.mp4");
    let mut bytes = b"    ftypisom".to_vec();
    bytes.resize((tisty_core::attach::COPIED_IN_DOC + 1_000_000) as usize, 0);
    std::fs::write(&too_heavy, bytes).unwrap();

    let said = served.call(
        "attach",
        json!({ "doc": page, "path": too_heavy.to_string_lossy() }),
    );
    let why = said["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(said["result"]["isError"], true, "{said}");
    assert!(why.contains("MB"), "{why}");
    assert!(
        walked(&served.home.path().join("data/attachments"))
            .next()
            .is_none(),
        "nothing over the ceiling reaches the store, page or not"
    );
}

#[test]
fn find_and_read_doc_agree_on_which_document_a_page_hangs_from() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Bitácora\n\nel resumen del año.");
    let page = wrote_page(
        &served,
        "# Marzo\n\nregaronpatioxyz mucho ese mes.",
        &parent,
    );

    let found = served.call("find", json!({ "query": "regaronpatioxyz" }));
    let docs = found["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap();
    assert_eq!(docs.len(), 1, "{found}");
    assert_eq!(docs[0]["doc"], page, "{found}");
    assert_eq!(docs[0]["page_of"], parent, "{found}");

    let read = served.call("read_doc", json!({ "doc": parent }));
    let pages = read["result"]["structuredContent"]["pages"]
        .as_array()
        .unwrap();
    assert!(pages.iter().any(|one| one == &page), "{read}");
}

#[test]
fn write_doc_refuses_to_hang_a_page_under_another_page() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);
    let before = docs_on_disk(&served);

    let deeper = served.call(
        "write_doc",
        json!({ "body": "# Anexo\n\nel plano.", "page_of": page }),
    );

    assert_eq!(deeper["result"]["isError"], true, "{deeper}");
    assert_eq!(
        docs_on_disk(&served),
        before,
        "a refusal leaves no .md behind"
    );
}

#[test]
fn page_doc_refuses_to_make_a_document_a_page_of_itself() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let a = wrote_paper(&served, "# A\n\nsolo.");
    let before = docs_on_disk(&served);

    let said = served.call("page_doc", json!({ "doc": a, "page_of": a }));

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(docs_on_disk(&served), before);
    let read = served.call("read_doc", json!({ "doc": a }));
    assert!(
        read["result"]["structuredContent"]["page_of"].is_null(),
        "{read}"
    );
}

#[test]
fn page_doc_refuses_to_turn_a_document_that_holds_pages_into_a_page() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let a = wrote_paper(&served, "# A\n\ntiene páginas.");
    wrote_page(&served, "# Página de A\n\nuna.", &a);
    let b = wrote_paper(&served, "# B\n\notro.");
    let before = docs_on_disk(&served);

    let said = served.call("page_doc", json!({ "doc": a, "page_of": b }));

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(docs_on_disk(&served), before);
    let read = served.call("read_doc", json!({ "doc": a }));
    assert!(
        read["result"]["structuredContent"]["page_of"].is_null(),
        "{read}"
    );
}

#[test]
fn file_doc_refuses_to_file_a_page_into_a_folder_of_its_own() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("folder", json!({ "name": "Casa" }));
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);
    let before = docs_on_disk(&served);

    let filed = served.call("file_doc", json!({ "doc": page, "folder": "Casa" }));

    assert_eq!(filed["result"]["isError"], true, "{filed}");
    assert_eq!(docs_on_disk(&served), before);
}

#[test]
fn write_doc_refuses_a_page_under_a_document_that_was_put_away() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Presupuesto viejo\n\ndel año pasado.");
    served.put_away(&parent);
    let before = docs_on_disk(&served);

    let said = served.call(
        "write_doc",
        json!({ "body": "# Marzo\n\nlo del mes.", "page_of": parent }),
    );

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(
        docs_on_disk(&served),
        before,
        "a refusal leaves no .md behind"
    );
}

#[test]
fn page_doc_refuses_to_hang_a_document_under_one_that_was_put_away() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Presupuesto viejo\n\ndel año pasado.");
    served.put_away(&parent);
    let loose = wrote_paper(&served, "# Anexo\n\nlas cifras.");
    let before = docs_on_disk(&served);

    let said = served.call("page_doc", json!({ "doc": loose, "page_of": parent }));

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(docs_on_disk(&served), before);
}

#[test]
fn write_doc_refuses_a_page_of_naming_a_document_that_does_not_exist() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let before = docs_on_disk(&served);

    let said = served.call(
        "write_doc",
        json!({ "body": "# Marzo\n\nlo del mes.", "page_of": "no-such-doc-0001" }),
    );

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(
        docs_on_disk(&served),
        before,
        "a refusal leaves no .md behind"
    );
}

#[test]
fn page_doc_refuses_a_page_of_naming_a_document_that_does_not_exist() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let a = wrote_paper(&served, "# A\n\nsolo.");
    let before = docs_on_disk(&served);

    let said = served.call(
        "page_doc",
        json!({ "doc": a, "page_of": "no-such-doc-0001" }),
    );

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(docs_on_disk(&served), before);
}

#[test]
fn write_doc_stops_at_the_document_ceiling_and_pages_count_toward_it() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.flood_docs(499);

    let five_hundredth = served.call(
        "write_doc",
        json!({ "body": "# El último que cabe\n\njusto entra." }),
    );
    assert!(
        five_hundredth["result"]["isError"].is_null(),
        "499 filler documents plus one real write is 500, still within the ceiling: {five_hundredth}"
    );

    let refused = served.call(
        "write_doc",
        json!({ "body": "# El que no cabe\n\nya no hay sitio." }),
    );
    let why = refused["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    assert!(
        why.contains("500"),
        "the refusal has to say the number: {why}"
    );
}

#[test]
fn write_doc_refuses_a_page_with_an_empty_body() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");
    let before = docs_on_disk(&served);

    let said = served.call("write_doc", json!({ "body": "", "page_of": parent }));

    let why = said["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(said["result"]["isError"], true, "{said}");
    assert!(why.contains("body"), "{why}");
    assert_eq!(docs_on_disk(&served), before);
}

#[test]
fn a_page_titled_with_only_an_emoji_is_written_with_an_empty_title() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");

    let made = served.call("write_doc", json!({ "body": "🎉", "page_of": parent }));
    assert!(made["result"]["isError"].is_null(), "{made}");
    let page = made["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    // An emoji carries no alphanumeric character, so `titled` finds no line to call a title —
    // this is shared behaviour with every document, not something specific to pages.
    assert_eq!(made["result"]["structuredContent"]["title"], "");
    let read = served.call("read_doc", json!({ "doc": page }));
    assert_eq!(read["result"]["structuredContent"]["title"], "");
    assert!(body_of(&served, &page).contains('🎉'));
}

#[test]
fn a_page_body_stops_at_what_the_assistant_may_send_long_before_the_reading_ceiling() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");

    let said = 64_000;
    let head = "# T\n\n";
    let brim = format!("{head}{}\n", "a".repeat(said - head.len() - 1));
    let over = format!("{brim}a");

    let made = served.call("write_doc", json!({ "body": brim, "page_of": parent }));
    let refused = served.call("write_doc", json!({ "body": over, "page_of": parent }));

    assert!(made["result"]["isError"].is_null(), "{made}");
    let page = made["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(body_of(&served, &page).chars().count(), said);
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    assert!(
        tisty_core::docs::BODY_AT_MOST as usize > said,
        "the ceiling an assistant meets is the one on what it sends, not the one on the file"
    );
}

#[test]
fn a_body_too_big_to_send_is_refused_without_leaving_an_empty_file_behind() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");
    let before = docs_on_disk(&served);

    let limit = tisty_core::docs::BODY_AT_MOST as usize;
    let head = "# T\n\n";
    let filler = "a".repeat(limit - head.len());
    let body = format!("{head}{filler}\n");
    assert_eq!(body.len(), limit + 1);

    let said = served.call("write_doc", json!({ "body": body, "page_of": parent }));

    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(
        docs_on_disk(&served),
        before,
        "a refusal must not leave an empty .md file behind"
    );
}

#[test]
fn append_doc_and_edit_doc_reach_a_pages_own_body_and_never_its_documents() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Acta\n\ntexto original del padre.");
    let page = wrote_page(&served, "# Marzo\n\nlo que se dijo.", &parent);

    let added = served.call(
        "append_doc",
        json!({ "doc": page, "body": "Se sumó esto." }),
    );
    assert!(added["result"]["isError"].is_null(), "{added}");
    assert!(body_of(&served, &page).contains("Se sumó esto."));
    assert!(!body_of(&served, &parent).contains("Se sumó esto."));
    assert!(body_of(&served, &parent).contains("texto original del padre."));

    let edited = served.call(
        "edit_doc",
        json!({ "doc": page, "old": "lo que se dijo.", "new": "lo que realmente se dijo." }),
    );
    assert!(edited["result"]["isError"].is_null(), "{edited}");
    assert!(body_of(&served, &page).contains("lo que realmente se dijo."));
    assert!(body_of(&served, &parent).contains("texto original del padre."));
    assert!(!body_of(&served, &parent).contains("lo que"));
}

#[test]
fn an_attachment_link_in_a_page_survives_an_unrelated_edit_doc_call() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Actas\n\nlas de este año.");
    let page = wrote_page(
        &served,
        "# Marzo\n\nprimera línea.\n\nsegunda línea.",
        &parent,
    );

    let loose = served.home.path().join("nota.png");
    std::fs::write(&loose, b"\x89PNG\r\n\x1a\nuna nota").unwrap();
    let attached = served.call(
        "attach",
        json!({ "doc": page, "path": loose.to_string_lossy(), "label": "la nota" }),
    );
    assert!(attached["result"]["isError"].is_null(), "{attached}");
    assert!(body_of(&served, &page).contains("![la nota](<attachments/"));

    let edited = served.call(
        "edit_doc",
        json!({ "doc": page, "old": "primera línea.", "new": "primera línea editada." }),
    );
    assert!(edited["result"]["isError"].is_null(), "{edited}");

    let whole = body_of(&served, &page);
    assert!(whole.contains("primera línea editada."), "{whole}");
    assert!(
        whole.contains("![la nota](<attachments/"),
        "an edit elsewhere in the page must not disturb the attachment link: {whole}"
    );
}

#[test]
fn putting_a_document_away_archives_its_pages_with_it() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let parent = wrote_paper(&served, "# Presupuesto\n\ndel año.");
    let page = wrote_page(&served, "# Marzo\n\nlo del mes.", &parent);
    served.put_away(&parent);

    let read = served.call("read_doc", json!({ "doc": page }));
    assert_eq!(
        read["result"]["structuredContent"]["archived"], true,
        "{read}"
    );
    assert!(
        read["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("put away"),
        "{read}"
    );

    let listed = served.call("docs", json!({}));
    let row = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["doc"] == page)
        .unwrap();
    assert_eq!(row["archived"], true, "{listed}");
}

#[test]
fn a_page_written_with_an_explicit_folder_keeps_its_documents_folder_instead() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("folder", json!({ "name": "Casa" }));
    served.call("folder", json!({ "name": "Trabajo" }));
    let parent = served.call(
        "write_doc",
        json!({ "body": "# Actas\n\nlas de este año.", "folder": "Casa" }),
    );
    let parent = parent["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    let page = served.call(
        "write_doc",
        json!({ "body": "# Marzo\n\nlo del mes.", "page_of": parent, "folder": "Trabajo" }),
    );

    assert!(page["result"]["isError"].is_null(), "{page}");
    assert_eq!(
        page["result"]["structuredContent"]["folder"], "Casa",
        "a page's folder comes from its document, and an explicit `folder` is not honoured: {page}"
    );
}
