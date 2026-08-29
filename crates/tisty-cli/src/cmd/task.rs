use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::{
    Op, Tag, TaskId,
    capture::{Draft, Filing, Rejected},
    event::{Body, LogAdd, StepAdd, StepRef, StepText, TaskMove, TaskPatch},
};
use ulid::Ulid;

use super::{confirm, resolved, text_or_stdin};
use crate::app::App;
use crate::i18n::Lang;
use crate::style;
use crate::{AddArgs, SetArgs, StepAction, date_flag, render};

pub fn add(app: &mut App, args: AddArgs, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let text = args.text.join(" ");
    let now = jiff::Zoned::now();
    let read = tisty_nl::parse(&text, &now, lang.code());
    let guessed = (args.date.is_none()
        && args.deadline.is_none()
        && read
            .spans
            .iter()
            .any(|span| span.certainty == tisty_nl::Certainty::Assumed))
    .then(|| as_written(&text, &read, lang));
    let mut draft = Draft::from(read);

    if let Some(date) = date_flag(args.date.as_deref(), lang)? {
        draft.date = Some(date);
    }
    if let Some(deadline) = date_flag(args.deadline.as_deref(), lang)? {
        draft.deadline = Some(deadline);
    }
    if let Some(priority) = &args.priority {
        draft.priority = Some(named_priority(priority, lang)?);
    }
    if let Some(name) = args.list {
        draft.filing = Some(Filing::Named(name));
    }

    let plan = tisty_core::capture::plan(&app.state, draft).map_err(|e| refused(e, lang))?;
    app.commit_all(plan.ops)?;

    let task = &app.state.tasks[&plan.task];
    warn_if_backwards(task.date.as_ref(), task.deadline.as_ref(), lang);

    if args.json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!(
            "{}",
            render::captured(task, &app.state, today, lang, guessed)
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn as_written(text: &str, read: &tisty_nl::Parsed, lang: Lang) -> String {
    let markers: Vec<_> = read
        .spans
        .iter()
        .copied()
        .filter(|span| !matches!(span.mark, tisty_nl::Mark::Date | tisty_nl::Mark::Deadline))
        .collect();
    tisty_nl::title_without(text, &markers, lang.code())
}

fn refused(e: Rejected, lang: Lang) -> anyhow::Error {
    let message = match &e {
        Rejected::Untitled => lang.get("needs-title").to_string(),
        Rejected::NoSuchList(name) => lang.fill("no-such-list", &[("selector", name)]),
        Rejected::ArchivedList(name) => lang.fill("archived-list-refuses", &[("name", name)]),
        Rejected::AmbiguousList(name) => lang.fill("ambiguous-list", &[("selector", name)]),
        Rejected::EndedAlready => lang.get("past-end").to_string(),
    };
    anyhow::anyhow!("{message}")
}

pub fn attach(
    app: &mut App,
    selector: &str,
    at: &std::path::Path,
    label: Option<String>,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let named = tisty_core::attach::called(at, label);
    let open: Vec<&tisty_core::Task> = app.state.tasks.values().collect();
    resolved!(app, Some(selector), open, lang, |id| {
        let kept = tisty_core::attach::keep(at, app.paths.data(), app.copies_up_to())
            .map_err(|e| weighed(e, &named, lang))?;
        let body = tisty_core::attach::journalled(&kept, &named, at, lang.get("attached-from"));
        app.commit(Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), body)
                .in_zone(jiff::tz::TimeZone::system().iana_name().map(str::to_string)),
        })?;
        report(app, id, today, lang);
    });
    Ok(ExitCode::SUCCESS)
}

fn weighed(e: tisty_core::Error, named: &str, lang: Lang) -> anyhow::Error {
    match e {
        tisty_core::Error::AttachmentTooBig { limit, .. } => anyhow::anyhow!(
            "{}",
            lang.fill(
                "attach-too-big",
                &[("limit", &format!("{}", limit / 1_000_000))]
            )
        ),
        _ => anyhow::anyhow!("{}", lang.fill("attach-unreadable", &[("name", named)])),
    }
}

pub fn done(
    app: &mut App,
    selector: Option<&str>,
    also: &[String],
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let open = app.state.ordered_open();
    if open.is_empty() && selector.is_none() {
        println!("  {}", style::dim(lang.get("nothing-open")));
        return Ok(ExitCode::SUCCESS);
    }

    let kept = also
        .iter()
        .map(|day| {
            day.parse::<Date>()
                .map_err(|_| anyhow::anyhow!(lang.fill("not-a-date", &[("value", day)])))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    resolved!(app, selector, open, lang, |id| {
        let owed = app.state.owed_since(id, today);
        let ops = if kept.is_empty() {
            app.state.completing(id, jiff::Zoned::now())
        } else {
            app.state.covering(id, jiff::Zoned::now(), &kept)
        };
        app.commit_all(ops)?;
        report(app, id, today, lang);
        if kept.is_empty()
            && !owed.is_empty()
            && let Some(task) = app.state.tasks.get(&id)
        {
            // The task is closed by now, so the hint has to name it by something that still resolves.
            let days = owed
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "  {}",
                style::dim(&lang.fill(
                    "owed-days",
                    &[
                        ("n", &owed.len().to_string()),
                        ("sel", &crate::render::short_id(task)),
                        ("day", &days),
                    ],
                ))
            );
        }
        Ok(ExitCode::SUCCESS)
    })
}

fn cadence_flag(
    said: Option<&str>,
    cleared: bool,
    lang: Lang,
) -> anyhow::Result<Option<Option<tisty_core::model::Repeat>>> {
    if cleared {
        return Ok(Some(None));
    }
    let Some(said) = said else {
        return Ok(None);
    };
    let read = tisty_nl::parse(&format!("· {said}"), &jiff::Zoned::now(), lang.code());
    match read.repeat {
        Some(over) if over.ended(jiff::Zoned::now().date()) => {
            anyhow::bail!("{}", lang.get("past-end"))
        }
        Some(over) => Ok(Some(Some(over))),
        None => anyhow::bail!("{}", lang.get("not-a-cadence").replace("{said}", said)),
    }
}

pub fn undone(app: &mut App, selector: &str, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let archived: Vec<_> = app.state.archived_tasks().collect();
    resolved!(app, Some(selector), archived, lang, |id| {
        let ops = app.state.reopening(id);
        app.commit_all(ops)?;
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

pub fn drop(app: &mut App, selector: &str, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let open = app.state.ordered_open();
    resolved!(app, Some(selector), open, lang, |id| {
        let repeats = app
            .state
            .tasks
            .get(&id)
            .is_some_and(|task| task.repeat.is_some());
        app.commit(Op::TaskDrop { id })?;
        report(app, id, today, lang);
        if repeats {
            println!("  {}", style::dim(lang.get("repeat-ended")));
        }
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
    let over = cadence_flag(args.repeat.as_deref(), args.no_repeat, lang)?;
    let all: Vec<_> = app.state.tasks.values().collect();

    resolved!(app, Some(&args.selector), all, lang, |id| {
        let tags = merged_tags(app, id, &args.tag, &args.untag)?;

        let d = TaskPatch {
            title: args.title.clone(),
            date: date.map(Some).or(args.no_date.then_some(None)),
            deadline: deadline.map(Some).or(args.no_deadline.then_some(None)),
            priority: args
                .priority
                .as_deref()
                .map(|raw| named_priority(raw, lang))
                .transpose()?,
            tags,
            reminders: None,
            repeat: over,
        };

        if d == TaskPatch::default() {
            anyhow::bail!("{}", lang.get("nothing-to-change"));
        }

        app.commit(Op::TaskUpdate { id, d })?;
        let task = &app.state.tasks[&id];
        warn_if_backwards(task.date.as_ref(), task.deadline.as_ref(), lang);
        report(app, id, today, lang);
        Ok(ExitCode::SUCCESS)
    })
}

fn warn_if_backwards(
    date: Option<&tisty_core::DateSpec>,
    deadline: Option<&tisty_core::DateSpec>,
    lang: Lang,
) {
    if let (Some(date), Some(deadline)) = (date, deadline)
        && deadline.at < date.at
    {
        eprintln!("{}", lang.get("deadline-before-date"));
    }
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
        (Some(name), false) => match app.state.find_list(name).as_slice() {
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

pub fn named_priority(raw: &str, lang: Lang) -> anyhow::Result<tisty_core::Priority> {
    tisty_nl::parse_priority(raw.trim_start_matches('!'), lang.code())
        .ok_or_else(|| anyhow::anyhow!(lang.fill("not-a-priority", &[("value", raw)])))
}
