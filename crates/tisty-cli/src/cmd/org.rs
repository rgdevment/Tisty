use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::{
    ListId, Op, Tag,
    event::{ListAdd, Name, TaskPatch},
    inverse,
};
use ulid::Ulid;

use super::confirm;
use crate::app::App;
use crate::i18n::Lang;
use crate::style::{self, GREEN};
use crate::{EXIT_NOT_FOUND, ListAction, TagAction, render};
use serde::Serialize;

pub fn lists(app: &App, json: bool, lang: Lang) -> anyhow::Result<ExitCode> {
    if json {
        println!("{}", serde_json::to_string(&app.state.ordered_lists())?);
    } else {
        print!("{}", render::lists(&app.state, lang));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn list(app: &mut App, action: Option<ListAction>, lang: Lang) -> anyhow::Result<ExitCode> {
    match action.unwrap_or(ListAction::Ls { json: false }) {
        ListAction::Ls { json } => lists(app, json, lang),

        ListAction::Add { name } => {
            let name = name.join(" ").trim().to_string();
            if name.is_empty() {
                anyhow::bail!("{}", lang.get("needs-name"));
            }

            taken(app, &name, None, lang)?;

            let id = Ulid::generate();
            app.commit(Op::ListAdd {
                id,
                d: ListAdd {
                    name: name.clone(),
                    order: app.state.next_list_order(),
                    color: None,
                },
            })?;
            println!("  {} {name}", style::paint(GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        ListAction::Rename { selector, name } => {
            let name = name.join(" ").trim().to_string();
            if name.is_empty() {
                anyhow::bail!("{}", lang.get("needs-name"));
            }
            let Some(id) = one_list(app, &selector, lang)? else {
                return Ok(ExitCode::from(EXIT_NOT_FOUND));
            };
            taken(app, &name, Some(id), lang)?;

            app.commit(Op::ListRename {
                id,
                d: Name { name: name.clone() },
            })?;
            println!("  {} {name}", style::paint(GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        ListAction::Archive { selector } => {
            let Some(id) = one_list(app, &selector, lang)? else {
                return Ok(ExitCode::from(EXIT_NOT_FOUND));
            };

            let name = app.state.lists[&id].name.clone();
            app.commit(Op::ListArchive { id })?;
            println!("  {} {}", style::dim("✕"), style::dim(&name));
            Ok(ExitCode::SUCCESS)
        }

        ListAction::Unarchive { selector } => {
            let Some(id) = one_list(app, &selector, lang)? else {
                return Ok(ExitCode::from(EXIT_NOT_FOUND));
            };

            let name = app.state.lists[&id].name.clone();
            app.commit(Op::ListUnarchive { id })?;
            println!("  {} {name}", style::paint(GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        ListAction::Rm { selector, force } => {
            let Some(id) = one_list(app, &selector, lang)? else {
                return Ok(ExitCode::from(EXIT_NOT_FOUND));
            };

            let name = app.state.lists[&id].name.clone();
            let orphans = app
                .state
                .tasks
                .values()
                .filter(|t| t.list == Some(id))
                .count();
            let question = lang.fill(
                "confirm-rm-list",
                &[("name", &name), ("n", &orphans.to_string())],
            );
            if !confirm(&question, force, lang)? {
                return Ok(ExitCode::SUCCESS);
            }

            app.commit(Op::ListDelete { id })?;
            println!("  {} {}", style::dim("✕"), style::dim(&name));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn taken(app: &App, name: &str, except: Option<ListId>, lang: Lang) -> anyhow::Result<()> {
    if app
        .state
        .find_list(name)
        .iter()
        .any(|l| Some(l.id) != except && l.name.eq_ignore_ascii_case(name))
    {
        anyhow::bail!("{}", lang.fill("list-exists", &[("name", name)]));
    }
    Ok(())
}

fn one_list(app: &App, selector: &str, lang: Lang) -> anyhow::Result<Option<ListId>> {
    match app.state.find_list(selector).as_slice() {
        [one] => Ok(Some(one.id)),
        [] => {
            eprintln!("{}", lang.fill("no-such-list", &[("selector", selector)]));
            Ok(None)
        }
        _ => anyhow::bail!("{}", lang.fill("ambiguous-list", &[("selector", selector)])),
    }
}

#[derive(Serialize)]
struct Counted {
    tag: String,
    tasks: usize,
}

pub fn tag(app: &mut App, action: Option<TagAction>, lang: Lang) -> anyhow::Result<ExitCode> {
    match action.unwrap_or(TagAction::Ls { json: false }) {
        TagAction::Ls { json } => {
            let counted: Vec<Counted> = app
                .state
                .tags()
                .into_iter()
                .map(|t| Counted {
                    tag: t.to_string(),
                    tasks: app.state.tasks_tagged(t).count(),
                })
                .collect();

            if json {
                println!("{}", serde_json::to_string(&counted)?);
            } else if counted.is_empty() {
                println!("\n  {}\n", style::dim(lang.get("no-tags-yet")));
            } else {
                println!("\n  {}\n", style::bold(lang.get("tags")));
                for c in counted {
                    println!(
                        "    {:<32}{}",
                        format!("#{}", c.tag),
                        style::dim(&c.tasks.to_string())
                    );
                }
                println!();
            }
            Ok(ExitCode::SUCCESS)
        }

        TagAction::Rename { old, new } => {
            let (old, new) = (parse_tag(&old)?, parse_tag(&new)?);
            let ops = retag(app, &old, Some(new.clone()));
            if ops.is_empty() {
                return Ok(missing_tag(&old, lang));
            }

            let n = app.commit_all(ops)?;
            println!(
                "  {} #{old} → #{new} {}",
                style::paint(GREEN, "✓"),
                style::dim(&lang.plural("tasks", n))
            );
            Ok(ExitCode::SUCCESS)
        }

        TagAction::Rm { tag, force } => {
            let tag = parse_tag(&tag)?;
            let ops = retag(app, &tag, None);
            if ops.is_empty() {
                return Ok(missing_tag(&tag, lang));
            }

            let question = lang.fill(
                "confirm-rm-tag",
                &[("tag", tag.as_str()), ("n", &ops.len().to_string())],
            );
            if !confirm(&question, force, lang)? {
                return Ok(ExitCode::SUCCESS);
            }

            let n = app.commit_all(ops)?;
            println!(
                "  {} #{tag} {}",
                style::dim("✕"),
                style::dim(&lang.plural("tasks", n))
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn retag(app: &App, from: &Tag, to: Option<Tag>) -> Vec<Op> {
    app.state
        .tasks_tagged(from)
        .map(|task| {
            let mut tags: Vec<Tag> = task.tags.iter().filter(|t| *t != from).cloned().collect();
            if let Some(to) = &to
                && !tags.contains(to)
            {
                tags.push(to.clone());
            }
            Op::TaskUpdate {
                id: task.id,
                d: TaskPatch {
                    tags: Some(tags),
                    ..Default::default()
                },
            }
        })
        .collect()
}

fn parse_tag(raw: &str) -> anyhow::Result<Tag> {
    Ok(Tag::new(raw.trim_start_matches('@'))?)
}

fn missing_tag(tag: &Tag, lang: Lang) -> ExitCode {
    eprintln!("{}", lang.fill("no-such-tag", &[("tag", tag.as_str())]));
    ExitCode::from(EXIT_NOT_FOUND)
}

pub fn undo(app: &mut App, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    step_history(app, false, today, lang)
}

pub fn redo(app: &mut App, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    step_history(app, true, today, lang)
}

fn step_history(app: &mut App, redoing: bool, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let (entity, ops) = if redoing {
        let change = app.last_undone_change()?;
        if change.is_empty() {
            println!("  {}", style::dim(lang.get("nothing-to-redo")));
            return Ok(ExitCode::SUCCESS);
        }
        let ops = app
            .state
            .afresh(change.into_iter().map(|e| e.op).collect::<Vec<_>>());
        let entity = ops.first().and_then(|op| op.about_whom());
        if entity.is_some_and(|one| app.state.is_erased(one)) {
            anyhow::bail!("{}", lang.get("cannot-redo"));
        }
        (entity, ops)
    } else {
        let change = app.last_own_change()?;
        if change.is_empty() {
            println!("  {}", style::dim(lang.get("nothing-to-undo")));
            return Ok(ExitCode::SUCCESS);
        }

        let mut ops = Vec::with_capacity(change.len());
        for (event, before) in change.iter().rev() {
            match inverse(event, before) {
                Some(op) => ops.push(op),
                None => anyhow::bail!("{}", lang.get("cannot-undo")),
            }
        }
        (change[0].0.entity_id(), ops)
    };

    let n = if redoing {
        app.commit_redo(ops)?
    } else {
        app.commit_undo(ops)?
    };
    let done = if redoing { "redone" } else { "undone" };

    match entity.and_then(|one| app.state.tasks.get(&one)) {
        Some(task) if n == 1 => print!("{}", render::line(task, &app.state, today, lang)),
        _ if n == 1 => println!("  {}", style::dim(lang.get(done))),
        _ => println!(
            "  {} {}",
            style::dim(lang.get(done)),
            style::dim(&lang.plural("changes", n))
        ),
    }
    Ok(ExitCode::SUCCESS)
}
