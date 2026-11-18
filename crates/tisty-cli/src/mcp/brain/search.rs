use serde_json::{Value, json};
use tisty_core::model::{Task, TaskId};
use tisty_core::{Paths, State};

use tisty_core::docs::Sighting;

use super::super::asked::{how_many, text};
use super::super::jsonrpc::{mirrored, told};
use super::super::looking::{
    UNPATHED, brief, how_it_ended, kept_here, list_of, placed, standing, standing_of,
};
use super::super::mistaken::no_document_called;
use super::super::unpathing::{PATH_MARK, fenced_in};
use super::super::{Refused, alike, already, doc_named, hitch, named_doc, opened, said, scoped};
use super::ranking::{Ranked, newest_line, newest_paper_line, ranked};

pub(in crate::mcp) fn found(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let today = jiff::Zoned::now().date();

    if let Some(source) = text(args, "source") {
        if text(args, "query").is_some() {
            return Err(Refused::Tool(
                "`brain` takes a `source` or a `query`, not both: one asks whether this exact thing was already filed, the other searches. Send one."
                    .into(),
            ));
        }
        let filed = already(&state, &source);
        if let Some(id) = filed.filter(|id| state.is_erased(*id)) {
            return Ok(told(
                "Already proposed from that source, and erased since: the person let it go. A \
                 new filing from this source takes `again`, and only if they want it back."
                    .into(),
                json!({ "found": { "id": id.to_string(), "erased": true } }),
            ));
        }
        if let Some(task) = filed
            .and_then(|id| state.tasks.get(&id))
            .filter(|one| one.hidden)
        {
            return Ok(told(
                format!(
                    "Already proposed from that source, and the person put it out of sight. {}",
                    super::super::chores::out_of_sight_again(task)
                ),
                json!({ "found": { "hidden": true } }),
            ));
        }
        let held = filed.and_then(|id| state.tasks.get(&id));
        return Ok(told(
            match held {
                Some(task) if !task.is_open() => format!(
                    "Already proposed from that source, and closed since ({}): {:?}. {}; a new \
                     filing from this source takes `again`.",
                    standing(task),
                    task.title,
                    how_it_ended(task)
                ),
                Some(task) => format!("Already proposed from that source: {:?}", task.title),
                None => "Nothing here came from that source.".into(),
            },
            json!({ "found": held.map(|task| brief(task, &state, today)) }),
        ));
    }

    if let Some(which) = text(args, "doc") {
        return inside_a_doc(paths, &state, &which, args);
    }

    let sifted = Sifted::asked(args)?;
    let query = text(args, "query");
    if query.is_none() && sifted.none() {
        return Err(Refused::Tool(
            "`brain` searches with words in `query`, or sifts with a value in one of `tag`, \
             `list`, `by_agent`, `said_done`, `part_of`, `from_source`, `from` and `to`; \
             these came empty."
                .into(),
        ));
    }
    let scope = scoped(args)?;
    let most = how_many(args, FOUND_BY_DEFAULT, FOUND_AT_MOST);
    let with_lines = args.get("lines").and_then(Value::as_bool) != Some(false);
    let terms = query
        .as_deref()
        .map(tisty_core::text::terms)
        .unwrap_or_default();
    let past = args.get("after").and_then(Value::as_u64).unwrap_or(0) as usize;
    let within = match text(args, "folder") {
        Some(said) => Some(super::super::folder_found(&state, &said)?),
        None => None,
    };
    let here = papers_here(&state, scope, within);

    // Counting what it cannot see would still say the thing exists.
    let keeps = |one: &Task| scope.keeps(one) && !one.folded() && sifted.keeps(one, &state);
    let paper_kept = |file: &str| sifted.none() && here.contains_key(file);
    let Ranked {
        tasks: hits,
        lined,
        papers: seen,
        every_word,
    } = match &query {
        Some(query) => ranked(paths, &state, query, scope, &keeps, &paper_kept),
        None => {
            let mut all: Vec<&Task> = state.tasks.values().filter(|one| keeps(one)).collect();
            all.sort_by_key(|one| (one.date.as_ref().map(|d| d.date()), one.id));
            Ranked {
                tasks: all,
                lined: Default::default(),
                papers: Vec::new(),
                every_word: true,
            }
        }
    };
    let all = hits.len();
    let hits: Vec<&Task> = hits.into_iter().skip(past).take(most).collect();
    let mut hid = false;
    let wheres: Vec<Option<(&str, String)>> = hits
        .iter()
        .map(|task| {
            if !with_lines {
                return None;
            }
            let (kind, (shown, unpathed)) = match lined.get(&task.id) {
                Some(line) => ("document", quoted(line, kept_here(line), &terms)),
                None => {
                    let (kind, line) = newest_line(task, &terms)?;
                    let kept = kept_where(task, kind, &line);
                    let by = tisty_core::view::unnamed(task, &terms);
                    (kind, quoted(&line, kept, &by))
                }
            };
            hid |= unpathed;
            Some((kind, shown))
        })
        .collect();
    let found: Vec<Value> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| {
            let mut one = brief(task, &state, today);
            if let Some((kind, line)) = said {
                one["in"] = json!(kind);
                one["line"] = json!(line);
            }
            one
        })
        .collect();
    let (papers, papers_all) = papers_matching(&seen, &here, most);
    let papers: Vec<Value> = papers
        .into_iter()
        .map(|mut one| {
            let shown = match with_lines && !said(&one, "line").is_empty() {
                true => {
                    let line = newest_paper_line(paths, said(&one, "doc"), &terms)
                        .unwrap_or_else(|| said(&one, "line").to_string());
                    let (shown, unpathed) = quoted(&line, kept_here(&line), &terms);
                    hid |= unpathed;
                    shown
                }
                false => String::new(),
            };
            one["line"] = json!(shown);
            one
        })
        .collect();
    let mut lines: Vec<String> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| match said {
            Some((kind, line)) => format!(
                "{:?} {} ({}) [{}]\n  {kind}: {line}",
                task.title,
                placed(&state, task),
                standing_said(task, today),
                task.id
            ),
            None => format!(
                "{:?} {} ({}) [{}]",
                task.title,
                placed(&state, task),
                standing_said(task, today),
                task.id
            ),
        })
        .collect();
    lines.extend(papers.iter().map(|one| {
        let put_away = if one["archived"] == json!(true) {
            ", put away"
        } else {
            ""
        };
        let what = match one["page_of"].as_str() {
            Some(up) => format!("a page of {}", super::super::doc_titled(&state, up)),
            None => "document".into(),
        };
        let marked = match one["marked"].as_str() {
            Some(how) => format!(", marked as {how}, waiting for the person"),
            None => String::new(),
        };
        let line = match said(one, "line") {
            "" => String::new(),
            line => format!("\n  line: {line}"),
        };
        format!(
            "{:?} ({what}{put_away}{marked}) [{}]{line}",
            said(one, "title"),
            said(one, "doc")
        )
    }));
    let asked = query.clone().unwrap_or_else(|| sifted.said());
    if hid {
        lines.push(UNPATHED.to_string());
    }
    let mut whole = json!({
        "matches": found,
        "total": all,
        "docs": papers,
        "docsTotal": papers_all,
    });
    if !every_word {
        whole["every_word"] = json!(false);
        lines.insert(0, NOT_EVERY_WORD.to_string());
    }
    if past + found.len() < all {
        whole["next"] = json!(format!(
            "`after` {} brings the next of the {all} tasks.",
            past + found.len()
        ));
    }
    if hid {
        whole["unpathed"] = json!(UNPATHED);
    }
    Ok(mirrored(
        format!(
            "{all} task(s) and {papers_all} document(s) match {asked:?}; showing {} and {}.
{}",
            found.len(),
            papers.len(),
            lines.join(
                "
"
            )
        ),
        whole,
    ))
}
struct Sifted {
    tag: Option<String>,
    list: Option<String>,
    by_agent: Option<bool>,
    said_done: Option<bool>,
    part_of: Option<TaskId>,
    from_source: Option<String>,
    from: Option<jiff::civil::Date>,
    to: Option<jiff::civil::Date>,
}

pub(super) fn inside_a_doc(
    paths: &Paths,
    state: &State,
    which: &str,
    args: &Value,
) -> Result<Value, Refused> {
    let Some(query) = text(args, "query") else {
        return Err(Refused::Tool(
            "looking inside a document needs a `query` to look for.".into(),
        ));
    };
    // A hidden task's words stay out of sight here as they do in `read`.
    let seen = state
        .docs
        .values()
        .find(|one| one.file == which)
        .filter(|one| state.doc_in_sight(one));
    if seen.is_none() {
        return Err(Refused::Tool(no_document_called(state, which)));
    }
    let named = doc_named(state, which);
    let body = tisty_core::docs::read(&paths.docs(), which).map_err(hitch)?;
    let most = how_many(args, FOUND_BY_DEFAULT, FOUND_AT_MOST);

    let terms = tisty_core::text::terms(&query);
    if terms.is_empty() {
        return Err(Refused::Tool(
            "looking inside a document needs a `query` with a word in it.".into(),
        ));
    }
    let lines: Vec<&str> = body.lines().collect();
    let at: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, one)| {
            let flat = tisty_core::text::folded(one);
            terms.iter().all(|term| flat.contains(term.as_str()))
        })
        .map(|(n, _)| n)
        .collect();
    let all = at.len();

    let outline = tisty_core::docs::outlined(&body);
    let section_of = |line: usize| {
        outline
            .iter()
            .rev()
            .find(|one| one.line <= line && line <= one.to)
            .map(|one| json!({ "at": one.at, "title": one.title }))
    };
    let reach = args
        .get("around")
        .and_then(Value::as_u64)
        .map_or(AROUND_A_HIT, |asked| (asked as usize).min(AROUND_AT_MOST));
    let shown: Vec<usize> = at.iter().copied().take(most).collect();
    let mut room = 0usize;
    let found: Vec<Value> = neighbourhoods(&shown, reach, lines.len())
        .into_iter()
        .map(|(n, around)| {
            let mut hit = serde_json::Map::new();
            hit.insert("line".into(), json!(n + 1));
            hit.insert(
                "text".into(),
                json!(tisty_core::docs::cut_around(
                    lines[n].to_string(),
                    &terms,
                    HIT_AT_MOST
                )),
            );
            if around
                .clone()
                .any(|at| at != n && !lines[at].trim().is_empty())
            {
                let around: Vec<String> = around
                    .map(|at| match at == n {
                        true => format!("(line {})", n + 1),
                        false => tisty_core::view::clipped(lines[at]),
                    })
                    .collect();
                hit.insert("around".into(), json!(around.join("\n")));
            }
            if let Some(section) = section_of(n + 1) {
                hit.insert("section".into(), section);
            }
            Value::Object(hit)
        })
        .take_while(|one| {
            let first = room == 0;
            room += said(one, "text").chars().count() + said(one, "around").chars().count();
            first || room <= super::super::WHOLE_UP_TO
        })
        .collect();

    let said = match all {
        0 => format!("Nothing in {named} says {query:?}."),
        _ => format!(
            "{all} line(s) of {named} say {query:?}; showing {}.\n{}{}",
            found.len(),
            found
                .iter()
                .map(|one| {
                    let head = match one["section"].is_object() {
                        true => format!(
                            "line {} (section {}, {}) — {}",
                            one["line"],
                            one["section"]["at"],
                            said(&one["section"], "title"),
                            said(one, "text")
                        ),
                        false => format!("line {} — {}", one["line"], said(one, "text")),
                    };
                    match said(one, "around") {
                        "" => head,
                        around => format!("{head}\n  around it:\n{}", indented(around)),
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            match all > found.len() {
                true => "\nThe rest: narrow the words, or read on with `read_doc` `from` a line.",
                false => "",
            }
        ),
    };
    Ok(mirrored(
        said,
        json!({
            "doc": which,
            "lines": found,
            "total": all,
            "print": tisty_core::attach::printed(body.as_bytes()),
        }),
    ))
}
const AROUND_A_HIT: usize = 1;
const AROUND_AT_MOST: usize = 10;
const HIT_AT_MOST: usize = 1_000;
const FOUND_BY_DEFAULT: u64 = 10;
const FOUND_AT_MOST: u64 = 100;
const LINE_FOUND_AT_MOST: usize = 120;
const NOT_EVERY_WORD: &str =
    "No task or document holds every word, so these hold the most of them.";

// Paths go before the cut, so none is left half shown; a word that was only in one is sought at its mark.
fn quoted(line: &str, kept: String, by: &[String]) -> (String, bool) {
    let unpathed = kept != line;
    let lost = unpathed && {
        let flat = tisty_core::text::folded(&kept);
        !by.iter().any(|term| flat.contains(term.as_str()))
    };
    let shown = match lost {
        true => tisty_core::docs::cut_around(kept, &[PATH_MARK.to_string()], LINE_FOUND_AT_MOST),
        false => tisty_core::docs::cut_around(kept, by, LINE_FOUND_AT_MOST),
    };
    let marked = unpathed && shown.contains(PATH_MARK);
    (shown, marked)
}

// A line two hits share is shown once, with the earlier of them, so hits close together do not pay for it twice.
fn neighbourhoods(
    hits: &[usize],
    reach: usize,
    lines: usize,
) -> Vec<(usize, std::ops::RangeInclusive<usize>)> {
    let mut free_from = 0;
    hits.iter()
        .enumerate()
        .map(|(nth, &hit)| {
            let first = hit.saturating_sub(reach).max(free_from);
            let reached = (hit + reach).min(lines.saturating_sub(1));
            let last = hits
                .get(nth + 1)
                .map_or(reached, |next| reached.min(next - 1));
            free_from = last + 1;
            (hit, first..=last)
        })
        .collect()
}

fn indented(lines: &str) -> String {
    lines
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
/// A line `brain` quotes is read as it sits where it came from: inside a fence it is an example.
fn kept_where(task: &Task, kind: &str, line: &str) -> String {
    let fenced = match kind {
        "journal" => task.journal().any(|one| fenced_in(&one.body, line)),
        "description" => task
            .description
            .as_deref()
            .is_some_and(|body| fenced_in(body, line)),
        _ => false,
    };
    if fenced {
        line.to_string()
    } else {
        kept_here(line)
    }
}

type Here = std::collections::HashMap<String, (bool, Option<String>, Option<String>)>;

fn papers_here(
    state: &State,
    scope: tisty_core::view::Scope,
    within: Option<tisty_core::model::FolderId>,
) -> Here {
    let inside = |one: &tisty_core::model::Kept| {
        within.is_none_or(|at| {
            one.folder
                .is_some_and(|it| it == at || super::super::placing::beneath(state, it, at))
        })
    };
    state
        .papers()
        .filter(|one| state.doc_kept_by(scope, one) && inside(one))
        .map(|one| {
            let marked = one
                .flagged
                .as_ref()
                .filter(|_| !state.held_away(one))
                .map(|how| {
                    super::super::papers::flagging::reason_said(
                        state,
                        how.reason.as_deref(),
                        how.instead.as_deref(),
                    )
                });
            (
                one.file.clone(),
                (
                    state.held_away(one),
                    one.page_of.and_then(|up| named_doc(state, up)),
                    marked,
                ),
            )
        })
        .collect()
}

// What was marked old waits for the person, so it comes after whatever still holds.
fn papers_matching(seen: &[Sighting], here: &Here, most: usize) -> (Vec<Value>, usize) {
    let mut seen: Vec<&Sighting> = seen
        .iter()
        .filter(|one| here.contains_key(&one.id))
        .collect();
    seen.sort_by_key(|one| here.get(&one.id).is_some_and(|(_, _, how)| how.is_some()));
    let all = seen.len();
    let shown = seen
        .into_iter()
        .take(most)
        .map(|one| {
            let (archived, page_of, marked) =
                here.get(&one.id).cloned().unwrap_or((false, None, None));
            let mut hit = json!({
                "doc": one.id,
                "title": one.title,
                "line": one.line,
                "page_of": page_of,
                "archived": archived,
                "flagged": marked.is_some(),
            });
            if let Some(how) = marked {
                hit["marked"] = json!(how);
            }
            hit
        })
        .collect();
    (shown, all)
}

pub(super) fn sightings(paths: &Paths, query: &str) -> Vec<tisty_core::docs::Sighting> {
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    tisty_core::docs::sighted(&paths.docs(), held.as_ref(), query, usize::MAX, |_| true)
        .unwrap_or_else(|| {
            tisty_core::docs::Corpus::default()
                .searching(&paths.docs(), query, usize::MAX, |_| true)
        })
}

impl Sifted {
    fn asked(args: &Value) -> Result<Self, Refused> {
        let on = |key: &str| -> Result<Option<jiff::civil::Date>, Refused> {
            let Some(said) = text(args, key) else {
                return Ok(None);
            };
            said.parse::<jiff::civil::Date>().map(Some).map_err(|_| {
                Refused::Tool(format!(
                    "`{key}` has to be a plain date like 2026-08-31, not {said:?}."
                ))
            })
        };
        Ok(Self {
            tag: text(args, "tag").map(|one| one.trim_start_matches('#').to_lowercase()),
            list: text(args, "list").map(|one| one.to_lowercase()),
            by_agent: args.get("by_agent").and_then(Value::as_bool),
            said_done: args.get("said_done").and_then(Value::as_bool),
            part_of: match text(args, "part_of") {
                None => None,
                Some(said) => Some(said.parse::<TaskId>().map_err(|_| {
                    Refused::Tool(format!(
                        "`part_of` takes a task id, and {said:?} is not one."
                    ))
                })?),
            },
            from_source: text(args, "from_source").map(|one| alike(&one)),
            from: on("from")?,
            to: on("to")?,
        })
    }

    fn none(&self) -> bool {
        self.tag.is_none()
            && self.list.is_none()
            && self.by_agent.is_none()
            && self.said_done.is_none()
            && self.part_of.is_none()
            && self.from_source.is_none()
            && self.from.is_none()
            && self.to.is_none()
    }

    fn said(&self) -> String {
        let mut all = Vec::new();
        if let Some(one) = &self.tag {
            all.push(format!("#{one}"));
        }
        if let Some(one) = &self.list {
            all.push(format!("in {one}"));
        }
        match self.by_agent {
            Some(true) => all.push("filed by an agent".into()),
            Some(false) => all.push("written by the person".into()),
            None => {}
        }
        match self.said_done {
            Some(true) => all.push("already said done".into()),
            Some(false) => all.push("not said done yet".into()),
            None => {}
        }
        if let Some(one) = &self.part_of {
            all.push(format!("parts of {one}"));
        }
        if let Some(one) = &self.from_source {
            all.push(format!("out of {one}"));
        }
        if let (Some(from), Some(to)) = (self.from, self.to) {
            all.push(format!("{from} to {to}"));
        } else if let Some(from) = self.from {
            all.push(format!("from {from}"));
        } else if let Some(to) = self.to {
            all.push(format!("up to {to}"));
        }
        all.join(", ")
    }

    fn keeps(&self, task: &Task, state: &State) -> bool {
        if let Some(want) = &self.tag
            && !task
                .tags
                .iter()
                .any(|one| one.as_str().to_lowercase() == *want)
        {
            return false;
        }
        if let Some(want) = &self.list {
            let named = list_of(state, task).map(str::to_lowercase);
            if named.as_deref() != Some(want.as_str()) {
                return false;
            }
        }
        if let Some(want) = self.by_agent {
            let by = task
                .created_by
                .as_ref()
                .is_some_and(|who| state.assistants.contains(who));
            if by != want {
                return false;
            }
        }
        if let Some(want) = self.said_done
            && (task.resolved.is_some() && task.is_open()) != want
        {
            return false;
        }
        if let Some(want) = self.part_of
            && task.part_of != Some(want)
        {
            return false;
        }
        if let Some(want) = &self.from_source {
            let came = task.source.as_deref().map(alike);
            if !came.is_some_and(|one| one.starts_with(want.as_str())) {
                return false;
            }
        }
        if self.from.is_some() || self.to.is_some() {
            let days: Vec<jiff::civil::Date> = [task.date.as_ref(), task.deadline.as_ref()]
                .into_iter()
                .flatten()
                .map(|one| one.date())
                .collect();
            if days.is_empty() {
                return false;
            }
            let within = days.iter().any(|one| {
                self.from.is_none_or(|first| *one >= first)
                    && self.to.is_none_or(|last| *one <= last)
            });
            if !within {
                return false;
            }
        }
        true
    }
}

// A reader of the text alone still learns what presses, not only what `structuredContent` says.
fn standing_said(task: &Task, today: jiff::civil::Date) -> String {
    let status = standing(task);
    match standing_of(task, today)
        .and_then(|how| json!(how).as_str().map(|one| one.replace('_', " ")))
    {
        Some(how) => format!("{status}, {how}"),
        None => status,
    }
}
