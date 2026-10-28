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
        assert_eq!(Lines::read(said, false).written(), said, "{table}");
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
        // The same edit on a file written with Windows line ends keeps them, whichever row ends up last.
        let crlf = |one: &str| one.replace('\n', "\r\n");
        let got = edited(&crlf(&text), &asked).map_err(|why| why.word().to_string());
        assert_eq!(got, expected.map(|one| crlf(&one)), "CRLF {case}");
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

#[test]
fn the_page_is_told_the_order_a_column_sorts_its_rows_in() {
    let text = "| Servicio | Precio {money CLP} |\n| --- | --: |\n| Netflix | 9990 |\n| Nada |  |\n| Spotify | 6490 |\n";
    let at = table_in(read(text).blocks()).expect("a table");
    assert_eq!(order(text, at.clone(), 1, false), Ok(vec![2, 0, 1]));
    assert_eq!(order(text, at.clone(), 1, true), Ok(vec![0, 2, 1]));
    assert_eq!(order(text, at.clone(), 0, false), Ok(vec![1, 0, 2]));
    assert_eq!(order(text, at, 2, false), Err(Refused::Beyond));
    assert_eq!(order(text, 0..3, 0, false), Err(Refused::Moved));
}

#[test]
fn edits_in_a_row_land_on_the_table_as_each_one_left_it() {
    let text = "Antes.\n\n| a |\n| --- |\n| 1 |\n| 2 |\n\nDespués.\n";
    let at = table_in(read(text).blocks()).expect("a table");
    let asked = [
        Edit::Cell {
            row: 0,
            column: 0,
            said: "uno largo".to_string(),
        },
        Edit::RowMove { from: 0, to: 1 },
        Edit::RowAdd { at: 2 },
    ];
    assert_eq!(
        edits(text, at.clone(), &asked),
        Ok("Antes.\n\n| a |\n| --- |\n| 2 |\n| uno largo |\n|  |\n\nDespués.\n".to_string())
    );
    assert_eq!(
        edits(text, at, &[Edit::RowRemove { row: 5 }]),
        Err(Refused::Beyond)
    );
}

#[test]
fn a_cell_opens_for_writing_as_the_file_has_it() {
    let text = "| Servicio | Precio {money CLP sum} |\n| --- | --: |\n| Netflix \\| 4K | **9990** |\n| Solo |\n";
    let at = table_in(read(text).blocks()).expect("a table");
    let said = written(text, at).expect("written");
    assert_eq!(said.heads, vec!["Servicio", "Precio"]);
    assert_eq!(
        said.rows,
        vec![vec![r"Netflix \| 4K", "**9990**"], vec!["Solo"]]
    );
    assert_eq!(written(text, 0..3), Err(Refused::Moved));
}

#[test]
fn a_byte_names_the_table_that_holds_it_even_inside_a_quote_or_a_list() {
    let text = "Antes.\n\n| a |\n| --- |\n| 1 |\n\n> | b |\n> | --- |\n> | 2 |\n\nDespués.\n";
    let first = table_in(read(text).blocks()).expect("a table");
    assert_eq!(around(text, first.start), Some(first.clone()));
    assert_eq!(around(text, first.start + 9), Some(first.clone()));
    assert_eq!(around(text, 2), None);
    let quoted = text.find("> | b").expect("the quote");
    let inner = around(text, quoted + 4).expect("the quoted table");
    assert!(text[inner.clone()].contains("| b |"));
    assert!(edit(text, inner, &Edit::RowAdd { at: 1 }).is_ok());
    assert_eq!(around(text, text.len() - 2), None);
}

// A column's head, the type word asked for, then the head after it.
const RECASTS: &str = "
Precio {money CLP sum} | money   => Precio {money CLP sum}
Precio {money CLP sum} | number  => Precio {number sum}
Precio {money CLP sum} | date    => Precio {date}
Fecha {date max}       | number  => Fecha {number max}
Fecha {date max}       | text    => Fecha
Notas                  | rating  => Notas {rating}
Notas                  | plata   => unknownType
";

#[test]
fn a_new_type_keeps_the_currency_and_total_that_still_apply() {
    for line in RECASTS.lines().filter(|one| !one.trim().is_empty()) {
        let (asked, expected) = line.split_once("=>").expect(line);
        let (head, word) = asked.split_once('|').expect(line);
        let text = format!("| {} |\n| --- |\n| 1 |\n", head.trim());
        let byte = text.find('|').expect("a pipe") + 2;
        let made = edits_near(
            &text,
            byte,
            &[Edit::Recast {
                column: 0,
                word: word.trim().to_string(),
            }],
        );
        let said = match made {
            Ok(one) => one
                .lines()
                .next()
                .unwrap_or_default()
                .trim_matches(['|', ' '])
                .to_string(),
            Err(why) => why.word().to_string(),
        };
        assert_eq!(said, expected.trim(), "{line}");
    }
}

#[test]
fn edits_near_a_byte_find_their_table_inside_a_quote_and_nothing_outside_it() {
    let text = "Antes.\n\n> | b |\n> | --- |\n> | 2 |\n";
    let byte = text.find("> | 2").expect("the row") + 4;
    let made = edits_near(text, byte, &[Edit::RowAdd { at: 1 }]).expect("edited");
    assert!(made.ends_with("> | 2 |\n> |  |\n"), "{made}");
    assert_eq!(
        edits_near(text, 1, &[Edit::RowAdd { at: 1 }]),
        Err(Refused::Moved)
    );
}
