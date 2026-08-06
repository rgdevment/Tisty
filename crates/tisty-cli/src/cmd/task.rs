use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::{
    Op, Tag, TaskId,
    event::{Body, LogAdd, StepAdd, StepRef, StepText, TaskAdd, TaskMove, TaskPatch},
};
use ulid::Ulid;

use super::{confirm, resolved, text_or_stdin};
use crate::app::App;
use crate::i18n::Lang;
use crate::style;
use crate::{AddArgs, SetArgs, StepAction, date_flag, render};

pub fn add(app: &mut App, args: AddArgs, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let text = args.text.join(" ").trim().to_string();
    if text.is_empty() {
        anyhow::bail!("{}", lang.get("needs-title"));
    }

    let now = jiff::Zoned::now();
    let parsed = tisty_nl::parse(&text, &now, lang.code());

    let list = match &args.list {
        Some(name) => match app.list_id(name) {
            Some(id) => Some(id),
            None => anyhow::bail!("{}", lang.fill("no-such-list", &[("selector", name)])),
        },
        // `#casa` in the text creates the list; the flag still demands one that exists.
        None => match parsed.list.as_deref() {
            Some(name) => Some(match app.list_id(name) {
                Some(id) => id,
                None => {
                    let id = Ulid::generate();
                    app.commit(Op::ListAdd {
                        id,
                        d: tisty_core::event::ListAdd {
                            name: name.to_string(),
                            order: app.next_list_order(),
                            color: None,
                        },
                    })?;
                    id
                }
            }),
            None => None,
        },
    };

    let id = Ulid::generate();
    let d = TaskAdd {
        date: date_flag(args.date.as_deref(), lang)?.or(parsed.date),
        deadline: date_flag(args.deadline.as_deref(), lang)?.or(parsed.deadline),
        priority: args
            .priority
            .map(tisty_core::Priority::try_from)
            .transpose()?
            .or(parsed.priority),
        tags: parsed.tags,
        list,
        ..TaskAdd::new(parsed.title, app.next_task_order())
    };

    app.commit(Op::TaskAdd { id, d })?;
    let task = &app.state.tasks[&id];

    if args.json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::captured(task, &app.state, today, lang));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn done(
    app: &mut App,
    selector: Option<&str>,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();
    if open.is_empty() {
        println!("  {}", style::dim(lang.get("nothing-open")));
        return Ok(ExitCode::SUCCESS);
    }

    resolved!(app, selector, open, lang, |id| {
        app.commit(Op::TaskDone { id })?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

pub fn undone(app: &mut App, selector: &str, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let archived: Vec<_> = app.state.archived_tasks().collect();
    resolved!(app, Some(selector), archived, lang, |id| {
        app.commit(Op::TaskReopen { id })?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

pub fn drop(app: &mut App, selector: &str, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();
    resolved!(app, Some(selector), open, lang, |id| {
        app.commit(Op::TaskDrop { id })?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

pub fn rm(app: &mut App, selector: &str, force: bool, lang: Lang) -> anyhow::Result<ExitCode> {
    let all: Vec<_> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), all, lang, |id| {
        let title = app.state.tasks[&id].title.clone();
        if !confirm(&lang.fill("confirm-rm", &[("title", &title)]), force, lang)? {
            return Ok(ExitCode::SUCCESS);
        }

        app.commit(Op::TaskDelete { id })?;
        println!("  {} {}", style::dim("✕"), style::dim(&title));
        Ok(ExitCode::SUCCESS)
    })
}

pub fn set(app: &mut App, args: SetArgs, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let date = date_flag(args.date.as_deref(), lang)?;
    let deadline = date_flag(args.deadline.as_deref(), lang)?;
    let all: Vec<_> = app.state.tasks.values().collect();

    resolved!(app, Some(&args.selector), all, lang, |id| {
        let tags = merged_tags(app, id, &args.tag, &args.untag)?;

        let d = TaskPatch {
            title: args.title.clone(),
            date: date.map(Some).or(args.no_date.then_some(None)),
            deadline: deadline.map(Some).or(args.no_deadline.then_some(None)),
            priority: args
                .priority
                .map(tisty_core::Priority::try_from)
                .transpose()?,
            tags,
            reminders: None,
        };

        if d == TaskPatch::default() {
            anyhow::bail!("{}", lang.get("nothing-to-change"));
        }

        app.commit(Op::TaskUpdate { id, d })?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

fn merged_tags(
    app: &App,
    id: TaskId,
    add: &[String],
    remove: &[String],
) -> anyhow::Result<Option<Vec<Tag>>> {
    if add.is_empty() && remove.is_empty() {
        return Ok(None);
    }

    let mut tags = app.state.tasks[&id].tags.clone();
    for raw in add {
        let tag = Tag::new(raw.trim_start_matches('@'))?;
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    for raw in remove {
        let tag = Tag::new(raw.trim_start_matches('@'))?;
        tags.retain(|t| *t != tag);
    }
    Ok(Some(tags))
}

pub fn mv(
    app: &mut App,
    selector: &str,
    list: Option<&str>,
    inbox: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let target = match (list, inbox) {
        (Some(name), false) => match app.find_list(name).as_slice() {
            [one] => Some(one.id),
            [] => anyhow::bail!("{}", lang.fill("no-such-list", &[("selector", name)])),
            _ => anyhow::bail!("{}", lang.fill("ambiguous-list", &[("selector", name)])),
        },
        (None, true) => None,
        _ => anyhow::bail!("{}", lang.get("needs-list")),
    };

    let all: Vec<_> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), all, lang, |id| {
        app.commit(Op::TaskMove {
            id,
            d: TaskMove {
                list: Some(target),
                order: None,
            },
        })?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

pub fn desc(
    app: &mut App,
    selector: &str,
    text: Vec<String>,
    clear: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let body = if clear {
        None
    } else {
        Some(text_or_stdin(text, lang)?)
    };

    let all: Vec<_> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), all, lang, |id| {
        app.commit(Op::TaskDescribe {
            id,
            d: Body { body },
        })?;
        print!(
            "{}",
            render::detail(&app.state.tasks[&id], &app.state, today, lang)
        );
        Ok(ExitCode::SUCCESS)
    })
}

pub fn log(
    app: &mut App,
    selector: &str,
    text: Vec<String>,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let body = text_or_stdin(text, lang)?;

    let all: Vec<_> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), all, lang, |id| {
        app.commit(Op::TaskLog {
            id,
            d: LogAdd {
                entry: Ulid::generate(),
                tz: jiff::Zoned::now()
                    .time_zone()
                    .iana_name()
                    .map(str::to_string),
                body,
            },
        })?;
        print!(
            "{}",
            render::detail(&app.state.tasks[&id], &app.state, today, lang)
        );
        Ok(ExitCode::SUCCESS)
    })
}

pub fn step(
    app: &mut App,
    selector: &str,
    action: StepAction,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let all: Vec<_> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), all, lang, |id| {
        let op = match action {
            StepAction::Add { text } => {
                let text = text_or_stdin(text, lang)?;
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step: Ulid::generate(),
                        text,
                        order: app.next_step_order(&app.state.tasks[&id]),
                    },
                }
            }
            StepAction::Done { number } => Op::StepDone {
                id,
                d: StepRef {
                    step: nth_step(app, id, number, lang)?,
                },
            },
            StepAction::Undone { number } => Op::StepUndone {
                id,
                d: StepRef {
                    step: nth_step(app, id, number, lang)?,
                },
            },
            StepAction::Rm { number } => Op::StepRemove {
                id,
                d: StepRef {
                    step: nth_step(app, id, number, lang)?,
                },
            },
            StepAction::Text { number, text } => Op::StepText {
                id,
                d: StepText {
                    step: nth_step(app, id, number, lang)?,
                    text: text_or_stdin(text, lang)?,
                },
            },
        };

        app.commit(op)?;
        print!(
            "{}",
            render::detail(&app.state.tasks[&id], &app.state, today, lang)
        );
        Ok(ExitCode::SUCCESS)
    })
}

/// Steps are addressed by the number `show` prints, not by their id.
fn nth_step(app: &App, id: TaskId, number: usize, lang: Lang) -> anyhow::Result<Ulid> {
    match number
        .checked_sub(1)
        .and_then(|i| app.state.tasks[&id].steps.get(i))
    {
        Some(step) => Ok(step.id),
        None => anyhow::bail!(
            "{}",
            lang.fill("no-such-step", &[("n", &number.to_string())])
        ),
    }
}

fn report(app: &App, id: TaskId, today: Date, lang: Lang) {
    print!(
        "{}",
        render::line(&app.state.tasks[&id], &app.state, today, lang)
    );
}
