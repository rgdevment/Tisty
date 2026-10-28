use serde_json::{Value, json};
use tisty_core::Paths;
use tisty_markdown::grid::{self, Edit, Refused as GridRefused, Shown};

use super::editing::writable;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::spoken::{heard, said_in_tisty};
use super::super::{Refused, UNSETTLED, by_how_much, grew, hitch, left_loose, loose_words, retold};

// Enough rows to find one by its first cell, without handing a long table over whole.
const ROWS_SHOWN: usize = 50;

/// One table of a document changed by what its columns and rows are called, never by its Markdown.
pub(in crate::mcp) fn edit_table(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool("`edit_table` needs the `doc` name.".into()));
    };
    let body = tisty_core::docs::read(&paths.docs(), &which).map_err(|_| {
        Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        ))
    })?;
    let tables = grid::shown(&body);
    let (nth, table) = chosen(&tables, args, &which)?;
    let asked = args["do"].as_str().unwrap_or("show");
    if asked == "show" {
        return Ok(told(described(nth, table), shape(&which, nth, table, &[])));
    }
    let mut changed = Vec::new();
    let (edits, touched) = edits_for(asked, table, args, &mut changed)?;
    let (state, mut store, named) = writable(paths, args, &which)?;
    let next = grid::edits(&body, table.at.clone(), &edits).map_err(refused)?;
    super::writing::still_speaks(&body, &next)?;
    let print = tisty_core::attach::printed(body.as_bytes());
    let made = tisty_core::docs::rewrite(&paths.docs(), paths.data(), &which, &next, &print)
        .map_err(hitch)?;
    let tisty_core::docs::Rewrite::Made { was, whole } = made else {
        return Err(Refused::Tool(format!(
            "{named} was written by someone else in the same moment, so the table was left as \
             it was. Ask again."
        )));
    };
    let loose = left_loose(&state, &which, &was, &whole);
    let settled = retold(&state, &mut store, &which, &whole).is_ok();
    let now = grid::shown(&whole);
    let table = now.get(nth).unwrap_or(table);
    let amiss = amiss_in(&whole, table, &touched);
    let mut shaped = shape(&which, nth, table, &amiss);
    shaped["grew"] = json!(grew(&was, &whole));
    shaped["loose"] = json!(loose);
    shaped["print"] = json!(tisty_core::attach::printed(whole.as_bytes()));
    let reply = told(
        format!(
            "Changed the table {} in {:?}, {}. What it was is kept beside the documents.{}{}{}",
            nth + 1,
            tisty_core::docs::titled(&whole),
            by_how_much(&was, &whole),
            warned(&amiss),
            loose_words(&state, &loose),
            if settled { "" } else { UNSETTLED },
        ),
        shaped,
    );
    Ok(heard(reply, &changed))
}

// Capitals, accents and the spaces at either end decide nothing, as with steps and tags.
fn same(one: &str, other: &str) -> bool {
    tisty_core::text::folded(one.trim()) == tisty_core::text::folded(other.trim())
}

// Assistants often send a number as words: "2" names the second when nothing is called that.
fn placed(said: &str) -> Option<usize> {
    said.trim().parse::<usize>().ok()?.checked_sub(1)
}

// What an assistant sends for a cell comes in as any Markdown does: turned into the Tisty language.
fn spoken(said: &str, changed: &mut Vec<&'static str>) -> String {
    said_in_tisty(said, changed).trim().to_string()
}

// A document with one table needs no `table`; with more, it is named by its place or by the heading over it.
fn chosen<'a>(
    tables: &'a [Shown],
    args: &Value,
    which: &str,
) -> Result<(usize, &'a Shown), Refused> {
    let listed = || {
        tables
            .iter()
            .enumerate()
            .map(|(at, one)| {
                format!(
                    "{} under {:?}: {}",
                    at + 1,
                    one.heading.as_deref().unwrap_or("no heading"),
                    titles(one)
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    let found = match &args["table"] {
        Value::Null if tables.len() == 1 => Some(0),
        Value::Number(at) => at.as_u64().and_then(|at| (at as usize).checked_sub(1)),
        Value::String(said) => {
            let under: Vec<usize> = (0..tables.len())
                .filter(|at| {
                    tables[*at]
                        .heading
                        .as_deref()
                        .is_some_and(|one| same(one, said))
                })
                .collect();
            match under.as_slice() {
                [] => placed(said),
                [one] => Some(*one),
                many => {
                    return Err(Refused::Tool(format!(
                        "{} tables sit under {said:?}; name the one you mean by its number. \
                         {which:?} holds: {}.",
                        many.len(),
                        listed()
                    )));
                }
            }
        }
        _ => None,
    };
    match found.and_then(|at| tables.get(at).map(|one| (at, one))) {
        Some(one) => Ok(one),
        None if tables.is_empty() => Err(Refused::Tool(format!(
            "{which:?} holds no table. Write one with `edit_doc`; `language` with the topic \
             `tables` says how."
        ))),
        None => Err(Refused::Tool(format!(
            "name the table with `table`, by its number or the heading over it. {which:?} holds: {}.",
            listed()
        ))),
    }
}

fn titles(table: &Shown) -> String {
    table
        .columns
        .iter()
        .map(|one| match one.kind.as_str() {
            "" => one.title.clone(),
            kind => format!("{} {{{kind}}}", one.title),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn described(nth: usize, table: &Shown) -> String {
    let rows: Vec<&str> = table
        .rows
        .iter()
        .take(ROWS_SHOWN)
        .map(String::as_str)
        .collect();
    let more = table.rows.len().saturating_sub(ROWS_SHOWN);
    format!(
        "Table {} {}has the columns {} and {} rows, named by their first cell: {}{}.",
        nth + 1,
        table
            .heading
            .as_deref()
            .map(|one| format!("under {one:?} "))
            .unwrap_or_default(),
        titles(table),
        table.rows.len(),
        rows.join(", "),
        if more > 0 {
            format!(" and {more} more")
        } else {
            String::new()
        }
    )
}

fn shape(which: &str, nth: usize, table: &Shown, amiss: &[Amiss]) -> Value {
    json!({
        "doc": which,
        "table": nth + 1,
        "heading": table.heading,
        "columns": table.columns,
        "rows": table.rows.len(),
        "first_cells": table.rows.iter().take(ROWS_SHOWN).collect::<Vec<_>>(),
        "amiss": amiss.iter().map(|one| json!({
            "row": one.row,
            "column": one.column,
            "value": one.value,
            "type": one.kind,
        })).collect::<Vec<_>>(),
    })
}

/// Which rows an edit wrote into, so only what was just written is checked against its type.
enum Touched {
    Rows(Vec<usize>),
    Column(usize),
    Nothing,
}

fn edits_for(
    asked: &str,
    table: &Shown,
    args: &Value,
    changed: &mut Vec<&'static str>,
) -> Result<(Vec<Edit>, Touched), Refused> {
    let column = || column_of(table, &args["column"]);
    let row = || row_of(table, &args["row"]);
    Ok(match asked {
        "set" => {
            let (row, column) = (row()?, column()?);
            let value = spoken(said(args, "value")?, changed);
            let value = tisty_markdown::canon::canonical(&table.columns[column].kind, &value);
            (
                vec![Edit::Cell {
                    row,
                    column,
                    said: value,
                }],
                Touched::Rows(vec![row]),
            )
        }
        "add" => {
            let row = table.rows.len();
            let mut edits = vec![Edit::RowAdd { at: row }];
            let Some(values) = args["values"].as_object() else {
                return Err(Refused::Tool(
                    "`add` needs `values`: each column's title with what goes in it.".into(),
                ));
            };
            for (title, value) in values {
                let column = column_of(table, &Value::String(title.clone()))?;
                let value = match value.as_str() {
                    Some(said) => spoken(said, changed),
                    None => value.to_string(),
                };
                let value = tisty_markdown::canon::canonical(&table.columns[column].kind, &value);
                edits.push(Edit::Cell {
                    row,
                    column,
                    said: value,
                });
            }
            (edits, Touched::Rows(vec![row]))
        }
        "remove" => (vec![Edit::RowRemove { row: row()? }], Touched::Nothing),
        "sort" => {
            let down = args["down"].as_bool().unwrap_or(false);
            (
                vec![Edit::Sort {
                    column: column()?,
                    down,
                }],
                Touched::Nothing,
            )
        }
        "type" => {
            let column = column()?;
            let words = said(args, "type")?.to_string();
            (
                vec![Edit::Retype { column, words }],
                Touched::Column(column),
            )
        }
        "rename" => {
            let said = spoken(said(args, "title")?, changed);
            (
                vec![Edit::Rename {
                    column: column()?,
                    said,
                }],
                Touched::Nothing,
            )
        }
        "add_column" => {
            let at = table.columns.len();
            let said = spoken(said(args, "title")?, changed);
            let mut edits = vec![Edit::ColumnAdd { at }, Edit::Rename { column: at, said }];
            if let Some(words) = args["type"].as_str().filter(|one| !one.trim().is_empty()) {
                edits.push(Edit::Retype {
                    column: at,
                    words: words.to_string(),
                });
            }
            (edits, Touched::Column(at))
        }
        "remove_column" => (
            vec![Edit::ColumnRemove { column: column()? }],
            Touched::Nothing,
        ),
        other => {
            return Err(Refused::Tool(format!(
                "`do` is one of show, set, add, remove, sort, type, rename, add_column or \
                 remove_column, not {other:?}."
            )));
        }
    })
}

fn said<'a>(args: &'a Value, key: &str) -> Result<&'a str, Refused> {
    args[key]
        .as_str()
        .ok_or_else(|| Refused::Tool(format!("this change needs `{key}`.")))
}

// A column by its title, as its head reads without its type, or by its number from 1.
fn column_of(table: &Shown, asked: &Value) -> Result<usize, Refused> {
    let found = match asked {
        Value::Number(at) => at.as_u64().and_then(|at| (at as usize).checked_sub(1)),
        Value::String(said) => table
            .columns
            .iter()
            .position(|one| same(&one.title, said))
            .or_else(|| placed(said)),
        _ => None,
    };
    found.filter(|at| *at < table.columns.len()).ok_or_else(|| {
        Refused::Tool(format!(
            "name the `column` by its title or its number from 1. This table's columns are: {}.",
            titles(table)
        ))
    })
}

// A row by the words of its first cell, the way a person names it, or by its number from 1.
fn row_of(table: &Shown, asked: &Value) -> Result<usize, Refused> {
    let numbered = |at: Option<usize>| {
        at.filter(|at| *at < table.rows.len())
            .ok_or_else(|| Refused::Tool(format!("this table has {} rows.", table.rows.len())))
    };
    if let Some(at) = asked.as_u64() {
        return numbered((at as usize).checked_sub(1));
    }
    let Some(said) = asked.as_str().map(str::trim) else {
        return Err(Refused::Tool(
            "name the `row` by the words of its first cell, or by its number from 1.".into(),
        ));
    };
    let found: Vec<usize> = (0..table.rows.len())
        .filter(|at| same(&table.rows[*at], said))
        .collect();
    match found.as_slice() {
        [one] => Ok(*one),
        [] if said.parse::<usize>().is_ok() => numbered(placed(said)),
        [] => Err(Refused::Tool(format!(
            "no row starts with {said:?}. Its first cells are: {}.",
            table
                .rows
                .iter()
                .take(ROWS_SHOWN)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ))),
        many => Err(Refused::Tool(format!(
            "{} rows start with {said:?}, at {}; name the one you mean by its number.",
            many.len(),
            many.iter()
                .map(|at| (at + 1).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

struct Amiss {
    row: String,
    column: String,
    value: String,
    kind: String,
}

// Only what this edit wrote is told: a value already amiss elsewhere is the person's, not news.
fn amiss_in(whole: &str, table: &Shown, touched: &Touched) -> Vec<Amiss> {
    let (Ok(found), Ok(written)) = (
        grid::amiss(whole, table.at.clone()),
        grid::written(whole, table.at.clone()),
    ) else {
        return Vec::new();
    };
    found
        .into_iter()
        .filter(|(row, column)| match touched {
            Touched::Rows(rows) => rows.contains(row),
            Touched::Column(at) => at == column,
            Touched::Nothing => false,
        })
        .map(|(row, column)| Amiss {
            row: table.rows.get(row).cloned().unwrap_or_default(),
            column: table.columns[column].title.clone(),
            value: written.rows[row].get(column).cloned().unwrap_or_default(),
            kind: table.columns[column].kind.clone(),
        })
        .collect()
}

fn warned(amiss: &[Amiss]) -> String {
    if amiss.is_empty() {
        return String::new();
    }
    let said: Vec<String> = amiss
        .iter()
        .map(|one| format!("{:?} in {} of {:?}", one.value, one.column, one.row))
        .collect();
    format!(
        " Not written as the column's type asks, so kept as written and shown marked: {}. \
         `language` with the topic `tables` says how each type is written.",
        said.join("; ")
    )
}

fn refused(why: GridRefused) -> Refused {
    Refused::Tool(match why {
        GridRefused::Moved => "the table changed while this edit was on its way; ask again.".into(),
        GridRefused::Beyond => "that row or column is past the table's end.".into(),
        GridRefused::UnknownType => "that `type` is not one Tisty knows; `language` with the \
                                      topic `tables` lists them."
            .into(),
        GridRefused::LastColumn => "a table keeps at least one column.".into(),
    })
}
