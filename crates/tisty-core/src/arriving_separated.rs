// Fewer rows than this are as likely a sentence with commas in it as a table.
const CSV_ROWS: usize = 3;
const CSV_FIELDS: usize = 3;

/// Rows copied out of a spreadsheet as plain text, a tab between their cells, as a Markdown table.
pub fn tabbed_table(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.trim_matches('\n').lines().collect();
    tabbed(&lines).map(|rows| drawn(&rows))
}

/// Rows written as CSV, a comma between their cells and quotes around a cell that holds one.
pub fn comma_table(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.trim_matches('\n').lines().collect();
    commas(&lines).map(|rows| drawn(&rows))
}

// Lines that open on a tab are indented code, and a table's head never starts empty.
fn tabbed(lines: &[&str]) -> Option<Vec<Vec<String>>> {
    let first = lines.first()?;
    if first.starts_with('\t') {
        return None;
    }
    let tabs = first.matches('\t').count();
    let even =
        lines.len() >= 2 && tabs > 0 && lines.iter().all(|one| one.matches('\t').count() == tabs);
    even.then(|| {
        lines
            .iter()
            .map(|one| one.split('\t').map(str::to_string).collect())
            .collect()
    })
}

fn commas(lines: &[&str]) -> Option<Vec<Vec<String>>> {
    if lines.len() < CSV_ROWS {
        return None;
    }
    if lines.iter().any(|one| statement(one)) {
        return None;
    }
    let rows: Vec<Vec<String>> = lines.iter().map(|one| fields(one)).collect::<Option<_>>()?;
    let wide = rows.first()?.len();
    (wide >= CSV_FIELDS && rows.iter().all(|one| one.len() == wide)).then_some(rows)
}

// A call, `foo(a, b)`, or a line closed as code closes one, is code that happens to hold commas.
fn statement(line: &str) -> bool {
    let line = line.trim();
    let called = line.find('(').is_some_and(|at| {
        at > 0
            && line[..at]
                .chars()
                .all(|one| one.is_alphanumeric() || "_.:".contains(one))
    });
    called || line.ends_with([';', '{', '}'])
}

// A field in quotes may hold commas, and two quotes in a row are one; a quote left open is not a CSV line.
fn fields(line: &str) -> Option<Vec<String>> {
    let mut all = vec![String::new()];
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(one) = chars.next() {
        let field = all.last_mut()?;
        match one {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' if quoted || field.trim().is_empty() => quoted = !quoted,
            ',' if !quoted => all.push(String::new()),
            _ => field.push(one),
        }
    }
    (!quoted).then_some(all)
}

fn drawn(rows: &[Vec<String>]) -> String {
    let cell = |said: &str| said.trim().replace('|', "\\|");
    let mut out = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        let said: Vec<String> = row.iter().map(|one| cell(one)).collect();
        out.push(format!("| {} |", said.join(" | ")));
        if at == 0 {
            out.push(format!("|{}", " --- |".repeat(row.len())));
        }
    }
    out.join("\n")
}
