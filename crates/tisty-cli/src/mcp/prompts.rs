use serde_json::{Value, json};
use tisty_core::Paths;

use super::looking::named_task;
use super::resuming::{last_task, left_off, own_events_newest_first};
use super::{TOOLS_STAY_FRESH, admitted};

const SAID_AT_MOST: usize = 2_000;

struct Prompt {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    takes: &'static str,
    takes_what: &'static str,
    asks: fn(&Paths, Option<&str>) -> String,
}

const PROMPTS: [Prompt; 5] = [
    Prompt {
        name: "pick_up",
        title: "Pick up where I left off",
        description: "What moved since last time, then the open task last worked on and its \
                      latest notes.",
        takes: "task",
        takes_what: "A word from the task to pick up; left out, the one last worked on",
        asks: picking_up,
    },
    Prompt {
        name: "leave_off",
        title: "Leave this where I can pick it up",
        description: "Tick what is finished and note what was done, ruled out and comes next, so \
                      the next session starts from there.",
        takes: "task",
        takes_what: "A word from the task to leave; left out, the one we are on",
        asks: leaving_off,
    },
    Prompt {
        name: "plan_task",
        title: "Plan this work as a task with steps",
        description: "One task with its steps in its list, once no task already covers it.",
        takes: "work",
        takes_what: "What is to be done; left out, what we have been talking about",
        asks: planning,
    },
    Prompt {
        name: "solved_before",
        title: "How did we solve this before?",
        description: "Search the archive of finished tasks and the documents before trying \
                      anything new.",
        takes: "problem",
        takes_what: "A word for the problem; left out, the one we are on",
        asks: looking_back,
    },
    Prompt {
        name: "keep_decision",
        title: "Keep this decision in its document",
        description: "Write the decision just made into the one document on its subject, in \
                      place.",
        takes: "subject",
        takes_what: "What the decision is about; left out, plain from the conversation",
        asks: keeping,
    },
];

pub(super) fn listed() -> Value {
    let prompts: Vec<Value> = PROMPTS
        .iter()
        .map(|one| {
            json!({
                "name": one.name,
                "title": one.title,
                "description": one.description,
                "arguments": [{ "name": one.takes, "description": one.takes_what }],
            })
        })
        .collect();
    json!({
        "resultType": "complete",
        "prompts": prompts,
        "ttlMs": TOOLS_STAY_FRESH,
        "cacheScope": "public",
    })
}

pub(super) fn got(paths: &Paths, params: &Value) -> Result<Value, (i32, String)> {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let Some(prompt) = PROMPTS.iter().find(|one| one.name == name) else {
        let all: Vec<&str> = PROMPTS.iter().map(|one| one.name).collect();
        return Err((
            -32602,
            format!("unknown prompt: {name:?}. These exist: {}.", all.join(", ")),
        ));
    };
    let said = argument(prompt, params)?;
    Ok(json!({
        "resultType": "complete",
        "description": prompt.description,
        "messages": [{
            "role": "user",
            "content": { "type": "text", "text": (prompt.asks)(paths, said.as_deref()) },
        }],
    }))
}

fn argument(prompt: &Prompt, params: &Value) -> Result<Option<String>, (i32, String)> {
    let given = match params.get("arguments") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Object(given)) => given,
        Some(_) => {
            return Err((-32602, "`arguments` is an object of texts.".into()));
        }
    };
    if let Some(other) = given.keys().find(|key| *key != prompt.takes) {
        return Err((
            -32602,
            format!(
                "{} takes only `{}`, not `{other}`.",
                prompt.name, prompt.takes
            ),
        ));
    }
    let said = match given.get(prompt.takes) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(said)) => said.trim(),
        Some(_) => return Err((-32602, format!("`{}` is a text.", prompt.takes))),
    };
    if said.chars().count() > SAID_AT_MOST {
        return Err((
            -32602,
            format!(
                "`{}` is longer than {SAID_AT_MOST} characters. Shorten it.",
                prompt.takes
            ),
        ));
    }
    if said
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err((
            -32602,
            format!(
                "`{}` carries control characters. Send plain text.",
                prompt.takes
            ),
        ));
    }
    Ok((!said.is_empty()).then(|| said.to_string()))
}

fn picking_up(paths: &Paths, said: Option<&str>) -> String {
    let which = match said {
        Some(said) => {
            format!(
                "`find` the open task about {} — ask me if several fit — and `read` it",
                quoted(said)
            )
        }
        None => match last_worked_on(paths) {
            Some(task) => format!("`read` {task}, the open task last worked on here,"),
            None => "`find` the open task we were on — ask me if you cannot tell — and `read` it"
                .to_string(),
        },
    };
    format!(
        "Pick up where I left off in Tisty. Call `catch_up` first, with no `since`: it brings what \
         moved since you were last here and who moved it. Then {which} with its steps, its \
         page and its latest notes. Tell me in a few lines what moved, where the task stands, \
         what was done last and what comes next, naming tasks by title and list, never by id. \
         Change nothing yet."
    )
}

fn leaving_off(paths: &Paths, said: Option<&str>) -> String {
    let which = match said {
        Some(said) => format!(
            "the open task about {} — `find` it, and ask me if several fit —",
            quoted(said)
        ),
        None => match last_worked_on(paths) {
            Some(task) => format!("the task we have been on, {task} unless it was another,"),
            None => "the task we have been on".to_string(),
        },
    };
    format!(
        "In Tisty, leave {which} where the next session can pick it up. `read` it first. `tick` \
         every step that is finished. `note` what was done, what was tried and ruled out and why, \
         and what comes next, in that order. If every step is done and the work holds, \
         `say_done` with the proof. A decision that outlasts the task goes in its document, \
         changed in place with `edit_doc`, and the note says where. Change nothing else, and \
         tell me in a few lines what you left."
    )
}

fn quoted(said: &str) -> String {
    format!("«{said}» (or all I typed after the command, if a client cut it at a space)")
}

fn last_worked_on(paths: &Paths) -> Option<String> {
    let (state, agent) = admitted(paths).ok()??;
    let left = left_off(paths, &state, own_events_newest_first(paths, &agent));
    last_task(&state, &left).map(|task| named_task(&state, task))
}

fn planning(_: &Paths, said: Option<&str>) -> String {
    let work = match said {
        Some(said) => format!("{}, with what we have said about it,", quoted(said)),
        None => "the work we have been talking about".to_string(),
    };
    format!(
        "Plan {work} as one task in Tisty. Ask `rules` «tasks» first. Then `find` whether a task \
         already covers it, the archive included: an open one takes the new steps with `plan` \
         instead of a second task, and a closed one is read for how it went. Otherwise \
         `propose` one task with its steps in order — parts only for pieces finished on their \
         own — in the list it belongs in (`lists`; ask me when none fits), with a date only if I \
         gave one. Then tell me its title, its list and its steps."
    )
}

fn looking_back(_: &Paths, said: Option<&str>) -> String {
    let problem = match said {
        Some(said) => quoted(said),
        None => "the problem we are on".to_string(),
    };
    format!(
        "How did we solve {problem} before? Search Tisty before trying anything: `find` reaches \
         the archive, where every finished task is kept whole, and the documents on the subject. \
         `read` the closest tasks for how they ended, their steps and their notes, and \
         `outline_doc` the documents that cover it. Tell me what was done, when, and where it is \
         written, by title and list or folder. If nothing turns up, say so plainly rather than \
         guessing."
    )
}

fn keeping(_: &Paths, said: Option<&str>) -> String {
    let about = match said {
        Some(said) => format!("the one document about {}", quoted(said)),
        None => "the one document on its subject".to_string(),
    };
    format!(
        "Keep the decision we just made in Tisty, in {about}. Ask `rules` «documents» first. \
         `find` that document, `outline_doc` it, and write the decision with `edit_doc` in the \
         passage it belongs to, dated and with its reason; a decision it replaces is struck \
         through and dated, never left standing beside it. If no document covers the subject, \
         ask me which folder it belongs in before `write_doc`. Tell me the document's title and \
         folder."
    )
}
