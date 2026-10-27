#![cfg(feature = "tables")]

use tisty_markdown::{html, printed, read, to_standard, to_tisty};

const SUBSCRIPTIONS: &str = "| Servicio | Categoría {select} | Precio {money CLP sum} | Próximo cobro {date min} | Valoración {rating avg} | Activa {check count} |
| -------- | ---------- | --------: | ---------- | ---------- | :----: |
| Spotify | Música | 6490 | 2026-11-14 | 5 | [x] |
| Netflix \\| 4K | Video | 9990 | 2026-11-03 | 3 | [x] |
| Duolingo | Educación | 7.990 | pronto | 2 | [ ] |
";

fn standard(text: &str) -> String {
    to_standard(text, &|_| None)
}

#[test]
fn a_typed_table_drawn_as_html() {
    insta::assert_snapshot!(html(&read(SUBSCRIPTIONS)));
}

#[test]
fn a_typed_table_in_standard_markdown() {
    insta::assert_snapshot!(standard(SUBSCRIPTIONS));
}

#[test]
fn a_typed_table_in_a_quote_keeps_its_foot_inside_the_quote() {
    insta::assert_snapshot!(standard(
        "> | Horas {duration sum} |\n> | --- |\n> | 1h30m |\n> | 45m |\n"
    ));
}

#[test]
fn a_typed_table_printed_for_word() {
    insta::assert_json_snapshot!(printed(&read(SUBSCRIPTIONS)));
}

#[test]
fn reading_a_typed_table_leaves_every_byte_as_written() {
    let doc = read(SUBSCRIPTIONS);
    assert_eq!(doc.text(), SUBSCRIPTIONS);
    assert_eq!(to_tisty(SUBSCRIPTIONS), SUBSCRIPTIONS);
}

// One table per case, split by a blank line: none has a type Tisty knows, so none changes on the way out.
const UNTYPED: &str = "| a | b |\n| --- | :-: |\n| 1 | 2 |\n
| Notas {importante} | Precio{money} |\n| ------- | --: |\n| x \\| y | 1 |\n
| `{money}` |\n| --- |\n| 1 |\n";

#[test]
fn a_table_with_no_type_tisty_knows_leaves_unchanged_byte_for_byte() {
    for table in UNTYPED.split("\n\n") {
        let table = format!("{}\n", table.trim_end());
        assert_eq!(standard(&table), table);
        assert_eq!(to_tisty(&table), table);
    }
}

#[test]
fn a_foot_added_on_the_way_out_ends_its_lines_as_the_table_does() {
    let crlf = standard("| Horas {duration sum} |\r\n| --- |\r\n| 1h |\r\n| 30m |\r\n");
    assert!(crlf.contains("\r\n| 1 h 30 min |"));
    assert!(!crlf.replace("\r\n", "").contains('\n'));
}
