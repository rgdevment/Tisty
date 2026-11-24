use super::json;

const HOOKS: &str = "hooks";
const COMMANDS: [&str; 4] = ["command", "exec", "bash", "powershell"];

type Span = (usize, usize);

#[derive(Debug)]
pub struct Run {
    pub event: &'static str,
    pub programs: Vec<Program>,
    pub all_read: bool,
    list: usize,
    span: Span,
    only_in_list: bool,
}

#[derive(Debug)]
pub struct Program {
    pub said: String,
    value: Span,
    dress: Dress,
    posix: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Dress {
    Whole,
    Shell {
        lead: &'static str,
        quote: Quote,
        rest: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Quote {
    Bare,
    Double,
    Posix,
    PowerShell,
}

pub fn list(text: &str, event: &str) -> Option<usize> {
    let raw = text.as_bytes();
    let under = json::member(raw, json::opens(raw)?, HOOKS)?;
    Some(listed(raw, under.value.0, event)?.0)
}

pub fn list_fits(text: &str, event: &str) -> bool {
    let raw = text.as_bytes();
    let under = json::opens(raw).and_then(|root| json::member(raw, root, HOOKS));
    let Some(under) = under else {
        return true;
    };
    raw.get(under.value.0) == Some(&b'{')
        && (json::member(raw, under.value.0, event).is_none()
            || listed(raw, under.value.0, event).is_some())
}

fn listed(raw: &[u8], object: usize, name: &str) -> Option<Span> {
    if raw.get(object) != Some(&b'{') {
        return None;
    }
    let at = json::member(raw, object, name)?.value;
    (raw.get(at.0) == Some(&b'[')).then_some(at)
}

pub struct Spoken<'a> {
    pub mark: &'a str,
    /// Its `command` line is run by a POSIX shell, on Windows too.
    pub posix: bool,
}

pub fn found(text: &str, events: &[&'static str], spoken: &Spoken) -> Vec<Run> {
    events
        .iter()
        .flat_map(|event| in_event(text, event, spoken))
        .collect()
}

fn in_event(text: &str, event: &'static str, spoken: &Spoken) -> Vec<Run> {
    let raw = text.as_bytes();
    let Some(list) = list(text, event) else {
        return Vec::new();
    };
    let groups = json::elements(raw, list).unwrap_or_default();
    let mut all = Vec::new();
    for group in &groups {
        let last = raw.len();
        let alone = groups.len() == 1
            && !json::commented(&text[list..group.0])
            && !json::commented(&text[group.1..json::shut(raw, list).unwrap_or(last)]);
        if is_ours(text, group.0, spoken.mark) {
            all.push(run(text, event, list, *group, spoken, alone));
            continue;
        }
        let Some(within) = listed(raw, group.0, HOOKS) else {
            continue;
        };
        let inner = json::elements(raw, within.0).unwrap_or_default();
        for one in inner.iter().filter(|one| is_ours(text, one.0, spoken.mark)) {
            let whole = inner.len() == 1
                && !json::commented(&text[group.0..one.0])
                && !json::commented(&text[one.1..group.1]);
            let read = run(text, event, within.0, *one, spoken, false);
            all.push(match whole {
                true => Run {
                    list,
                    span: *group,
                    only_in_list: alone,
                    ..read
                },
                false => read,
            });
        }
    }
    all
}

fn run(
    text: &str,
    event: &'static str,
    list: usize,
    span: Span,
    spoken: &Spoken,
    only_in_list: bool,
) -> Run {
    let raw = text.as_bytes();
    let args = listed(raw, span.0, "args").map(|at| args_said(text, at.0));
    let shell: Option<String> =
        json::member(raw, span.0, "shell").and_then(|it| json::said(text, it.value));
    let bash = spoken.posix && shell.as_deref() != Some("powershell");
    let fields: Vec<(&str, Span)> = COMMANDS
        .iter()
        .filter_map(|field| Some((*field, json::member(raw, span.0, field)?.value)))
        .collect();
    let programs: Vec<Program> = fields
        .iter()
        .filter_map(|&(field, value)| {
            let said: String = json::said(text, value)?;
            let posix = field == "bash" || (field == "command" && bash);
            let whole = args
                .as_ref()
                .filter(|_| matches!(field, "command" | "exec"));
            let (dress, said) = match whole {
                Some(args) => args
                    .starts_with(spoken.mark)
                    .then_some((Dress::Whole, said))?,
                None => shelled(&said, spoken.mark, posix)?,
            };
            Some(Program {
                said,
                value,
                dress,
                posix,
            })
        })
        .collect();
    Run {
        event,
        all_read: !programs.is_empty() && programs.len() == fields.len(),
        programs,
        list,
        span,
        only_in_list,
    }
}

fn args_said(text: &str, list: usize) -> String {
    let all: Vec<String> = json::elements(text.as_bytes(), list)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|span| json::said(text, span))
        .collect();
    all.join(" ")
}

fn is_ours(text: &str, object: usize, mark: &str) -> bool {
    let raw = text.as_bytes();
    if raw.get(object) != Some(&b'{') {
        return false;
    }
    let mut said: Vec<String> = COMMANDS
        .iter()
        .filter_map(|field| json::said(text, json::member(raw, object, field)?.value))
        .collect();
    said.extend(listed(raw, object, "args").map(|at| args_said(text, at.0)));
    let said = said.join(" ");
    said.to_lowercase().contains("tisty") && said.contains(mark)
}

// Anything but the program and what follows it, as a wrapper, is the person's own way of running it.
fn shelled(said: &str, mark: &str, posix: bool) -> Option<(Dress, String)> {
    let cut = said.find(&format!(" {mark}"))?;
    let (head, rest) = said.split_at(cut);
    let (lead, body) = match head.strip_prefix("& ") {
        Some(_) if posix => return None,
        Some(body) => ("& ", body),
        None => ("", head),
    };
    let (quote, program) = unquoted(lead, body)?;
    let rest = rest.to_string();
    Some((Dress::Shell { lead, quote, rest }, program))
}

fn unquoted(lead: &str, body: &str) -> Option<(Quote, String)> {
    let inside = |quote: char| {
        (body.len() > 1)
            .then(|| body.strip_prefix(quote)?.strip_suffix(quote))
            .flatten()
    };
    if let Some(inner) = inside('"') {
        return (!inner.contains('"')).then(|| (Quote::Double, inner.to_string()));
    }
    if let Some(inner) = inside('\'') {
        let (quote, escape) = match lead.is_empty() {
            true => (Quote::Posix, "'\\''"),
            false => (Quote::PowerShell, "''"),
        };
        let plain: Vec<&str> = inner.split(escape).collect();
        return plain
            .iter()
            .all(|part| !part.contains('\''))
            .then(|| (quote, plain.join("'")));
    }
    let bare = !body.is_empty() && !body.contains(|c: char| c.is_whitespace() || "'\"".contains(c));
    bare.then(|| (Quote::Bare, body.to_string()))
}

impl Program {
    fn repointed(&self, new: &New) -> Option<String> {
        let command = new.command;
        let Dress::Shell { lead, quote, rest } = &self.dress else {
            return Some(command.to_string());
        };
        if self.posix && new.windows {
            return Some(format!("{}{rest}", bash_on_windows(command, *quote)?));
        }
        let said = match quote {
            Quote::Double if command.contains('"') => return None,
            Quote::Double => format!("{lead}\"{command}\""),
            Quote::Posix => format!("{lead}'{}'", command.replace('\'', "'\\''")),
            Quote::PowerShell => format!("{lead}'{}'", command.replace('\'', "''")),
            Quote::Bare if command.contains(|c: char| c.is_whitespace() || c == '\'') => {
                new.spoken.to_string()
            }
            Quote::Bare => format!("{lead}{command}"),
        };
        Some(format!("{said}{rest}"))
    }
}

// Bash eats a bare backslash, so there the program goes with `/`.
fn bash_on_windows(command: &str, quote: Quote) -> Option<String> {
    let command = command.replace('\\', "/");
    if command.contains(['"', '$', '`']) {
        return None;
    }
    let bare = |c: char| c.is_ascii_alphanumeric() || "/:._-".contains(c);
    Some(match quote == Quote::Bare && command.chars().all(bare) {
        true => command,
        false => format!("\"{command}\""),
    })
}

pub struct New<'a> {
    pub command: &'a str,
    pub spoken: &'a str,
    pub windows: bool,
}

pub fn repointed(
    text: &str,
    runs: &[Run],
    which: impl Fn(&Program) -> bool,
    new: &New,
) -> Option<String> {
    let mut edits: Vec<(Span, String)> = Vec::new();
    for program in runs
        .iter()
        .flat_map(|run| &run.programs)
        .filter(|it| which(it))
    {
        let said = serde_json::to_string(&program.repointed(new)?).ok()?;
        edits.push((program.value, said));
    }
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.0));
    let mut now = text.to_string();
    for (span, said) in edits {
        now.replace_range(span.0..span.1, &said);
    }
    Some(now)
}

pub fn taken_out(text: &str, run: &Run) -> Option<String> {
    match run.only_in_list {
        true => json::unset(text, HOOKS, run.event),
        false => json::cut(text, run.list, run.span),
    }
}

#[cfg(test)]
#[path = "running_test.rs"]
mod tests;
