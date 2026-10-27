use std::ops::Range;

use super::*;
use crate::read;

fn table_in(blocks: &[Block]) -> Option<Range<usize>> {
    blocks.iter().find_map(|one| match one.kind {
        Kind::Table { .. } => Some(one.at.clone()),
        _ => table_in(&one.inner),
    })
}

fn edited(text: &str, asked: &Edit) -> Result<String, Refused> {
    let at = table_in(read(text).blocks()).expect("a table");
    edit(text, at, asked)
}

// Tables as people write them, split by a line of `~`, so a new one is a few new lines.
const WRITTEN: &str = "| Vaccine        | Given | Due again |
| :------------- | :---: | --------: |
| Six-in-one     | march | 12 months |
| Rabies         | march | 12 months |
~
| Servicio | Precio {money CLP sum} | Nota \\| x |
| -------- | ---------------------: | ---- |
| Netflix \\| 4K | 9990 | `a \\| b` |
| Vacío |  |  |
~
a | b
--- | ---
1 | 2
~
| a | b |
| --- | --- |
| 1 | 2 |
~
> | Horas {duration sum} |
> | --- |
> | 1h30m |
~
- Una lista:

  | a | b |
  | - | - |
  | 1 |
~
| a | b |\r
| --- | --- |\r
| 1 | 2 |\r
";

#[test]
fn a_table_read_and_written_untouched_is_the_same_bytes() {
    for table in WRITTEN.split("~\n") {
        let said = table.trim_end_matches('\n');
        let said = said.strip_suffix('\r').unwrap_or(said);
        assert_eq!(Lines::read(said).written(), said, "{table}");
        let moved = edited(table, &Edit::RowMove { from: 0, to: 0 }).expect(table);
        assert_eq!(moved, table, "{table}");
    }
}

// Each case: `@` and the edit as the window sends it, the table, `=>`, and the table after or why not.
const EDITS: &str = r#"
@ {"kind":"cell","row":0,"column":1,"said":"9990"}
| Servicio | Precio |
| -------- | -----: |
| Spotify  | 6490   |
=>
| Servicio | Precio |
| -------- | -----: |
| Spotify  | 9990   |
@ {"kind":"cell","row":0,"column":0,"said":"Netflix | 4K\nUHD"}
| Servicio | Precio |
| --- | --- |
| x | 1 |
=>
| Servicio | Precio |
| --- | --- |
| Netflix \| 4K UHD | 1 |
@ {"kind":"cell","row":0,"column":2,"said":"sí"}
| a | b | c |
| --- | --- | --- |
| 1 |
=>
| a | b | c |
| --- | --- | --- |
| 1 |  | sí |
@ {"kind":"rename","column":1,"said":"Costo"}
| Servicio | Precio {money CLP sum} |
| --- | --- |
| x | 1 |
=>
| Servicio | Costo {money CLP sum}  |
| --- | --- |
| x | 1 |
@ {"kind":"retype","column":1,"words":"money  USD"}
| Servicio | Precio |
| --- | --- |
| x | 1 |
=>
| Servicio | Precio {money USD} |
| --- | --- |
| x | 1 |
@ {"kind":"retype","column":0,"words":""}
| Precio {money} |
| --- |
| 1 |
=>
| Precio         |
| --- |
| 1 |
@ {"kind":"retype","column":0,"words":"plata"}
| Precio |
| --- |
| 1 |
=>
unknownType
@ {"kind":"rowAdd","at":1}
> | a | b |
> | --- | --- |
> | 1 | 2 |
=>
> | a | b |
> | --- | --- |
> | 1 | 2 |
> |  |  |
@ {"kind":"rowAdd","at":0}
| a |
| --- |
=>
| a |
| --- |
|  |
@ {"kind":"rowRemove","row":0}
| a |
| --- |
| 1 |
| 2 |
=>
| a |
| --- |
| 2 |
@ {"kind":"rowCopy","row":0}
| a | b |
| --- | --- |
| 1 | x \| y |
| 2 | z |
=>
| a | b |
| --- | --- |
| 1 | x \| y |
| 1 | x \| y |
| 2 | z |
@ {"kind":"rowMove","from":2,"to":0}
| a |
| --- |
| 1 |
| 2 |
| 3 |
=>
| a |
| --- |
| 3 |
| 1 |
| 2 |
@ {"kind":"columnAdd","at":1}
| a | b |
| --- | :-: |
| 1 | 2 |
=>
| a |  | b |
| --- | --- | :-: |
| 1 |  | 2 |
@ {"kind":"columnRemove","column":0}
| a | b {number} |
| ------ | --: |
| 1 | 2 |
=>
| b {number} |
| --: |
| 2 |
@ {"kind":"columnRemove","column":0}
| a |
| --- |
| 1 |
=>
lastColumn
@ {"kind":"columnMove","from":0,"to":1}
| a | b {date} |
| :---------- | --- |
| 1 | 2026-11-14 |
=>
| b {date} | a |
| --- | :---------- |
| 2026-11-14 | 1 |
@ {"kind":"sort","column":1,"down":false}
| Servicio | Precio {money CLP} |
| --- | --: |
| Netflix | 9990 |
| Nada |  |
| Raro | pronto |
| Spotify | 6490 |
| Gratis | 0 |
=>
| Servicio | Precio {money CLP} |
| --- | --: |
| Gratis | 0 |
| Spotify | 6490 |
| Netflix | 9990 |
| Raro | pronto |
| Nada |  |
@ {"kind":"sort","column":1,"down":true}
| Servicio | Precio {money CLP} |
| --- | --: |
| Netflix | 9990 |
| Nada |  |
| Spotify | 6490 |
=>
| Servicio | Precio {money CLP} |
| --- | --: |
| Netflix | 9990 |
| Spotify | 6490 |
| Nada |  |
@ {"kind":"sort","column":0,"down":false}
| Ciudad |
| --- |
| Zapallar |
| **Ávila** |
| arica |
=>
| Ciudad |
| --- |
| arica |
| **Ávila** |
| Zapallar |
@ {"kind":"sort","column":0,"down":false}
| Cuándo {date} |
| --- |
| 2026-11-14 |
| 2026-03-01 |
| 2025-12-31 |
=>
| Cuándo {date} |
| --- |
| 2025-12-31 |
| 2026-03-01 |
| 2026-11-14 |
@ {"kind":"cell","row":5,"column":0,"said":"x"}
| a |
| --- |
| 1 |
=>
beyond
@ {"kind":"columnMove","from":0,"to":3}
| a | b |
| --- | --- |
=>
beyond
"#;

#[test]
fn each_edit_changes_the_table_as_asked_and_nothing_else() {
    for case in EDITS.split("\n@ ").filter(|one| !one.trim().is_empty()) {
        let case = case.trim_start_matches("@ ");
        let (asked, rest) = case.split_once('\n').expect(case);
        let (before, after) = rest.split_once("=>\n").expect(case);
        let asked: Edit = serde_json::from_str(asked).expect(asked);
        let text = format!("Antes.\n\n{before}\nDespués.\n");
        let expected = match after.trim_end() {
            "moved" | "beyond" | "unknownType" | "lastColumn" => Err(after.trim_end().to_string()),
            table => Ok(format!("Antes.\n\n{table}\n\nDespués.\n")),
        };
        let got = edited(&text, &asked).map_err(|why| why.word().to_string());
        assert_eq!(got, expected, "{case}");
    }
}

#[test]
fn a_table_that_moved_since_it_was_drawn_is_left_alone() {
    let text = "Antes.\n\n| a |\n| --- |\n| 1 |\n";
    let at = table_in(read(text).blocks()).expect("a table");
    let asked = Edit::RowRemove { row: 0 };
    assert_eq!(
        edit(text, at.start + 1..at.end, &asked),
        Err(Refused::Moved)
    );
    assert_eq!(edit(text, 0..6, &asked), Err(Refused::Moved));
    assert_eq!(
        edit(text, at.start..at.end + 50, &asked),
        Err(Refused::Moved)
    );
    assert!(edit(text, at, &asked).is_ok());
}

#[test]
fn an_edit_travels_as_the_window_writes_it() {
    let asked = Edit::Retype {
        column: 2,
        words: "money CLP sum".to_string(),
    };
    let said = serde_json::to_string(&asked).expect("written");
    assert_eq!(
        said,
        r#"{"kind":"retype","column":2,"words":"money CLP sum"}"#
    );
    assert_eq!(Refused::LastColumn.to_string(), "lastColumn");
}
