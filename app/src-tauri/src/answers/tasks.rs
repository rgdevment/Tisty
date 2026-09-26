use std::sync::Mutex;

use tisty_core::event::{LogAdd, LogEdit, StepAdd, StepRef, StepText, TaskPatch};
use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Config, Op, Tag, Task};

use crate::{
    Answer, Asking, Change, Filter, Left, Logs, Refusal, Scope, Session, Snapshot, View, ahead,
    asking, coming, dated_field, erasing, finding, held, herald, language, offering,
    opening_to_agents, recalled, recurring, refusal_code, repeated, tagged, tags_in_use, tally,
    today, weighed, wiring, zone,
};

#[tauri::command]
pub fn task_left(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<Left>> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    let Some(task) = session.state.tasks.get(&id) else {
        return Err(Refusal::of("notATaskId"));
    };

    let root = session.paths.data().to_path_buf();
    let shared = session.shared_now();
    let on_disk = tisty_core::docs::all(&session.paths.docs());
    let named: std::collections::BTreeMap<&str, &tisty_core::docs::Doc> =
        on_disk.iter().map(|one| (one.id.as_str(), one)).collect();

    let left = task
        .references()
        .into_iter()
        .map(|one| match one.kind {
            // The window writes a document as `[title](tisty:doc/ID)`, which parses as a link.
            _ if one.target.starts_with("tisty:doc/") => {
                // A reference names the document by its file or by its id, depending on who wrote it.
                let held = one.target.strip_prefix("tisty:doc/").and_then(|raw| {
                    raw.parse()
                        .ok()
                        .and_then(|id| session.state.docs.get(&id))
                        .or_else(|| session.state.docs.values().find(|doc| doc.file == raw))
                });
                let on_paper = held.and_then(|doc| named.get(doc.file.as_str()));
                Left {
                    kind: "doc",
                    label: on_paper
                        .map(|doc| doc.title.clone())
                        .filter(|title| !title.is_empty())
                        .or_else(|| one.label.clone()),
                    away: held.is_some_and(|doc| session.state.held_away(doc)),
                    gone: held.is_none() || on_paper.is_none(),
                    target: one.target,
                    bytes: None,
                }
            }
            tisty_core::refs::Kind::Doc => Left {
                kind: "named",
                label: one.label.clone(),
                away: false,
                gone: false,
                target: one.target,
                bytes: None,
            },
            tisty_core::refs::Kind::Link
                if tisty_core::attach::names_an_attachment(&one.target) =>
            {
                let bytes = finding::where_it_lies(&one.target, &root, shared.as_deref())
                    .and_then(|at| std::fs::metadata(at).ok())
                    .filter(|told| told.is_file())
                    .map(|told| told.len());
                Left {
                    kind: "file",
                    label: one.label.clone(),
                    away: false,
                    gone: bytes.is_none(),
                    target: one.target,
                    bytes,
                }
            }
            tisty_core::refs::Kind::Link => Left {
                kind: if one.target.starts_with("http") {
                    "link"
                } else {
                    "named"
                },
                label: one.label.clone(),
                away: false,
                gone: false,
                target: one.target,
                bytes: None,
            },
        })
        .collect();
    Ok(left)
}

#[tauri::command]
pub fn task_story(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::story::Story> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    let told = tisty_core::story::story(session.log()?, id);
    Ok(told)
}

#[tauri::command]
pub fn task_series(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<tisty_core::series::Series>> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    Ok(tisty_core::series::series(&session.state, id))
}

#[tauri::command]
pub fn routines(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<Vec<tisty_core::series::Series>> {
    let mut session = held(&session);
    session.reload()?;
    Ok(tisty_core::series::routines(&session.state))
}

#[tauri::command]
pub fn archive_shape(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<tisty_core::shape::Shape> {
    let mut session = held(&session);
    session.reload()?;
    let now = jiff::Zoned::now();
    Ok(tisty_core::shape::shape(
        &session.state,
        18,
        &now.time_zone().clone(),
        now.date(),
    ))
}

#[tauri::command]
pub fn snapshot(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    view: Option<View>,
) -> Answer<Snapshot> {
    let mut session = held(&session);
    session.reload()?;
    let spoken = Config::load(&session.paths.config_file())
        .ok()
        .flatten()
        .and_then(|c| c.locale);
    if spoken != session.locale {
        session.locale = spoken.clone();
        language(&app, &spoken);
    }

    let filter = match view {
        Some(view) => view.resolve()?,
        None => Filter::default(),
    };

    Ok(Snapshot {
        tasks: session
            .state
            .matching(&filter, today())
            .into_iter()
            .cloned()
            .collect(),
        ahead: coming(&session.state, today()),
        routines: recurring(&session.state, today()),
        lists: session.state.ordered_lists().into_iter().cloned().collect(),
        tags: tags_in_use(&session.state),
        refs: session.state.references(),
        counts: tally(&session.state),
        locale: session.locale.clone(),
        agents: named_agents(&session.state),
        agent_tag: tisty_core::model::AGENT_TAG,
        hosts: session
            .state
            .hosts
            .iter()
            .map(|(agent, machine)| (agent.0.clone(), machine.0.clone()))
            .collect(),
        machines: session
            .state
            .devices
            .iter()
            .filter(|one| !session.state.assistants.contains(one))
            .map(|one| (one.0.clone(), tisty_core::config::nicknamed(&one.0)))
            .collect(),
        machine_here: session.config.device_id.0.clone(),
        clients: named_clients(&session.state),
    })
}

fn named_clients(state: &tisty_core::State) -> std::collections::BTreeMap<String, String> {
    state
        .tasks
        .values()
        .flat_map(|task| {
            task.created_via
                .iter()
                .chain(task.resolved.iter().filter_map(|one| one.via.as_ref()))
                .chain(task.log.iter().filter_map(|entry| entry.via.as_ref()))
        })
        .map(|via| (via.clone(), tisty_core::agent::client_named(via)))
        .collect()
}

fn named_agents(state: &tisty_core::State) -> std::collections::BTreeMap<String, String> {
    state
        .agents
        .iter()
        .chain(state.assistants.iter())
        .map(|one| (one.0.clone(), tisty_core::config::nicknamed(&one.0)))
        .collect()
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Edits {
    #[serde(default)]
    pub no_date: bool,
    #[serde(default)]
    pub no_deadline: bool,
    #[serde(default)]
    pub no_list: bool,
    #[serde(default)]
    pub no_priority: bool,
    #[serde(default)]
    pub no_repeat: bool,
    #[serde(default)]
    pub no_tags: Vec<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub take_offer: bool,
}

pub fn named_priority(raw: &str) -> Answer<tisty_core::Priority> {
    raw.parse().map_err(|_| Refusal::of("notAPriority"))
}

pub fn dated(raw: &str, now: &jiff::Zoned, spoken: &str) -> Result<tisty_core::DateSpec, Refusal> {
    if let Ok(day) = raw.parse::<jiff::civil::Date>() {
        return Ok(tisty_core::DateSpec::all_day(day, zone()));
    }
    if let Ok(when) = raw.parse::<jiff::civil::DateTime>() {
        return Ok(tisty_core::DateSpec::floating(when, zone()));
    }
    tisty_nl::parse_date(raw, now, spoken).ok_or_else(|| Refusal::about("notADate", raw))
}

#[tauri::command]
pub fn capture(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
    view: Option<View>,
    edits: Option<Edits>,
) -> Answer<Task> {
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let now = jiff::Zoned::now();
    let read = tisty_nl::parse(&text, &now, &spoken);
    let mut draft: tisty_core::capture::Draft = read.clone().into();

    if let Some(view) = view {
        if draft.filing.is_none()
            && let Some(list) = &view.list
        {
            let id = list.parse().map_err(|_| Refusal::of("notAListId"))?;
            draft.filing = Some(tisty_core::capture::Filing::Kept(id));
        }
        for name in &view.tags {
            if let Ok(tag) = Tag::written(name)
                && !draft.tags.contains(&tag)
            {
                draft.tags.push(tag);
            }
        }
        if draft.date.is_none() && view.window.as_deref() == Some("today") {
            draft.date = Some(tisty_core::DateSpec::all_day(today(), zone()));
        }
    }

    let edits = edits.unwrap_or_default();
    edits.apply(&mut draft, &now, &spoken)?;
    if let Some(spec) = &draft.deadline {
        ahead(spec, &now, "pastDeadline")?;
    }
    if let Some(title) = edits.retitled(&text, &read, &spoken) {
        draft.title = title;
    }

    let plan = tisty_core::capture::plan(&session.state, draft)?;
    session.commit_all(plan.ops)?;
    let task = session
        .state
        .tasks
        .get(&plan.task)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    drop(session);
    let _ = herald::told(
        &app,
        tisty_core::herald::Happening::Filed {
            title: task.title.clone(),
        },
    );
    Ok(task)
}

#[tauri::command]
pub fn read(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
) -> Answer<tisty_nl::Parsed> {
    let spoken = held(&session).locale.clone().unwrap_or(locale);
    Ok(tisty_nl::parse(&text, &jiff::Zoned::now(), &spoken))
}

#[tauri::command]
pub fn patch(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    change: Change,
    locale: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let now = jiff::Zoned::now();
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?
        .clone();

    let d = TaskPatch {
        title: change
            .title
            .as_deref()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty()),
        date: dated_field(change.date.as_deref(), change.no_date, &now, &spoken)?,
        deadline: {
            let field = dated_field(
                change.deadline.as_deref(),
                change.no_deadline,
                &now,
                &spoken,
            )?;
            if let Some(Some(spec)) = &field {
                ahead(spec, &now, "pastDeadline")?;
            }
            field
        },
        priority: change.priority.as_deref().map(named_priority).transpose()?,
        tags: tagged(&task, &change)?,
        reminders: recalled(&task, &change, &now)?,
        repeat: repeated(&change, &now)?,
        // Converting and opening to agents are verbs of their own, never part of an edit.
        ..Default::default()
    };

    let mut ops = Vec::new();
    let named = match change.list_named.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => Some(match session.state.list_called(name).as_slice() {
            [one] => one.id,
            [_, _, ..] => return Err(Refusal::about("manyLists", name)),
            [] => {
                let made = ulid::Ulid::generate();
                ops.push(Op::ListAdd {
                    id: made,
                    d: tisty_core::event::ListAdd {
                        name: name.to_string(),
                        order: session.state.next_list_order(),
                        color: None,
                    },
                });
                made
            }
        }),
        _ => None,
    };

    let filed = match (named, &change.list, change.inbox) {
        (Some(id), _, _) => Some(Some(id)),
        (None, Some(raw), _) => Some(Some(raw.parse().map_err(|_| Refusal::of("notAListId"))?)),
        (None, None, true) => Some(None),
        _ => None,
    };

    if d != TaskPatch::default() {
        ops.push(Op::TaskUpdate { id, d });
    }
    if let Some(body) = &change.description {
        let kept = body.trim().to_string();
        tisty_core::state::short_enough(&kept).map_err(|e| match e {
            tisty_core::Error::TextTooLong { limit, .. } => {
                Refusal::about("textTooLong", weighed(limit))
            }
            _ => Refusal::of("internal"),
        })?;
        ops.push(Op::TaskDescribe {
            id,
            d: tisty_core::event::Body {
                body: (!kept.is_empty()).then_some(kept),
            },
        });
    }
    if let Some(list) = filed {
        ops.push(Op::TaskMove {
            id,
            d: tisty_core::event::TaskMove {
                list: Some(list),
                order: None,
            },
        });
    }
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn write_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: Option<String>,
    text: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(Refusal::of("emptyStep"));
    }
    let mut session = held(&session);

    let op = match step {
        Some(raw) => Op::StepText {
            id,
            d: StepText {
                step: raw.parse().map_err(|_| Refusal::of("notAStepId"))?,
                text,
            },
        },
        None => Op::StepAdd {
            id,
            d: StepAdd {
                step: ulid::Ulid::generate(),
                text,
                order: tisty_core::order::last_of(
                    session
                        .state
                        .tasks
                        .get(&id)
                        .ok_or_else(|| Refusal::of("notATaskId"))?
                        .steps
                        .iter()
                        .map(|s| s.order.as_str()),
                ),
            },
        },
    };
    session.commit(op)?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn mark_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
    done: bool,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let d = StepRef {
        step: step.parse().map_err(|_| Refusal::of("notAStepId"))?,
    };
    let mut session = held(&session);
    session.commit(if done {
        Op::StepDone { id, d }
    } else {
        Op::StepUndone { id, d }
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn drop_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let d = StepRef {
        step: step.parse().map_err(|_| Refusal::of("notAStepId"))?,
    };
    let mut session = held(&session);
    session.commit(Op::StepRemove { id, d })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn write_log(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    entry: Option<String>,
    body: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(Refusal::of("emptyEntry"));
    }
    tisty_core::state::short_enough(&body).map_err(|e| match e {
        tisty_core::Error::TextTooLong { limit, .. } => {
            Refusal::about("textTooLong", weighed(limit))
        }
        _ => Refusal::of("internal"),
    })?;
    let mut session = held(&session);

    session.commit(match entry {
        Some(raw) => Op::TaskLogEdit {
            id,
            d: LogEdit {
                entry: raw.parse().map_err(|_| Refusal::of("notAnEntry"))?,
                body,
            },
        },
        None => Op::TaskLog {
            id,
            d: LogAdd::new(ulid::Ulid::generate(), body).in_zone(Some(zone())),
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn discard(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(Op::TaskDrop { id })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn fold(session: tauri::State<'_, Mutex<Session>>, id: String, away: bool) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(if away {
        Op::TaskHide { id }
    } else {
        Op::TaskShow { id }
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn still_open(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    let marked = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?
        .resolved
        .is_some();
    if marked {
        session.commit(Op::TaskUnresolve { id })?;
    }
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
pub fn note_break(kind: String, said: Option<String>, frames: String) {
    let cut = |text: String, most: usize| text.chars().take(most).collect::<String>();
    witness::error(
        channel::WINDOW,
        "the window broke and stopped drawing",
        &[
            ("kind", Fact::Why(cut(kind, 40))),
            ("said", Fact::Why(cut(said.unwrap_or_default(), 200))),
            ("frames", Fact::Why(cut(frames, 400))),
        ],
    );
}

#[tauri::command]
pub fn note_trouble(code: String, name: Option<String>) {
    let Some(code) = refusal_code(&code) else {
        return;
    };
    let mut facts = vec![("code", Fact::Code(code))];
    if let Some(name) = name.filter(|one| !one.is_empty()) {
        facts.push(("at", Fact::Path(std::path::PathBuf::from(name))));
    }
    witness::warn(channel::WINDOW, "the window showed a refusal", &facts);
}

#[tauri::command(async)]
pub fn logs(session: tauri::State<'_, Mutex<Session>>, most: usize) -> Answer<Logs> {
    let session = held(&session);
    Ok(Logs {
        at: witness::file(&session.paths).display().to_string(),
        bytes: witness::weighs(&session.paths),
        lines: if most == 0 {
            Vec::new()
        } else {
            witness::recent(&session.paths, most)
        },
    })
}

#[tauri::command]
pub fn star_due(session: tauri::State<'_, Mutex<Session>>) -> Answer<bool> {
    let mut session = held(&session);
    let now = jiff::Timestamp::now();
    let asked = session.config.asked_for_a_star;
    let since = session.config.here_since;
    let papers = session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none())
        .count();
    let counted = || {
        let filter = Filter {
            scope: Scope::Archived,
            ..Default::default()
        };
        let today = today();
        session
            .state
            .archived_tasks()
            .filter(|task| filter.matches(task, today))
            .count()
    };
    let decided = asking(asked, since, now, papers, counted);
    match decided {
        Asking::Start => {
            session.keep(|c| c.here_since = Some(now))?;
            Ok(false)
        }
        Asking::Wait => Ok(false),
        Asking::Now => Ok(true),
    }
}

#[tauri::command]
pub fn star_done(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    held(&session).keep(|c| c.asked_for_a_star = Some(true))
}

#[tauri::command]
pub fn door_due(session: tauri::State<'_, Mutex<Session>>) -> Answer<bool> {
    if held(&session).config.asked_to_wire.unwrap_or(false) {
        return Ok(false);
    }
    let seen = wiring::seen();
    Ok(offering(
        seen.len(),
        seen.iter().filter(|one| one.wired).count(),
    ))
}

#[tauri::command]
pub fn door_done(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    held(&session).keep(|c| c.asked_to_wire = Some(true))
}

#[tauri::command]
pub fn erase(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    erasing(&mut held(&session), id)
}

#[tauri::command]
pub fn open_to_agents(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    open: bool,
) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    opening_to_agents(&mut held(&session), id, open)
}

#[tauri::command]
pub fn reopen(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    let ops = session.state.reopening(id);
    session.commit_all(ops)?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}
