const TOPS: [char; 3] = ['┌', '╔', '┏'];
const SEAMS: [char; 6] = ['├', '╠', '┣', '╞', '╟', '┠'];
const BOTTOMS: [char; 3] = ['└', '╚', '┗'];
const WALLS: [char; 3] = ['│', '║', '┃'];

pub fn from_plain(text: &str) -> Option<String> {
    let text = even(text);
    // `tree` draws with the same corners but never opens on a top one, which every table does.
    if text.lines().any(|one| ruler(one.trim())) {
        return boxed(&text);
    }
    if shelled(&text) {
        return Some(fenced(text.trim_matches('\n')));
    }
    if let Some(table) = super::separated::tabbed_table(&text) {
        return Some(table);
    }
    // Lines of code often hold the same commas, and code is what they are.
    if coded(&text) {
        return Some(fenced(text.trim_matches('\n')));
    }
    super::separated::comma_table(&text)
}

fn boxed(text: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut found = false;
    for line in text.lines() {
        let flat = line.trim();
        let open = !held.is_empty();
        if ruler(flat) || (open && (flat.starts_with(SEAMS) || flat.starts_with(WALLS))) {
            held.push(flat);
            continue;
        }
        if open && flat.starts_with(BOTTOMS) {
            out.extend(table(&held));
            held.clear();
            found = true;
            continue;
        }
        if open {
            out.extend(table(&held));
            held.clear();
            found = true;
        }
        out.push(line.to_string());
    }
    if !held.is_empty() {
        out.extend(table(&held));
        found = true;
    }
    found.then(|| out.join("\n"))
}

// A table that rules every row merges the lines between rules; one ruled only under its head has a row per line.
fn table(lines: &[&str]) -> Vec<String> {
    let seams = lines.iter().filter(|one| one.starts_with(SEAMS)).count();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut open: Option<Vec<String>> = None;
    let mut ruled = 0;
    for line in lines {
        if !line.starts_with(WALLS) {
            ruled += usize::from(line.starts_with(SEAMS));
            if let Some(row) = open.take() {
                rows.push(row);
            }
            continue;
        }
        let cells = cells(line);
        if seams == 0 || (seams < 2 && ruled > 0) {
            rows.push(cells);
            continue;
        }
        open = Some(match open.take() {
            None => cells,
            Some(was) => joined(was, cells),
        });
    }
    if let Some(row) = open.take() {
        rows.push(row);
    }
    let wide = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = Vec::new();
    for (at, mut row) in rows.into_iter().enumerate() {
        row.resize(wide, String::new());
        out.push(format!("| {} |", row.join(" | ")));
        if at == 0 {
            out.push(format!("|{}", " --- |".repeat(wide)));
        }
    }
    out
}

// A table opens on one unbroken rule; boxes drawn side by side for a diagram leave gaps between them.
fn ruler(flat: &str) -> bool {
    flat.starts_with(TOPS)
        && flat.chars().all(|one| {
            matches!(one, '─' | '━' | '═' | '┬' | '┳' | '╦' | '┐' | '┓' | '╗')
                || TOPS.contains(&one)
        })
}

fn cells(line: &str) -> Vec<String> {
    line.trim_matches(WALLS.as_slice())
        .split(WALLS.as_slice())
        .map(|one| one.trim().replace('|', "\\|"))
        .collect()
}

fn joined(was: Vec<String>, more: Vec<String>) -> Vec<String> {
    let wide = was.len().max(more.len());
    let mut was = was;
    let mut more = more;
    was.resize(wide, String::new());
    more.resize(wide, String::new());
    was.into_iter()
        .zip(more)
        .map(|(one, next)| match (one.is_empty(), next.is_empty()) {
            (_, true) => one,
            (true, false) => next,
            (false, false) => format!("{one} {next}"),
        })
        .collect()
}

const WORDS: [&str; 16] = [
    "fn ",
    "let ",
    "use ",
    "def ",
    "const ",
    "import ",
    "function ",
    "class ",
    "return ",
    "pub ",
    "#include",
    "if (",
    "for (",
    "} else",
    "=> ",
    "-> ",
];

/// Windows ends a line with `\r\n` and old Macs with `\r`; the language ends it with `\n`.
pub(super) fn even(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// What a terminal shows: the dashes PowerShell rules under each column, or a session that opens on a prompt or holds several.
pub(super) fn shelled(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|one| !one.trim().is_empty()).collect();
    // A README or a how-to that quotes one command is still prose; Markdown around it says so.
    if lines.len() < 2 || super::pasted::markdowned(text, false) {
        return false;
    }
    let prompts = lines
        .iter()
        .filter(|one| prompted(one.trim_start()))
        .count();
    lines.iter().any(|one| columns_ruled(one.trim()))
        || prompts >= 2
        || lines.first().is_some_and(|one| prompted(one.trim_start()))
}

fn prompted(flat: &str) -> bool {
    let drive = |said: &str| {
        let bytes = said.as_bytes();
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && &bytes[1..3] == b":\\"
    };
    let powershell = flat
        .strip_prefix("PS ")
        .is_some_and(|rest| (drive(rest) || rest.starts_with(r"\\")) && rest.contains('>'));
    // `C:\fotos -> respaldo` points with an arrow; cmd's prompt ends the path itself on `>`.
    let cmd = drive(flat)
        && flat
            .find('>')
            .is_some_and(|at| !flat[..at].ends_with(['-', '=', ' ']));
    let unix = flat.find(['$', '#']).is_some_and(|at| {
        let before = &flat[..at];
        before.contains('@')
            && before.contains(':')
            && !before.contains(' ')
            && flat[at + 1..].starts_with(' ')
    });
    let dollar = flat
        .strip_prefix("$ ")
        .is_some_and(|rest| rest.starts_with(|one: char| one.is_ascii_lowercase()));
    powershell || cmd || unix || dollar
}

fn columns_ruled(flat: &str) -> bool {
    let runs: Vec<&str> = flat.split_whitespace().collect();
    runs.len() >= 2
        && runs
            .iter()
            .all(|one| one.len() >= 2 && one.chars().all(|dash| dash == '-'))
}

/// A line only code writes: a keyword beside code's punctuation, or a brace that opens or shuts a block.
pub(super) fn code_line(flat: &str) -> bool {
    keyed(flat) || braced(flat)
}

// «let me know» opens a sentence too; a keyword counts only beside the punctuation code is made of.
fn keyed(flat: &str) -> bool {
    WORDS
        .iter()
        .any(|word| flat.starts_with(word) || flat.contains(&format!(" {word}")))
        && flat.contains(['=', '(', '{', ';', ':', '<'])
}

fn braced(flat: &str) -> bool {
    flat.ends_with('{') || flat == "}" || flat == "};"
}

// Commas and semicolons end prose lines too; only a keyword or a brace says the lines are code.
fn coded(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|one| !one.trim().is_empty()).collect();
    if lines.len() < 3 {
        return false;
    }
    if lines.iter().any(|one| {
        let flat = one.trim_start();
        flat.starts_with("```") || flat.starts_with("- ") || flat.starts_with("> ")
    }) {
        return false;
    }
    let strong = lines
        .iter()
        .any(|one| keyed(one.trim()) || braced(one.trim()));
    let signs = lines
        .iter()
        .filter(|one| {
            let flat = one.trim();
            braced(flat) || keyed(flat) || flat.ends_with([';', ')', ',', '('])
        })
        .count();
    strong && signs * 5 >= lines.len() * 3
}

// The fence outgrows any run of backticks inside, so code that quotes a fence never closes its own.
pub(super) fn fenced(code: &str) -> String {
    let longest = code
        .split(|one| one != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let wall = "`".repeat(longest.max(2) + 1);
    format!("{wall}{}\n{code}\n{wall}", tongue(code))
}

// Only signs no prose carries name a language; anything less leaves the block plain.
fn tongue(code: &str) -> &'static str {
    let has = |one: &str| code.contains(one);
    let starts = |one: &str| code.lines().any(|line| line.trim_start().starts_with(one));
    let scripted = has("=>") || starts("const ") || starts("function ") || starts("export ");
    if (starts("fn ") && (has("let ") || has("->") || has("::"))) || (starts("use ") && has("::")) {
        "rust"
    } else if starts("def ")
        || ((starts("import ") || starts("from "))
            && code.lines().any(|line| line.trim_end().ends_with(':')))
    {
        "python"
    } else if scripted && (has(": string") || has(": number") || has("interface ")) {
        "typescript"
    } else if scripted {
        "javascript"
    } else if code.trim_start().starts_with(['{', '[']) && has("\":") {
        "json"
    } else if ["$ ", "git ", "npm ", "cargo ", "cd "]
        .iter()
        .any(|one| starts(one))
    {
        "sh"
    } else {
        ""
    }
}
