const CORNERS: [char; 6] = ['┌', '├', '└', '╔', '╠', '╚'];
const WALLS: [char; 2] = ['│', '║'];

// Plain text from a terminal or a code editor carries its shape in characters, not in markup.
pub fn from_plain(text: &str) -> Option<String> {
    let text = text.replace("\r\n", "\n");
    if text
        .lines()
        .any(|one| one.trim_start().starts_with(CORNERS))
    {
        return boxed(&text);
    }
    coded(&text).then(|| format!("```\n{}\n```", text.trim_matches('\n')))
}

fn boxed(text: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut open: Option<Vec<String>> = None;
    let mut found = false;
    let flush = |rows: &mut Vec<Vec<String>>, out: &mut Vec<String>| {
        if rows.is_empty() {
            return;
        }
        let wide = rows.iter().map(Vec::len).max().unwrap_or(0);
        for (at, row) in rows.iter().enumerate() {
            let mut cells = row.clone();
            cells.resize(wide, String::new());
            out.push(format!("| {} |", cells.join(" | ")));
            if at == 0 {
                out.push(format!("|{}", " --- |".repeat(wide)));
            }
        }
        rows.clear();
    };
    for line in text.lines() {
        let flat = line.trim();
        if flat.starts_with(CORNERS) {
            found = true;
            if let Some(row) = open.take() {
                rows.push(row);
            }
            if flat.starts_with(['└', '╚']) {
                flush(&mut rows, &mut out);
            }
            continue;
        }
        if flat.starts_with(WALLS) {
            let cells: Vec<String> = flat
                .trim_matches(WALLS.as_slice())
                .split(WALLS.as_slice())
                .map(|one| one.trim().replace('|', "\\|"))
                .collect();
            open = Some(match open.take() {
                None => cells,
                Some(held) => held
                    .into_iter()
                    .zip(cells.into_iter().chain(std::iter::repeat(String::new())))
                    .map(|(was, more)| match (was.is_empty(), more.is_empty()) {
                        (_, true) => was,
                        (true, false) => more,
                        (false, false) => format!("{was} {more}"),
                    })
                    .collect(),
            });
            continue;
        }
        if let Some(row) = open.take() {
            rows.push(row);
        }
        flush(&mut rows, &mut out);
        out.push(line.to_string());
    }
    if let Some(row) = open.take() {
        rows.push(row);
    }
    flush(&mut rows, &mut out);
    found.then(|| out.join("\n"))
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

// Several lines most of which end like statements or open with a keyword read as code, not prose.
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
    let signs = lines
        .iter()
        .filter(|one| {
            let flat = one.trim();
            flat.ends_with(['{', '}', ';', ')', ',', '('])
                || flat == "}"
                || WORDS
                    .iter()
                    .any(|word| flat.starts_with(word) || flat.contains(&format!(" {word}")))
        })
        .count();
    signs * 5 >= lines.len() * 3
}
