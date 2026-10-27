use crate::i18n::Lang;

const STOPS: &str = "`\"')]>,;»";

/// Spaces a folder name may hold ("OneDrive - Personal") before it reads as prose.
const MOST_SPACES: usize = 2;

/// More room when a closing quote or a file name already says where the path ends.
const HELD_SPACES: usize = 4;

/// What sits in a fence or between backticks is an example someone wrote, not their disk.
pub(super) fn unpathed_body(body: &str) -> String {
    fenced_lines(body)
        .map(|(line, fenced)| {
            if fenced {
                line.to_string()
            } else {
                unpathed(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether a line quoted out of `body`, clipped or trimmed, sits inside one of its fences.
pub(super) fn fenced_in(body: &str, line: &str) -> bool {
    let quoted = line.trim_end_matches('…').trim();
    !quoted.is_empty()
        && fenced_lines(body).any(|(one, fenced)| fenced && one.trim().starts_with(quoted))
}

/// Each line with whether it belongs to a fence, the fence lines themselves included.
fn fenced_lines(body: &str) -> impl Iterator<Item = (&str, bool)> {
    let mut open: Option<(char, usize)> = None;
    body.lines().map(move |line| {
        let fence = fence_of(line);
        match (open, fence) {
            (Some((mark, opened)), Some((this, run, bare)))
                if this == mark && run >= opened && bare =>
            {
                open = None;
            }
            (Some(_), _) => {}
            (None, Some((mark, run, _))) => open = Some((mark, run)),
            (None, None) => return (line, false),
        }
        (line, true)
    })
}

pub(super) fn unpathed(line: &str) -> String {
    if let Some(said) = provenance(line) {
        return format!("{said}[path]");
    }
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some((start, end)) = code_span(rest) {
        out.push_str(&unpathed_prose(&rest[..start]));
        out.push_str(&rest[start..end]);
        rest = &rest[end..];
    }
    out.push_str(&unpathed_prose(rest));
    out
}

/// The line attaching writes ends in the path it copied from, so all after its words goes.
fn provenance(line: &str) -> Option<&'static str> {
    Lang::every()
        .filter_map(|lang| lang.get("attached-from").strip_suffix("{path}"))
        .find(|said| line.strip_prefix(said).is_some_and(opens_a_path))
}

fn opens_a_path(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    let drive = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');
    drive || rest.starts_with('/') || rest.starts_with(r"\\")
}

fn unpathed_prose(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = absolute(rest) {
        out.push_str(&rest[..at]);
        out.push_str("[path]");
        let path = &rest[at..];
        rest = &path[path_end(path, rest[..at].chars().next_back())..];
    }
    out.push_str(rest);
    out
}

/// The fence's mark, its length, and whether nothing follows it, which only a closing one needs.
fn fence_of(line: &str) -> Option<(char, usize, bool)> {
    let lead = line.len() - line.trim_start_matches(' ').len();
    let mark = line[lead..]
        .chars()
        .next()
        .filter(|c| matches!(c, '`' | '~'))?;
    let run = line[lead..].chars().take_while(|&c| c == mark).count();
    let bare = line[lead + run..].trim().is_empty();
    (lead <= 3 && run >= 3).then_some((mark, run, bare))
}

/// An unmatched run of backticks is a plain character, as in Markdown.
fn code_span(line: &str) -> Option<(usize, usize)> {
    let ticks_at = |at: usize| line[at..].bytes().take_while(|&b| b == b'`').count();
    let mut from = 0;
    while let Some(start) = line[from..].find('`').map(|at| from + at) {
        let open = ticks_at(start);
        let mut seek = start + open;
        while let Some(at) = line[seek..].find('`').map(|at| seek + at) {
            let run = ticks_at(at);
            if run == open {
                return Some((start, at + run));
            }
            seek = at + run;
        }
        from = start + open;
    }
    None
}

fn path_end(path: &str, opened: Option<char>) -> usize {
    let closer = match opened {
        Some(quote @ ('"' | '\'')) => Some(quote),
        Some('«') => Some('»'),
        Some('(') => Some(')'),
        _ => None,
    };
    let closed = closer.and_then(|closer| match closer {
        ')' => Some(stop_in(path, 0, Some(')'))).filter(|&at| at < path.len()),
        _ => path.find(closer),
    });
    if let Some(at) = closed
        && spaces(&path[..at]) <= HELD_SPACES
    {
        return at;
    }
    let sep = match path.as_bytes().get(2) {
        Some(b'\\') => '\\',
        _ => '/',
    };
    let reach = match sep {
        '/' => path.find("://").unwrap_or(path.len()),
        _ => path.len(),
    };
    let named = named_file_end(&path[..reach]);
    let room = if named.is_some() {
        HELD_SPACES
    } else {
        MOST_SPACES
    };
    let text = &path[..named.unwrap_or(reach)];
    let mut end = stop_in(text, 0, None);
    while let Some(next) = spaced_on(text, end, sep, room) {
        end = stop_in(text, next, None);
    }
    end
}

/// Where a word stops, or with `sep` where the next separator is; `(x86)` stays inside.
fn stop_in(text: &str, from: usize, sep: Option<char>) -> usize {
    let mut open = 0usize;
    text[from..]
        .char_indices()
        .find(|&(_, c)| match c {
            '(' => {
                open += 1;
                false
            }
            ')' if open > 0 => {
                open -= 1;
                false
            }
            _ if Some(c) == sep => true,
            _ => (sep.is_none() && c.is_whitespace()) || STOPS.contains(c),
        })
        .map_or(text.len(), |(at, _)| from + at)
}

/// The next separator past a space, when the words before it are few enough for one folder.
fn spaced_on(text: &str, from: usize, sep: char, room: usize) -> Option<usize> {
    let rest = &text[from..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let at = stop_in(rest, 0, Some(sep));
    let stretch = &rest[..at];
    let after = &rest[at..];
    let joins = after.starts_with(sep)
        && !stretch.ends_with(char::is_whitespace)
        && after[sep.len_utf8()..].starts_with(|c: char| !c.is_whitespace())
        && absolute(&rest[..at + sep.len_utf8()]).is_none();
    (joins && spaces(stretch) <= room).then_some(from + at)
}

fn spaces(text: &str) -> usize {
    text.chars().filter(|c| c.is_whitespace()).count()
}

pub(super) fn absolute(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len()).find(|&at| {
        // A drive is one letter: without this, the "s" of "https://" reads as one and the rest
        // of the line goes with it.
        let alone = at == 0 || !bytes[at - 1].is_ascii_alphanumeric();
        let drive = alone
            && at + 2 < bytes.len()
            && bytes[at].is_ascii_alphabetic()
            && bytes[at + 1] == b':'
            && (bytes[at + 2] == b'/' || bytes[at + 2] == b'\\');
        // A second slash in the same word: `/code-review` or `RMDir /r` are words, not folders.
        let rooted = bytes[at] == b'/'
            && line[..at]
                .chars()
                .next_back()
                .is_some_and(|before| matches!(before, ' ' | '"' | '\'' | '(' | '«'))
            && bytes
                .get(at + 1)
                .is_some_and(|next| !next.is_ascii_whitespace())
            && line[at + 1..]
                .split(char::is_whitespace)
                .next()
                .is_some_and(|word| word.contains('/'));
        drive || rooted
    })
}

fn named_file_end(path: &str) -> Option<usize> {
    let marks: Vec<(usize, char)> = path.char_indices().collect();
    for (at, &(dot, c)) in marks.iter().enumerate() {
        if c != '.' || at == 0 || marks[at - 1].1.is_whitespace() {
            continue;
        }
        let ext = marks[at + 1..]
            .iter()
            .take_while(|(_, one)| one.is_ascii_alphanumeric())
            .count();
        if !(1..=8).contains(&ext) {
            continue;
        }
        let end = at + 1 + ext;
        let closed = marks
            .get(end)
            .is_none_or(|&(_, next)| next.is_whitespace() || STOPS.contains(next));
        if closed && path[..dot].contains(['/', '\\']) {
            return Some(marks.get(end).map_or(path.len(), |&(at, _)| at));
        }
    }
    None
}
