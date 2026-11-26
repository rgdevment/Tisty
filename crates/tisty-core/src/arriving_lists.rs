use std::borrow::Cow;

struct Item {
    level: usize,
    number: Option<u32>,
    body: String,
}

/// What Word drew before a paragraph: a bullet, a plain number, or words a list cannot say.
enum Marker {
    Bullet,
    Number(u32),
    Words(String),
}

/// Word writes a list as paragraphs that each draw their own bullet; the list they were is said again.
pub(super) fn listed(html: &str) -> Cow<'_, str> {
    let low = html.to_ascii_lowercase();
    if !low.contains("mso-list:l") {
        return Cow::Borrowed(html);
    }
    let mut out = String::with_capacity(html.len());
    let mut run: Vec<Item> = Vec::new();
    let mut base = 0;
    let mut copied = 0;
    let mut seek = 0;
    while let Some(found) = paragraph(&low, seek) {
        let Some(opened) = low[found..].find('>').map(|at| found + at + 1) else {
            break;
        };
        let Some(level) = level_of(&low[found..opened]) else {
            seek = opened;
            continue;
        };
        let Some(shut) = low[opened..].find("</p>").map(|at| opened + at) else {
            break;
        };
        let between = &html[copied..found];
        let (marker, body) = unmarked(&html[opened..shut]);
        let number = match marker_of(&marker) {
            Marker::Bullet => None,
            Marker::Number(one) => Some(one),
            Marker::Words(said) => {
                if !run.is_empty() {
                    out.push_str(&nested(&run));
                    run.clear();
                }
                out.push_str(between);
                out.push_str(&format!("<p>{said} {body}</p>"));
                copied = shut + "</p>".len();
                seek = copied;
                continue;
            }
        };
        if !run.is_empty() && !quiet(between) {
            out.push_str(&nested(&run));
            run.clear();
        }
        if run.is_empty() {
            out.push_str(between);
            base = level - 1;
        }
        // A list picked up below its first level, as after a paragraph, starts at the top rather than under empty items.
        run.push(Item {
            level: level.saturating_sub(base).max(1),
            number,
            body,
        });
        copied = shut + "</p>".len();
        seek = copied;
    }
    if !run.is_empty() {
        out.push_str(&nested(&run));
    }
    out.push_str(&html[copied..]);
    Cow::Owned(out)
}

fn paragraph(low: &str, from: usize) -> Option<usize> {
    let mut at = from;
    loop {
        let found = at + low[at..].find("<p")?;
        let next = low.as_bytes().get(found + 2);
        if next.is_some_and(|one| *one == b'>' || one.is_ascii_whitespace()) {
            return Some(found);
        }
        at = found + 2;
    }
}

// `mso-list:l0 level2 lfo1` puts a paragraph second deep in list 0; `mso-list:Ignore` marks the bullet itself.
fn level_of(tag: &str) -> Option<usize> {
    let rest = &tag[tag.find("mso-list:")? + "mso-list:".len()..];
    let rest = rest.trim_start();
    if !rest.starts_with('l') || rest.starts_with("lfo") {
        return None;
    }
    let rest = &rest[..rest.find([';', '\'', '"', '>']).unwrap_or(rest.len())];
    let after = &rest[rest.find("level")? + "level".len()..];
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    digits.parse::<usize>().ok().map(|one| one.clamp(1, 9))
}

fn quiet(between: &str) -> bool {
    let mut rest = between;
    while let Some(open) = rest.find("<!") {
        if !rest[..open].trim().is_empty() {
            return false;
        }
        match rest[open..].find('>') {
            Some(shut) => rest = &rest[open + shut + 1..],
            None => return false,
        }
    }
    rest.trim().is_empty()
}

fn unmarked(inner: &str) -> (String, String) {
    let low = inner.to_ascii_lowercase();
    let mut marker = String::new();
    let mut body = inner.to_string();
    if let Some(ignored) = low.find("mso-list:ignore")
        && let Some(start) = low[..ignored].rfind("<span")
        && let Some(opened) = low[start..].find('>').map(|at| start + at + 1)
        && let Some(ends) = super::html::closing(&low[start..opened], &low[opened..])
    {
        marker = bare(&inner[opened..opened + ends]);
        body = format!("{}{}", &inner[..start], &inner[opened + ends..]);
    }
    for said in ["<![if !supportLists]>", "<![endif]>"] {
        body = body.replace(said, "");
    }
    (marker, body)
}

fn bare(fragment: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for one in fragment.chars() {
        match one {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(one),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace('\u{a0}', " ")
        .trim()
        .to_string()
}

// Markdown counts only `1.` or `1)`; a bullet is a symbol, or Word's `o` set in a symbol font. `1.1.`, `a)` or `Artículo 2.` stay words.
fn marker_of(marker: &str) -> Marker {
    let said = marker.trim();
    let digits = said.trim_end_matches(['.', ')']);
    if said.len() == digits.len() + 1
        && !digits.is_empty()
        && digits.len() <= 9
        && digits.chars().all(|one| one.is_ascii_digit())
    {
        return digits
            .parse()
            .map_or(Marker::Words(said.to_string()), Marker::Number);
    }
    let symbol = said.chars().count() == 1 && !said.chars().all(char::is_alphanumeric);
    match symbol || said == "o" || said.is_empty() {
        true => Marker::Bullet,
        false => Marker::Words(said.to_string()),
    }
}

// A numbered list opens on the number Word gave its first item, so one cut by a paragraph goes on counting.
fn opened(item: &Item) -> String {
    match item.number {
        Some(start) if start != 1 => format!("<ol start=\"{start}\">"),
        Some(_) => "<ol>".to_string(),
        None => "<ul>".to_string(),
    }
}

// Every list kept open holds one item still open, so a deeper list always lands inside an item.
fn nested(run: &[Item]) -> String {
    let mut out = String::new();
    let mut open: Vec<&str> = Vec::new();
    for (at, item) in run.iter().enumerate() {
        let kind = if item.number.is_some() { "ol" } else { "ul" };
        while open.len() > item.level {
            out.push_str(&format!("</li></{}>", open.pop().unwrap_or("ul")));
        }
        if open.len() == item.level {
            match open.last() {
                Some(was) if *was != kind => {
                    out.push_str(&format!("</li></{was}>{}", opened(item)));
                    open.pop();
                    open.push(kind);
                }
                _ => out.push_str("</li>"),
            }
        }
        while open.len() < item.level {
            let last = open.len() + 1 == item.level;
            out.push_str(&if last {
                opened(item)
            } else {
                format!("<{kind}>")
            });
            open.push(kind);
            if open.len() < item.level {
                out.push_str("<li>");
            }
        }
        out.push_str("<li>");
        // Only a list counting from 1 may follow an item's words on the next line; a later count needs its own paragraph.
        let counted_on = run
            .get(at + 1)
            .is_some_and(|next| next.level > item.level && next.number.is_some_and(|one| one != 1));
        match counted_on {
            true => out.push_str(&format!("<p>{}</p>", item.body)),
            false => out.push_str(&item.body),
        }
    }
    while let Some(kind) = open.pop() {
        out.push_str(&format!("</li></{kind}>"));
    }
    out
}
