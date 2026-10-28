use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        let person = person::Person::at(served.home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        served
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
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
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
    }

    fn wrote(&self, body: &str) -> String {
        let made = self.call("write_doc", json!({ "body": body }));
        made["result"]["structuredContent"]["doc"]
            .as_str()
            .unwrap_or_else(|| panic!("{made}"))
            .to_string()
    }

    fn body(&self, doc: &str) -> String {
        self.call("read_doc", json!({ "doc": doc }))["result"]["structuredContent"]["body"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn table(&self, doc: &str, args: Value) -> Value {
        let mut args = args;
        args["doc"] = json!(doc);
        self.call("edit_table", args)["result"].clone()
    }
}

fn said(result: &Value) -> &str {
    result["content"][0]["text"].as_str().unwrap()
}

const SHOPPING: &str = "# Compras

## Despensa

| Producto | Precio {money CLP sum} | Llegó {date} | Listo {check} |
| --- | --- | --- | --- |
| Arroz | 1290 | 2026-10-01 | [x] |
| Aceite | 4590 | 2026-10-03 | [ ] |
| Café | 7990 |  | [ ] |
";

#[test]
fn a_table_is_shown_by_its_columns_with_their_types_and_its_rows_by_their_first_cell() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    let shown = served.table(&doc, json!({}));

    assert_eq!(shown["isError"], Value::Null, "{shown}");
    let shape = &shown["structuredContent"];
    assert_eq!(shape["heading"], "Despensa");
    assert_eq!(shape["rows"], 3);
    assert_eq!(shape["first_cells"], json!(["Arroz", "Aceite", "Café"]));
    assert_eq!(
        shape["columns"],
        json!([
            { "title": "Producto", "type": "" },
            { "title": "Precio", "type": "money CLP sum" },
            { "title": "Llegó", "type": "date" },
            { "title": "Listo", "type": "check" },
        ])
    );
    assert!(said(&shown).contains("Precio {money CLP sum}"), "{shown}");
    assert_eq!(served.body(&doc), SHOPPING, "showing writes nothing");
}

#[test]
fn a_cell_is_set_by_its_row_s_first_cell_and_its_column_title_in_its_type_s_form() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    for (row, column, value) in [
        ("café", "listo", "sí"),
        ("Aceite", "Precio", "4 990"),
        ("Café", "Llegó", "2026-10-05"),
    ] {
        let made = served.table(
            &doc,
            json!({ "do": "set", "row": row, "column": column, "value": value }),
        );
        assert_eq!(made["isError"], Value::Null, "{made}");
        assert_eq!(made["structuredContent"]["amiss"], json!([]), "{made}");
    }

    insta::assert_snapshot!(served.body(&doc));
}

#[test]
fn a_value_its_type_cannot_read_is_kept_as_sent_and_named_in_the_answer() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    let made = served.table(
        &doc,
        json!({ "do": "add", "values": { "Producto": "Té", "Precio": 2490, "Llegó": "pronto" } }),
    );

    assert_eq!(
        made["structuredContent"]["amiss"],
        json!([{ "row": "Té", "column": "Llegó", "value": "pronto", "type": "date" }]),
        "{made}"
    );
    assert!(
        said(&made).contains("\"pronto\" in Llegó of \"Té\""),
        "{made}"
    );
    assert!(
        served.body(&doc).contains("| Té | 2490 | pronto |"),
        "{}",
        served.body(&doc)
    );
}

#[test]
fn a_column_given_a_type_names_the_values_already_there_that_do_not_fit_it() {
    let served = Served::new();
    let doc = served.wrote(
        "| Tarea | Horas |
| --- | --- |
| Diseño | 4 |
| Pruebas | varias |
",
    );

    let made = served.table(
        &doc,
        json!({ "do": "type", "column": "Horas", "type": "number" }),
    );

    assert_eq!(
        made["structuredContent"]["amiss"],
        json!([{ "row": "Pruebas", "column": "Horas", "value": "varias", "type": "number" }]),
        "{made}"
    );
    assert!(
        served.body(&doc).contains("Horas {number}"),
        "{}",
        served.body(&doc)
    );
}

#[test]
fn rows_and_columns_are_added_moved_and_taken_out_without_touching_what_is_around() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    for asked in [
        json!({ "do": "sort", "column": "Precio", "down": true }),
        json!({ "do": "rename", "column": 1, "title": "Artículo" }),
        json!({ "do": "add_column", "title": "Nota" }),
        json!({ "do": "remove", "row": "Aceite" }),
        json!({ "do": "remove_column", "column": "Llegó" }),
    ] {
        let made = served.table(&doc, asked.clone());
        assert_eq!(made["isError"], Value::Null, "{asked} → {made}");
    }

    insta::assert_snapshot!(served.body(&doc));
}

#[test]
fn among_several_tables_one_is_named_by_its_number_or_the_heading_over_it() {
    let served = Served::new();
    let doc = served.wrote(
        "# Viaje

## Ida

| Tramo | Hora |
| --- | --- |
| Bus | 08:00 |

## Vuelta

| Tramo | Hora |
| --- | --- |
| Tren | 19:30 |
",
    );

    let unnamed = served.table(&doc, json!({}));
    assert_eq!(unnamed["isError"], true);
    assert!(
        said(&unnamed).contains("1 under \"Ida\": Tramo, Hora; 2 under \"Vuelta\""),
        "{unnamed}"
    );

    let by_heading = served.table(&doc, json!({ "table": "vuelta" }));
    assert_eq!(
        by_heading["structuredContent"]["first_cells"],
        json!(["Tren"])
    );
    let by_number = served.table(&doc, json!({ "table": 1 }));
    assert_eq!(
        by_number["structuredContent"]["first_cells"],
        json!(["Bus"])
    );
}

#[test]
fn what_cannot_be_done_is_refused_saying_what_would_work_and_nothing_is_written() {
    let served = Served::new();
    let doc = served.wrote(
        "| Nombre | Rol |
| --- | --- |
| Ana | Diseño |
| Ana | Pruebas |
| Luis | Datos |
",
    );
    let plain = served.wrote("# Notas\n\nSin tablas.\n");
    let before = served.body(&doc);

    for (asked, told) in [
        (
            json!({ "do": "set", "row": "Ana", "column": "Rol", "value": "x" }),
            "2 rows start with \"Ana\", at 1, 2",
        ),
        (
            json!({ "do": "remove", "row": "Pedro" }),
            "Its first cells are: Ana, Ana, Luis",
        ),
        (json!({ "do": "remove", "row": 9 }), "this table has 3 rows"),
        (
            json!({ "do": "sort", "column": "Edad" }),
            "This table's columns are: Nombre, Rol",
        ),
        (
            json!({ "do": "type", "column": "Rol", "type": "colour" }),
            "not one Tisty knows",
        ),
        (
            json!({ "do": "set", "row": "Luis", "column": "Rol" }),
            "this change needs `value`",
        ),
        (json!({ "do": "add" }), "`add` needs `values`"),
        (json!({ "do": "merge" }), "not \"merge\""),
    ] {
        let refused = served.table(&doc, asked.clone());
        assert_eq!(refused["isError"], true, "{asked}");
        assert!(said(&refused).contains(told), "{asked} → {refused}");
    }
    assert_eq!(served.body(&doc), before);

    let none = served.table(&plain, json!({}));
    assert!(said(&none).contains("holds no table"), "{none}");
}

#[test]
fn a_table_keeps_its_last_column() {
    let served = Served::new();
    let doc = served.wrote("| Solo |\n| --- |\n| uno |\n");

    let refused = served.table(&doc, json!({ "do": "remove_column", "column": "Solo" }));

    assert!(said(&refused).contains("at least one column"), "{refused}");
    assert_eq!(served.body(&doc), "| Solo |\n| --- |\n| uno |\n");
}

#[test]
fn the_language_s_tables_topic_tells_the_column_types_and_counts_as_tisty_s_own() {
    let served = Served::new();

    let topic = served.call("language", json!({ "topic": "tables" }))["result"].clone();
    let index = served.call("language", json!({}))["result"].clone();

    assert!(said(&topic).contains("### Column types"), "{topic}");
    let (added, plain) = said(&index).split_once("Markdown as anywhere").unwrap();
    assert!(added.contains("- tables:"), "{index}");
    assert!(!plain.contains("- tables:"), "{index}");
}

#[test]
fn rows_and_columns_are_named_whatever_their_capitals_and_accents() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    let made = served.table(
        &doc,
        json!({ "do": "set", "row": "CAFE", "column": "llego", "value": "2026-10-09" }),
    );

    assert_eq!(made["isError"], Value::Null, "{made}");
    assert!(served.body(&doc).contains("| Café | 7990 | 2026-10-09 |"));
}

#[test]
fn a_value_in_another_markdown_comes_in_as_the_tisty_language() {
    let served = Served::new();
    let doc = served.wrote("| Tarea | Nota |\n| --- | --- |\n| Diseño | lista |\n");

    let made = served.table(
        &doc,
        json!({ "do": "set", "row": "Diseño", "column": "Nota", "value": "<mark>urgente</mark>" }),
    );

    assert_eq!(made["isError"], Value::Null, "{made}");
    let body = served.body(&doc);
    assert!(body.contains("| Diseño | ==urgente== |"), "{body}");
    assert!(made["structuredContent"]["print"].is_string(), "{made}");
    assert_eq!(made["structuredContent"]["loose"], json!([]), "{made}");
    assert!(
        made["structuredContent"]["turned"]
            .as_array()
            .is_some_and(|one| !one.is_empty()),
        "{made}"
    );
    assert!(
        said(&made).contains("was written in the Tisty language"),
        "{made}"
    );
}

#[test]
fn a_number_sent_as_words_names_a_place_when_nothing_is_called_so() {
    let served = Served::new();
    let doc = served.wrote(
        "## Ida

| Tramo | Hora |
| --- | --- |
| Bus | 08:00 |

## Vuelta

| Tramo | Hora |
| --- | --- |
| Tren | 19:30 |
| 1 | 20:00 |
",
    );

    let made = served.table(
        &doc,
        json!({ "table": "2", "do": "set", "row": "2", "column": "2", "value": "21:00" }),
    );
    assert_eq!(made["isError"], Value::Null, "{made}");
    assert!(
        served.body(&doc).contains("| 1 | 21:00 |"),
        "{}",
        served.body(&doc)
    );

    let by_words = served.table(&doc, json!({ "table": 2, "do": "remove", "row": "1" }));
    assert_eq!(by_words["isError"], Value::Null, "{by_words}");
    let body = served.body(&doc);
    assert!(
        body.contains("| Tren | 19:30 |") && !body.contains("| 1 | 21:00 |"),
        "{body}"
    );
}

#[test]
fn a_row_that_named_a_page_tells_the_page_is_left_loose_when_it_goes() {
    let served = Served::new();
    let page = served.wrote(
        "# Marzo

lo de marzo.",
    );
    let book = served.wrote(&format!(
        "# Actas

| Mes | Acta |
| --- | --- |
| Marzo | ![Marzo](tisty:doc/{page}) |
| Abril | nada |
"
    ));
    let hung = served.call("page_doc", json!({ "doc": page, "page_of": book }));
    assert_eq!(hung["result"]["isError"], Value::Null, "{hung}");
    let body = served.body(&book);
    assert_eq!(
        body.matches(&format!("tisty:doc/{page}")).count(),
        1,
        "the table's card already names it: {body}"
    );

    let made = served.table(&book, json!({ "do": "remove", "row": "Marzo" }));

    assert_eq!(made["structuredContent"]["loose"], json!([page]), "{made}");
    assert!(said(&made).contains("Marzo"), "{made}");
}

#[test]
fn a_type_is_changed_only_when_one_is_sent_and_an_empty_one_takes_it_away() {
    let served = Served::new();
    let doc = served.wrote(SHOPPING);

    let refused = served.table(&doc, json!({ "do": "type", "column": "Precio" }));
    assert!(
        said(&refused).contains("this change needs `type`"),
        "{refused}"
    );
    assert_eq!(served.body(&doc), SHOPPING);

    let untyped = served.table(
        &doc,
        json!({ "do": "type", "column": "Precio", "type": "" }),
    );
    assert_eq!(untyped["isError"], Value::Null, "{untyped}");
    let body = served.body(&doc);
    assert!(
        body.contains("| Producto | Precio                 | Llegó {date} |"),
        "the cell keeps its width: {body}"
    );
}

#[test]
fn a_heading_two_tables_share_names_neither_of_them() {
    let served = Served::new();
    let doc = served.wrote(
        "## Semana\n\n| Día | Hora |\n| --- | --- |\n| Lunes | 9 |\n\n| Día | Sala |\n| --- | --- |\n| Martes | 2 |\n",
    );

    let refused = served.table(&doc, json!({ "table": "semana", "do": "remove", "row": 1 }));

    assert!(
        said(&refused).contains("2 tables sit under \"semana\""),
        "{refused}"
    );
    assert!(served.body(&doc).contains("| Lunes | 9 |"));
}
