use std::iter::Peekable;
use std::str::Chars;

const SAYING: [&str; 4] = ["cat", "echo", "printf", "write-output"];
const WRITING: [&str; 7] = [
    "out-file",
    "set-content",
    "add-content",
    "new-item",
    "sc",
    "ac",
    "ni",
];
const NAMING: [&str; 4] = ["-path", "-filepath", "-literalpath", "-pspath"];
const MOVING: [&str; 6] = ["cd", "pushd", "chdir", "set-location", "sl", "popd"];
const HERE_STRING: &str = "@";

#[derive(Debug, PartialEq, Eq)]
pub struct Written {
    pub path: String,
    /// The command moved somewhere first, so a path that depends on where it runs cannot be placed.
    pub moved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Word(String),
    Into,
    Elsewhere,
    End,
}

/// Where a terminal command writes what it says itself: never what a program it runs would write.
pub fn written(command: &str) -> Vec<Written> {
    let pieces = pieces(&without_what_is_written(command));
    let mut all = Vec::new();
    let mut moved = false;
    for one in pieces.split(|piece| *piece == Piece::End) {
        let (words, into) = apart(one);
        let Some(head) = words.first().map(|it| program(it)) else {
            continue;
        };
        moved |= MOVING.contains(&head.as_str());
        let mut named: Vec<&str> = Vec::new();
        if SAYING.contains(&head.as_str()) || head == HERE_STRING {
            named.extend(into);
        }
        if head == "tee" || head == "tee-object" {
            named.extend(words[1..].iter().filter(|it| !it.starts_with('-')));
        }
        if WRITING.contains(&head.as_str()) {
            named.extend(target(&words[1..]));
        }
        all.extend(named.into_iter().map(|path| Written {
            path: path.to_string(),
            moved,
        }));
    }
    all
}

fn apart(one: &[Piece]) -> (Vec<&str>, Vec<&str>) {
    let mut words = Vec::new();
    let mut into = Vec::new();
    let mut redirect: Option<&Piece> = None;
    for piece in one {
        match (piece, redirect.take()) {
            (Piece::Word(word), Some(Piece::Into)) => into.push(word.as_str()),
            (Piece::Word(_), Some(_)) => {}
            (Piece::Word(word), None) => words.push(word.as_str()),
            (next, _) => redirect = Some(next),
        }
    }
    (words, into)
}

fn program(word: &str) -> String {
    let name = word.rsplit(['/', '\\']).next().unwrap_or(word);
    name.trim_end_matches(".exe").to_lowercase()
}

fn target<'a>(words: &[&'a str]) -> Option<&'a str> {
    let named = words
        .iter()
        .position(|it| NAMING.contains(&it.to_lowercase().as_str()));
    match named {
        Some(at) => words.get(at + 1).copied(),
        None => {
            let after_value = |at: usize| at > 0 && words[at - 1].eq_ignore_ascii_case("-value");
            let mut plain = words.iter().enumerate();
            plain
                .find(|(at, it)| !it.starts_with('-') && !after_value(*at))
                .map(|(_, it)| *it)
        }
    }
}

// A here-document and a here-string hold what is written, which is never read.
fn without_what_is_written(command: &str) -> String {
    let mut kept = String::new();
    let mut lines = command.lines();
    while let Some(line) = lines.next() {
        match here_string(line) {
            Some((before, shut)) => {
                let after = lines.by_ref().find_map(|it| it.strip_prefix(shut));
                kept.push_str(before);
                kept.push_str(HERE_STRING);
                kept.push_str(after.unwrap_or_default());
            }
            None => {
                kept.push_str(line);
                for word in here_documents(line) {
                    let _ = lines.by_ref().find(|it| it.trim() == word);
                }
            }
        }
        kept.push('\n');
    }
    kept
}

fn here_string(line: &str) -> Option<(&str, &'static str)> {
    let line = line.trim_end();
    let single = line.strip_suffix("@'").map(|before| (before, "'@"));
    single.or_else(|| line.strip_suffix("@\"").map(|before| (before, "\"@")))
}

fn here_documents(line: &str) -> Vec<String> {
    let mut all = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("<<") {
        rest = &rest[at + 2..];
        if let Some(past) = rest.strip_prefix('<') {
            rest = past;
            continue;
        }
        let word: String = rest
            .trim_start_matches('-')
            .trim_start()
            .trim_start_matches(['\'', '"', '\\'])
            .chars()
            .take_while(|it| it.is_alphanumeric() || *it == '_')
            .collect();
        if !word.is_empty() {
            all.push(word);
        }
    }
    all
}

fn pieces(said: &str) -> Vec<Piece> {
    let mut all = Vec::new();
    let mut word: Option<String> = None;
    let mut left = said.chars().peekable();
    while let Some(one) = left.next() {
        match one {
            '\'' | '"' => quoted(&mut left, one, word.get_or_insert_default()),
            '$' => substituted(&mut left, word.get_or_insert_default()),
            '\\' | '`' if matches!(left.peek(), Some('\n' | ' ')) => {
                if left.next() == Some(' ') {
                    word.get_or_insert_default().push(' ');
                }
            }
            '#' if word.is_none() => while left.next_if(|it| *it != '\n').is_some() {},
            '&' if left.peek() == Some(&'>') => finish(&mut all, &mut word),
            '\n' | ';' | '|' | '&' | '(' | ')' | '{' | '}' => {
                finish(&mut all, &mut word);
                all.push(Piece::End);
            }
            '>' => {
                let stream = word.take_if(|it| it.chars().all(|c| c.is_ascii_digit()));
                finish(&mut all, &mut word);
                while left.next_if(|it| matches!(it, '>' | '|')).is_some() {}
                let copied = left.next_if_eq(&'&').is_some();
                let said = stream.is_none_or(|it| it == "1");
                all.push(match said && !copied {
                    true => Piece::Into,
                    false => Piece::Elsewhere,
                });
            }
            '<' => {
                finish(&mut all, &mut word);
                while left.next_if(|it| matches!(it, '<' | '-')).is_some() {}
                all.push(Piece::Elsewhere);
            }
            one if one.is_whitespace() => finish(&mut all, &mut word),
            one => word.get_or_insert_default().push(one),
        }
    }
    finish(&mut all, &mut word);
    all
}

fn finish(all: &mut Vec<Piece>, word: &mut Option<String>) {
    all.extend(word.take().map(Piece::Word));
}

fn quoted(left: &mut Peekable<Chars>, quote: char, word: &mut String) {
    while let Some(one) = left.next() {
        match one {
            '\\' if quote == '"' && left.peek() == Some(&'"') => {
                left.next();
                word.push('"');
            }
            one if one == quote => return,
            one => word.push(one),
        }
    }
}

fn substituted(left: &mut Peekable<Chars>, word: &mut String) {
    word.push('$');
    let (open, shut) = match left.peek() {
        Some('(') => ('(', ')'),
        Some('{') => ('{', '}'),
        _ => return,
    };
    let mut deep = 0;
    for one in left.by_ref() {
        word.push(one);
        deep += i32::from(one == open) - i32::from(one == shut);
        if deep == 0 {
            return;
        }
    }
}

#[cfg(test)]
#[path = "command_test.rs"]
mod tests;
