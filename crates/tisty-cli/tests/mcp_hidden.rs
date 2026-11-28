use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const NOBODY: &str = "t_01j00000000000000000000000-0001";
const NAMES_A_DOCUMENT: [&str; 10] = [
    "doc",
    "docs",
    "page_of",
    "parent",
    "translation_of",
    "of",
    "after",
    "before",
    "order",
    "instead",
];
const MISSING: &str = "zzzzzzzz-0001";
// Each key a tool takes, with the JSON types it accepts.
type Keys = Vec<(String, Vec<String>)>;

const SECRET: [&str; 3] = ["secret zebra", "Nota uno", "Hoja escondida"];

struct Served {
    home: TempDir,
    task: String,
    page: String,
    under_it: String,
    open_page: String,
    other: String,
    file: String,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let mut served = Self {
            home,
            task: String::new(),
            page: String::new(),
            under_it: String::new(),
            open_page: String::new(),
            other: String::new(),
            file: String::new(),
        };
        (served.task, served.page) = served.task_with_a_page("secret zebra plan", "Nota uno.");
        (_, served.open_page) = served.task_with_a_page("plan a la vista", "Nota dos.");
        served.other = served.person().writes_doc("# Planes\n\nAlgo.\n");
        served.under_it = served.person().hangs_page_under(
            &served.page,
            &format!(
                "# Hoja escondida\n\nNota uno. Ver [Planes](tisty:doc/{}).\n",
                served.other
            ),
        );
        let file = served.home.path().join("hoja.md");
        std::fs::write(&file, "# Hoja colgada\n\nTexto.\n").unwrap();
        served.file = file.to_string_lossy().into_owned();
        served.person().hides(person::task(&served.task));
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn task_with_a_page(&self, title: &str, note: &str) -> (String, String) {
        let made = self.call(
            "task_create",
            json!({ "for": "agent", "title": title, "list": "Personal" }),
        );
        let task = made["structuredContent"]["id"]
            .as_str()
            .unwrap()
            .to_string();
        let added = self.call(
            "doc_edit",
            json!({ "mode": "append", "doc": task, "content": note }),
        );
        assert_ne!(added["isError"], true, "{added}");
        let read = self.call("workspace_read", json!({ "task": task }));
        let page = read["structuredContent"]["document"]["file"]
            .as_str()
            .unwrap()
            .to_string();
        (task, page)
    }

    fn call(&self, name: &str, args: Value) -> Value {
        self.ask("tools/call", json!({ "name": name, "arguments": args }))
    }

    fn ask(&self, method: &str, params: Value) -> Value {
        let root = self.home.path();
        let asked =
            json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{asked}").unwrap();
        let out = child.wait_with_output().unwrap();
        let said: Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        said["result"].clone()
    }

    fn body(&self, file: &str) -> String {
        let root = self.home.path();
        let paths = tisty_core::Paths::new(root.join("data"), root.join("config"));
        tisty_core::docs::read(&paths.docs(), file).unwrap_or_default()
    }

    fn ops(&self) -> usize {
        self.person().ops().len()
    }

    fn tools_that_write(&self) -> Vec<(String, Keys)> {
        let listed = self.ask("tools/list", json!({}));
        listed["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|one| one["annotations"]["readOnlyHint"] != true)
            .map(|one| {
                let taken = one["inputSchema"]["properties"].as_object().unwrap();
                let keys = taken
                    .iter()
                    .map(|(key, shape)| (key.clone(), kinds(shape)))
                    .collect();
                (one["name"].as_str().unwrap().to_string(), keys)
            })
            .collect()
    }

    fn answers_as_nobody(&self, name: &str, args: impl Fn(&str) -> Value) {
        let wrong = self.told_apart(&self.page, NOBODY, name, args);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    // What the person hid and what was never there take the same road: same answer, same writes.
    fn told_apart(
        &self,
        hidden_one: &str,
        nobody_one: &str,
        name: &str,
        args: impl Fn(&str) -> Value,
    ) -> Vec<String> {
        let before = (self.body(&self.page), self.body(&self.under_it));
        let at_first = self.ops();
        let hidden = self.call(name, args(hidden_one)).to_string();
        let after_hidden = self.ops();
        let nobody = self.call(name, args(nobody_one)).to_string();
        let after_nobody = self.ops();
        let asked = args(hidden_one);

        let mut wrong = Vec::new();
        if SECRET.iter().any(|word| hidden.contains(word)) {
            wrong.push(format!("{name} {asked} named hidden words: {hidden}"));
        }
        if hidden.replace(hidden_one, nobody_one) != nobody {
            wrong.push(format!(
                "{name} {asked} told hidden from nothing:\n  {hidden}\n  {nobody}"
            ));
        }
        if (self.body(&self.page), self.body(&self.under_it)) != before {
            wrong.push(format!("{name} {asked} wrote on a hidden task's words"));
        }
        if after_hidden - at_first != after_nobody - after_hidden {
            wrong.push(format!("{name} {asked} wrote what nothing would not"));
        }
        wrong
    }
}

fn kinds(shape: &Value) -> Vec<String> {
    match (&shape["type"], shape.get("anyOf").or(shape.get("oneOf"))) {
        (Value::String(one), _) => vec![one.clone()],
        (Value::Array(many), _) => many
            .iter()
            .filter_map(|one| one.as_str().map(String::from))
            .collect(),
        (_, Some(Value::Array(many))) => many.iter().flat_map(kinds).collect(),
        _ => Vec::new(),
    }
}

fn around(
    served: &Served,
    name: &str,
    keys: &[(String, Vec<String>)],
    key: &str,
    said: Value,
) -> Value {
    let takes = |one: &str| one != key && keys.iter().any(|(k, _)| k == one);
    let mut args = serde_json::Map::new();
    args.insert(key.to_string(), said);
    match name {
        "doc_edit" => {
            args.insert("mode".into(), json!("patch"));
            args.insert("old".into(), json!("Nota uno."));
            args.insert("content".into(), json!("Otra."));
        }
        "doc_edit_table" => {
            args.insert("action".into(), json!("set"));
        }
        "doc_organize" => {
            let action = match key {
                "order" => "reorder_pages",
                "docs" | "parent" | "after" | "before" => "add_page",
                _ => "file",
            };
            args.insert("action".into(), json!(action));
            if key != "parent" {
                args.insert("parent".into(), json!(served.other));
            }
            if key != "docs" && action == "add_page" {
                args.insert("docs".into(), json!([served.other]));
            }
        }
        "doc_annotate" => {
            args.insert("action".into(), json!("flag"));
            if key != "docs" {
                args.insert("docs".into(), json!([served.other]));
            }
            args.insert("reason".into(), json!("moved"));
        }
        "file_transfer" => {
            args.insert("action".into(), json!("attach"));
            args.insert("path".into(), json!(served.file));
        }
        _ => {}
    }
    for (one, value) in [
        ("doc", json!(served.other)),
        ("body", json!("# Hoja\n\nAlgo.\n")),
        ("path", json!(served.file)),
        ("anyway", json!(true)),
        ("reason", json!("moved")),
        ("summary", json!("Un resumen.")),
    ] {
        if takes(one) && !args.contains_key(one) {
            args.insert(one.to_string(), value);
        }
    }
    Value::Object(args)
}

#[test]
fn every_tool_that_writes_answers_a_hidden_task_s_page_as_a_page_nobody_has() {
    let served = Served::new();
    let tools = served.tools_that_write();
    assert_eq!(tools.len(), 11, "{tools:?}");
    let mut tried = 0;
    let mut wrong = Vec::new();
    for (name, keys) in &tools {
        for (key, shapes) in keys
            .iter()
            .filter(|(key, _)| NAMES_A_DOCUMENT.contains(&key.as_str()))
        {
            for (hidden, nobody) in [(&served.page, NOBODY), (&served.under_it, MISSING)] {
                for (shape, wrapped) in [("string", false), ("array", true)] {
                    if !shapes.iter().any(|one| one == shape) {
                        continue;
                    }
                    wrong.extend(served.told_apart(hidden, nobody, name, |one| {
                        let said = if wrapped { json!([one]) } else { json!(one) };
                        around(&served, name, keys, key, said)
                    }));
                    tried += 1;
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(tried >= 20, "only {tried} calls reached a document name");
}

#[test]
fn a_page_is_never_hung_under_a_hidden_task_s_page_by_import() {
    let served = Served::new();
    let page_before = served.body(&served.page);

    served.answers_as_nobody(
        "file_transfer",
        |page| json!({ "action": "import_doc", "path": served.file, "page_of": page, "anyway": true }),
    );

    assert_eq!(served.body(&served.page), page_before);
    let state = served.person().state();
    let hung = state.docs.values().filter(|one| one.page_of.is_some());
    assert_eq!(hung.count(), 1, "a page was hung under a task's page");
}

#[test]
fn a_task_s_page_written_by_name_never_hangs_from_a_hidden_one() {
    let served = Served::new();
    let open_before = served.body(&served.open_page);

    served.answers_as_nobody(
        "doc_create",
        |page| json!({ "page_of": page, "body": open_before }),
    );
    served.answers_as_nobody(
        "file_transfer",
        |page| json!({ "action": "import_doc", "path": served.file, "page_of": page }),
    );
}

#[test]
fn an_open_task_s_page_takes_no_page_by_import_either() {
    let served = Served::new();

    let hung = served.call(
        "file_transfer",
        json!({ "action": "import_doc", "path": served.file, "page_of": served.open_page, "anyway": true }),
    );

    assert_eq!(hung["isError"], true, "{hung}");
    assert!(hung.to_string().contains("is a task's page"), "{hung}");
    assert!(!served.body(&served.open_page).contains("Hoja colgada"));
}

#[test]
fn a_file_naming_a_hidden_task_comes_in_as_one_naming_no_task() {
    let served = Served::new();
    let page_body = served.body(&served.page);
    let nobody_id = "01J00000000000000000000000";
    let bodies = [
        page_body.clone(),
        page_body.replace(&served.task, nobody_id),
    ];
    assert_ne!(bodies[0], bodies[1], "the page names its task");
    let task_before = served.person().state().tasks[&person::task(&served.task)].clone();

    for (at, body) in bodies.iter().enumerate() {
        let file = served.home.path().join(format!("pagina-{at}.md"));
        std::fs::write(&file, body).unwrap();
        let brought = served.call(
            "file_transfer",
            json!({ "action": "import_doc", "path": file.to_string_lossy(), "anyway": true }),
        );
        assert!(!brought.to_string().contains("secret zebra"), "{brought}");
        assert_ne!(brought["isError"], true, "{body}: {brought}");
    }

    assert_eq!(served.body(&served.page), page_body);
    let task_after = &served.person().state().tasks[&person::task(&served.task)];
    assert_eq!(task_after.title, task_before.title);
    assert_eq!(task_after.log.len(), task_before.log.len());
}

#[test]
fn a_page_hung_under_a_hidden_task_s_page_is_read_as_one_nobody_has() {
    let served = Served::new();

    let home = served.home.path().to_string_lossy().into_owned();
    for (name, more) in [
        ("workspace_read", json!({})),
        ("workspace_read", json!({ "view": "card" })),
        (
            "workspace_search",
            json!({ "mode": "in_doc", "query": "Nota" }),
        ),
        (
            "file_transfer",
            json!({ "action": "export_doc", "path": home }),
        ),
    ] {
        let with = |one: &str| {
            let mut args = more.clone();
            args["doc"] = json!(one);
            args
        };
        let wrong = served.told_apart(&served.under_it, MISSING, name, with);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
    for (hidden, nobody) in [(&served.page, NOBODY), (&served.under_it, MISSING)] {
        let wrong = served.told_apart(
            hidden,
            nobody,
            "doc_create",
            |one| json!({ "page_of": one, "body": "# Hoja\n\nAlgo.\n" }),
        );
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
    for (name, args) in [
        (
            "workspace_read",
            json!({ "doc": served.other, "view": "card" }),
        ),
        (
            "workspace_read",
            json!({ "doc": served.other, "view": "links", "links": "all" }),
        ),
        ("workspace_read", json!({ "doc": served.other })),
        ("workspace_search", json!({})),
        ("workspace_search", json!({ "scope": "either" })),
        (
            "workspace_search",
            json!({ "query": "escondida", "scope": "either" }),
        ),
        ("workspace_catch_up", json!({})),
    ] {
        let said = served.call(name, args.clone()).to_string();
        for word in SECRET {
            assert!(!said.contains(word), "{name} {args}: {said}");
        }
        assert!(!said.contains(&served.under_it), "{name} {args}: {said}");
    }
    let read = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{}", served.under_it) }),
    );
    let nothing = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{MISSING}") }),
    );
    assert_eq!(
        read.to_string().replace(&served.under_it, MISSING),
        nothing.to_string()
    );
}

#[test]
fn a_task_the_person_let_go_is_never_named_by_its_source() {
    let served = Served::new();
    let made = served.call(
        "task_create",
        json!({ "for": "agent", "title": "secret zebra gift", "source": "mail:77", "list": "Personal" }),
    );
    let task = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().drops(person::task(&task));

    let found = served.call(
        "workspace_search",
        json!({ "mode": "source", "source": "mail:77" }),
    );
    let proposed = served.call(
        "task_create",
        json!({ "for": "agent", "title": "otra", "source": "mail:77", "list": "Personal" }),
    );

    for said in [&found, &proposed] {
        assert!(!said.to_string().contains("secret zebra"), "{said}");
        assert!(!said.to_string().contains(&task), "{said}");
        assert!(said.to_string().contains("let it go"), "{said}");
    }
    assert_eq!(
        found["structuredContent"]["found"]["let_go"], true,
        "{found}"
    );
    assert_eq!(
        proposed["structuredContent"]["proposed"], false,
        "{proposed}"
    );
    let again = served.call(
        "task_create",
        json!({ "for": "agent", "title": "otra vez", "source": "mail:77", "list": "Personal", "again": true }),
    );
    assert_eq!(again["structuredContent"]["proposed"], true, "{again}");
}
