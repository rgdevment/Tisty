struct Row {
    start: usize,
    end: usize,
    entry: usize,
    line: usize,
}

pub fn holds(text: &str, bridge: &str, name: &str) -> bool {
    find_row(body_of(text).1, bridge, name).is_some()
}

pub fn reads(text: &str, bridge: &str, name: &str) -> Option<String> {
    let body = body_of(text).1;
    let row = find_row(body, bridge, name)?;
    let starts = line_starts(body);
    for j in row.line + 1..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        let said = said(line_of(body, &starts, j));
        if said.starts_with("- ") {
            continue;
        }
        if let Some(said) = field(said, "command") {
            return scalar(said);
        }
    }
    None
}

pub fn set(text: &str, bridge: &str, name: &str, command: &str) -> Option<String> {
    let (mark, body) = body_of(text);
    if body.trim().is_empty() {
        return Some(written(command, "  ", "\n", bridge, name));
    }
    let step = step_of(body);
    let eol = eol_of(body);
    let now = match find_row(body, bridge, name) {
        Some(row) => set_in_row(body, &row, command, step, bridge, name)?,
        None => match replaced_empty(body, command, step, eol, bridge, name) {
            Some(now) => now,
            None if sequence(body) => appended(body, command, step, eol, bridge, name),
            None => return None,
        },
    };
    let sound =
        reads(&now, bridge, name).as_deref() == Some(command) && same_but(body, &now, bridge, name);
    sound.then(|| format!("{}{}", &text[..mark], now))
}

pub fn unset(text: &str, bridge: &str, name: &str) -> Option<String> {
    let (mark, body) = body_of(text);
    let row = find_row(body, bridge, name)?;
    let eol = eol_of(body);
    let mut now = format!("{}{}", &body[..row.start], &body[row.end..]);
    let starts = line_starts(&now);
    if entry_bare(&now, &starts, row.entry) {
        let end = starts
            .iter()
            .find(|&&at| at > row.entry)
            .copied()
            .unwrap_or(now.len());
        now = format!("{}{}", &now[..row.entry], &now[end..]);
    }
    if now.trim().is_empty() {
        now = format!("[]{eol}");
    }
    (!holds(&now, bridge, name) && same_but(body, &now, bridge, name))
        .then(|| format!("{}{}", &text[..mark], now))
}

pub fn repoint(text: &str, bridge: &str, name: &str, command: &str) -> Option<String> {
    let (mark, body) = body_of(text);
    let row = find_row(body, bridge, name)?;
    let now = replace_field(body, &row, "command", &plain(command))?;
    let sound =
        reads(&now, bridge, name).as_deref() == Some(command) && same_but(body, &now, bridge, name);
    sound.then(|| format!("{}{}", &text[..mark], now))
}

pub fn same_but(was: &str, now: &str, bridge: &str, name: &str) -> bool {
    bare(body_of(was).1, bridge, name) == bare(body_of(now).1, bridge, name)
}

pub fn pasted(bridge: &str, name: &str, command: &str) -> String {
    let said = |it: &str| format!("'{}'", it.replace('\'', "''"));
    format!(
        "- insert:\n    - id: {}\n      name: {}\n      config:\n        serverName: {}\n        transport: stdio\n        command: {}\n        args: [mcp]",
        said(name),
        said(bridge),
        said(name),
        said(command)
    )
}

fn set_in_row(
    body: &str,
    row: &Row,
    command: &str,
    step: &str,
    bridge: &str,
    name: &str,
) -> Option<String> {
    let said = plain(command);
    let now = replace_field(body, row, "command", &said)
        .or_else(|| insert_under_config(body, row, "command", &said, step))?;
    let row = find_row(&now, bridge, name)?;
    replace_field(&now, &row, "args", "[mcp]")
        .or_else(|| insert_under_config(&now, &row, "args", "[mcp]", step))
}

fn replace_field(text: &str, row: &Row, key: &str, value: &str) -> Option<String> {
    let starts = line_starts(text);
    for j in row.line..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        let line = line_of(text, &starts, j);
        if let Some((from, to)) = value_span(line, key) {
            let at = starts[j];
            return Some(format!(
                "{}{}{}",
                &text[..at + from],
                value,
                &text[at + to..]
            ));
        }
    }
    None
}

fn insert_under_config(
    text: &str,
    row: &Row,
    key: &str,
    value: &str,
    step: &str,
) -> Option<String> {
    let starts = line_starts(text);
    let mut config: Option<usize> = None;
    for j in row.line..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        let said = said(line_of(text, &starts, j));
        if !said.starts_with("- ") && field(said, "config").is_some() {
            config = Some(j);
        }
    }
    let Some(config) = config else {
        return insert_config(text, row, key, value, step);
    };
    let config_line = line_of(text, &starts, config);
    let mut last = config;
    for j in config + 1..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        let line = line_of(text, &starts, j);
        if said(line).is_empty() {
            continue;
        }
        if indent_of(line) > indent_of(config_line) {
            last = j;
        } else {
            break;
        }
    }
    let line = line_of(text, &starts, last);
    let pad = match last == config {
        true => format!("{}{}", leading(config_line), step),
        false => leading(line).to_string(),
    };
    Some(insert_after(text, last, &format!("{pad}{key}: {value}")))
}

fn insert_config(text: &str, row: &Row, key: &str, value: &str, step: &str) -> Option<String> {
    let starts = line_starts(text);
    let eol = eol_of(text);
    let mut last = row.line;
    for j in row.line..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        if !said(line_of(text, &starts, j)).is_empty() {
            last = j;
        }
    }
    let line = line_of(text, &starts, last);
    let pad = match last == row.line {
        true => format!("{}{}", leading(line), step),
        false => leading(line).to_string(),
    };
    let piece = format!("{pad}config:{eol}{pad}{step}{key}: {value}");
    Some(insert_after(text, last, &piece))
}

fn appended(body: &str, command: &str, step: &str, eol: &str, bridge: &str, name: &str) -> String {
    let starts = line_starts(body);
    for i in 0..starts.len() {
        let line = line_of(body, &starts, i);
        if indent_of(line) != 0 || said(line) != "- insert:" {
            continue;
        }
        let mut item: Option<String> = None;
        let mut last = i;
        for j in i + 1..starts.len() {
            let next = line_of(body, &starts, j);
            if said(next).is_empty() {
                continue;
            }
            if indent_of(next) == 0 {
                break;
            }
            if item.is_none() && said(next).starts_with("- ") {
                item = Some(leading(next).to_string());
            }
            last = j;
        }
        let fallback = step.repeat(2);
        let rows = row_at(
            command,
            item.as_deref().unwrap_or(&fallback),
            step,
            eol,
            bridge,
            name,
        );
        return insert_after(body, last, &rows);
    }
    let mut now = body.to_string();
    if !now.is_empty() && !now.ends_with('\n') {
        now.push_str(eol);
    }
    now.push_str(&format!("- insert:{eol}"));
    now.push_str(&format!(
        "{}{}",
        row_at(command, &step.repeat(2), step, eol, bridge, name),
        eol
    ));
    now
}

fn replaced_empty(
    body: &str,
    command: &str,
    step: &str,
    eol: &str,
    bridge: &str,
    name: &str,
) -> Option<String> {
    let starts = line_starts(body);
    for j in 0..starts.len() {
        let line = line_of(body, &starts, j);
        if indent_of(line) == 0 && said(line) == "[]" {
            let to = starts.get(j + 1).copied().unwrap_or(body.len());
            let piece = format!(
                "- insert:{eol}{}{}",
                row_at(command, &step.repeat(2), step, eol, bridge, name),
                eol
            );
            return Some(format!("{}{}{}", &body[..starts[j]], piece, &body[to..]));
        }
    }
    None
}

fn sequence(text: &str) -> bool {
    text.lines().all(|line| {
        let ind = indent_of(line);
        let said = said(line);
        said.is_empty() || ind > 0 || said.starts_with("- ")
    })
}

fn find_row(text: &str, bridge: &str, name: &str) -> Option<Row> {
    let starts = line_starts(text);
    for i in 0..starts.len() {
        let line = line_of(text, &starts, i);
        if indent_of(line) == 0
            && said(line) == "- insert:"
            && let Some(row) = row_in(text, &starts, i, bridge, name)
        {
            return Some(row);
        }
    }
    None
}

fn row_in(text: &str, starts: &[usize], entry: usize, bridge: &str, name: &str) -> Option<Row> {
    let step = step_of(text);
    let mut item_indent = 2 * step.len();
    for j in entry + 1..starts.len() {
        let line = line_of(text, starts, j);
        if said(line).is_empty() {
            continue;
        }
        if indent_of(line) == 0 {
            break;
        }
        if said(line).starts_with("- ") {
            item_indent = indent_of(line);
            break;
        }
    }
    let mut j = entry + 1;
    while j < starts.len() {
        let line = line_of(text, starts, j);
        if said(line).is_empty() {
            j += 1;
            continue;
        }
        if indent_of(line) == 0 {
            break;
        }
        if indent_of(line) == item_indent && said(line).starts_with("- ") {
            let end = row_end(text, starts, j + 1, item_indent);
            let row = Row {
                start: starts[j],
                end,
                entry: starts[entry],
                line: j,
            };
            if is_ours(text, &row, bridge, name) {
                return Some(row);
            }
            j = line_index(starts, end).unwrap_or(j + 1);
        } else {
            j += 1;
        }
    }
    None
}

fn row_end(text: &str, starts: &[usize], from: usize, item_indent: usize) -> usize {
    for j in from..starts.len() {
        let line = line_of(text, starts, j);
        if said(line).is_empty() {
            continue;
        }
        if indent_of(line) <= item_indent {
            return starts[j];
        }
    }
    text.len()
}

fn is_ours(text: &str, row: &Row, bridge: &str, name: &str) -> bool {
    let starts = line_starts(text);
    let first = said(line_of(text, &starts, row.line));
    let first = first.strip_prefix("- ").unwrap_or(first);
    if field(first, "id").and_then(scalar).as_deref() == Some(name) {
        return true;
    }
    let mut named = field(first, "name").and_then(scalar).as_deref() == Some(bridge);
    let mut served = false;
    for j in row.line + 1..starts.len() {
        if starts[j] >= row.end {
            break;
        }
        let said = said(line_of(text, &starts, j));
        if said.starts_with("- ") {
            continue;
        }
        if field(said, "id").and_then(scalar).as_deref() == Some(name) {
            return true;
        }
        if field(said, "name").and_then(scalar).as_deref() == Some(bridge) {
            named = true;
        }
        if field(said, "serverName").and_then(scalar).as_deref() == Some(name) {
            served = true;
        }
    }
    named && served
}

fn entry_bare(text: &str, starts: &[usize], entry: usize) -> bool {
    let Some(from) = line_index(starts, entry) else {
        return false;
    };
    for j in from + 1..starts.len() {
        let line = line_of(text, starts, j);
        if said(line).is_empty() {
            continue;
        }
        if indent_of(line) == 0 {
            break;
        }
        if said(line).starts_with("- ") {
            return false;
        }
    }
    true
}

fn bare(text: &str, bridge: &str, name: &str) -> String {
    match strip_ours(text, bridge, name).trim() {
        "" | "[]" => String::new(),
        rest => rest.to_string(),
    }
}

fn strip_ours(text: &str, bridge: &str, name: &str) -> String {
    let Some(row) = find_row(text, bridge, name) else {
        return without_empty(text);
    };
    let mut now = format!("{}{}", &text[..row.start], &text[row.end..]);
    let starts = line_starts(&now);
    if entry_bare(&now, &starts, row.entry) {
        let end = starts
            .iter()
            .find(|&&at| at > row.entry)
            .copied()
            .unwrap_or(now.len());
        now = format!("{}{}", &now[..row.entry], &now[end..]);
    }
    without_empty(&now)
}

fn without_empty(text: &str) -> String {
    let starts = line_starts(text);
    for j in 0..starts.len() {
        let line = line_of(text, &starts, j);
        if indent_of(line) == 0 && said(line) == "[]" {
            let to = starts.get(j + 1).copied().unwrap_or(text.len());
            return format!("{}{}", &text[..starts[j]], &text[to..]);
        }
    }
    text.to_string()
}

fn written(command: &str, step: &str, eol: &str, bridge: &str, name: &str) -> String {
    format!(
        "- insert:{eol}{}{}",
        row_at(command, &step.repeat(2), step, eol, bridge, name),
        eol
    )
}

fn row_at(command: &str, item: &str, step: &str, eol: &str, bridge: &str, name: &str) -> String {
    let fields = format!("{item}{step}");
    let config = format!("{fields}{step}");
    format!(
        "{item}- id: {name}{eol}{fields}name: {}{eol}{fields}config:{eol}{config}serverName: {name}{eol}{config}transport: stdio{eol}{config}command: {}{eol}{config}args: [mcp]",
        plain(bridge),
        plain(command)
    )
}

fn insert_after(text: &str, line: usize, piece: &str) -> String {
    let starts = line_starts(text);
    let at = starts.get(line + 1).copied().unwrap_or(text.len());
    let eol = eol_of(text);
    let lead = match at == text.len() && !text.ends_with('\n') {
        true => eol,
        false => "",
    };
    format!("{}{lead}{piece}{eol}{}", &text[..at], &text[at..])
}

fn field<'a>(said: &'a str, key: &str) -> Option<&'a str> {
    let rest = said.strip_prefix(key)?;
    Some(rest.strip_prefix(':')?.trim_start())
}

fn scalar(said: &str) -> Option<String> {
    if let Some(inner) = said.strip_prefix('\'').and_then(|it| it.strip_suffix('\'')) {
        return Some(inner.replace("''", "'"));
    }
    if let Some(inner) = said.strip_prefix('"').and_then(|it| it.strip_suffix('"')) {
        return Some(inner.replace("\\\"", "\"").replace("\\\\", "\\"));
    }
    (!said.is_empty()).then(|| said.to_string())
}

fn plain(said: &str) -> String {
    let needs = said.is_empty()
        || said.trim() != said
        || said.contains(": ")
        || said.contains([
            '\'', '"', '#', '%', '@', '`', '&', '*', '!', '|', '>', '{', '}', '[', ']', ',',
        ])
        || reserved(said);
    match needs {
        true => format!("'{}'", said.replace('\'', "''")),
        false => said.to_string(),
    }
}

fn reserved(said: &str) -> bool {
    let said = said.to_ascii_lowercase();
    matches!(
        said.as_str(),
        "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off"
    ) || said.parse::<f64>().is_ok()
}

fn value_span(line: &str, key: &str) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let cut = comment_cut(line, start);
    let said = &line[start..cut];
    let rest = said.strip_prefix(key)?;
    rest.strip_prefix(':')?;
    let mut from = start + key.len() + 1;
    while bytes.get(from).is_some_and(u8::is_ascii_whitespace) {
        from += 1;
    }
    let to = from + line[from..cut].trim_end().len();
    (from <= cut).then_some((from, to.max(from)))
}

fn said(line: &str) -> &str {
    let bytes = line.as_bytes();
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let cut = comment_cut(line, start);
    line[start..cut].trim_end()
}

fn comment_cut(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'#' if !single && !double && (i == start || bytes[i - 1].is_ascii_whitespace()) => {
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut all = vec![0];
    for (i, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            all.push(i + 1);
        }
    }
    all
}

fn line_of<'a>(text: &'a str, starts: &[usize], j: usize) -> &'a str {
    let end = starts.get(j + 1).copied().unwrap_or(text.len());
    &text[starts[j]..end]
}

fn line_index(starts: &[usize], at: usize) -> Option<usize> {
    starts.iter().position(|&one| one == at)
}

fn indent_of(line: &str) -> usize {
    line.as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()
}

fn leading(line: &str) -> &str {
    &line[..indent_of(line)]
}

fn step_of(text: &str) -> &'static str {
    match text
        .lines()
        .any(|line| line.starts_with('\t') && line.trim_start().starts_with('-'))
    {
        true => "\t",
        false => "  ",
    }
}

fn eol_of(text: &str) -> &'static str {
    match text.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    }
}

fn body_of(text: &str) -> (usize, &str) {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    (text.len() - body.len(), body)
}

#[cfg(test)]
#[path = "patch_test.rs"]
mod tests;
