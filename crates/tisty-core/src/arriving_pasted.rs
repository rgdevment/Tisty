use htmd::node::{Node, NodeType};

/// What a clipboard's html turns out to be, by what it holds rather than what it claims.
pub(super) enum Arrived {
    /// Text a code editor or a terminal laid out by spaces and coloured by syntax, said as Markdown or as code.
    Laid(String),
    /// Bare words, Markdown among them, as a chat or a plain editor copies what was typed.
    Written(String),
    /// A page with formatting of its own, for the converter to say in Markdown.
    Formatted,
}

const BLOCKS: [&str; 24] = [
    "p",
    "div",
    "li",
    "ul",
    "ol",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "pre",
    "blockquote",
    "tr",
    "table",
    "section",
    "article",
    "header",
    "footer",
    "main",
    "nav",
    "aside",
    "dd",
    "dt",
];

// A paragraph ends with a blank line; a `div` is how editors and chats lay out one line.
const SPACED: [&str; 8] = ["p", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote"];

// A rule between messages divides them without dressing a word; the typed Markdown around it still reads as typed.
const FORMATTING: [&str; 27] = [
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "strong",
    "i",
    "em",
    "a",
    "ul",
    "ol",
    "li",
    "table",
    "img",
    "code",
    "pre",
    "blockquote",
    "mark",
    "u",
    "ins",
    "s",
    "del",
    "strike",
    "sup",
    "sub",
    "details",
];

pub(super) fn arrived(tree: &Node) -> Arrived {
    if let Some(laid) = laid_out(tree) {
        return Arrived::Laid(laid);
    }
    if formatted(tree) {
        return Arrived::Formatted;
    }
    let mut said = String::new();
    words(tree, false, &mut said);
    let said = knitted(&evened(&said));
    match markdowned(&said, false) {
        true => Arrived::Written(unmarked_beyond_markdown(&said)),
        false => Arrived::Formatted,
    }
}

// What a chat shows as typed stays as typed: Markdown's own marks still work, but a tag, an entity,
// a backslash, a sum in dollars or a mark only Tisty has is a letter here too.
fn unmarked_beyond_markdown(text: &str) -> String {
    let mut fence: Option<String> = None;
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let flat = line.trim_start();
        let mark = ["```", "~~~"].into_iter().find(|one| flat.starts_with(one));
        match (&fence, mark) {
            (Some(open), Some(mark)) if mark == open => fence = None,
            (None, Some(mark)) => fence = Some(mark.to_string()),
            _ => {}
        }
        if fence.is_some() || mark.is_some() {
            out.push(line.to_string());
            continue;
        }
        let kept: String = super::spans(line)
            .into_iter()
            .map(|(coded, part)| match coded {
                true => part.to_string(),
                false => lettered(part),
            })
            .collect();
        out.push(kept);
    }
    out.join("\n")
}

fn lettered(part: &str) -> String {
    let mut out = String::with_capacity(part.len());
    let chars: Vec<char> = part.chars().collect();
    for (at, one) in chars.iter().enumerate() {
        let next = chars.get(at + 1).copied();
        let before = at.checked_sub(1).and_then(|was| chars.get(was)).copied();
        let doubled = |sign: char| *one == sign && (next == Some(sign) || before == Some(sign));
        let shortcode = *one == ':'
            && (next == Some(':') || (before != Some(':') && emoji_ahead(&chars[at + 1..])));
        if matches!(one, '\\' | '<' | '&' | '$') || doubled('=') || doubled('+') || shortcode {
            out.push('\\');
        }
        out.push(*one);
    }
    out
}

// `:tada:` would turn into its emoji; a word between two colons with nothing else is the code for one.
fn emoji_ahead(rest: &[char]) -> bool {
    let word = rest
        .iter()
        .take_while(|one| one.is_ascii_alphanumeric() || matches!(one, '_' | '+' | '-'))
        .count();
    word > 0 && rest.get(word) == Some(&':')
}

// The element that holds every word on the clipboard and lays them out by spaces: VS Code's, a terminal's.
fn laid_out(tree: &Node) -> Option<String> {
    let holder = first(tree, &|node| {
        style(node).is_some_and(|said| {
            let flat = said
                .to_ascii_lowercase()
                .replace([' ', '\t', '\n', '\r'], "");
            flat.split(';').any(|one| one == "white-space:pre")
        })
    })?;
    let mut all = String::new();
    words(tree, false, &mut all);
    let mut held = String::new();
    words(&holder, true, &mut held);
    let squeezed = |one: &str| one.split_whitespace().collect::<String>();
    if squeezed(&all) != squeezed(&held) {
        return None;
    }
    Some(laid(&super::plain::even(&held)))
}

// Lines read as Markdown when they are, or else as the code or output they are; one line of code stays in its sentence.
fn laid(text: &str) -> String {
    let text = text.trim_matches('\n');
    if !text.contains('\n') {
        return spoken(text.trim());
    }
    let read = markdowned(text, true)
        && !super::plain::shelled(text)
        && !yamled(text)
        && !outside_fences(text).any(|one| super::plain::code_line(one.trim()));
    if read {
        return text.to_string();
    }
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    super::plain::fenced(&lines.join("\n"))
}

// YAML opens its items with the same dash a Markdown list does; its `key:` lines tell them apart.
fn yamled(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|one| !one.trim().is_empty()).collect();
    let keyed = lines
        .iter()
        .filter(|one| {
            let flat = one.trim_start();
            let flat = flat.strip_prefix("- ").unwrap_or(flat);
            flat.find(':').is_some_and(|at| {
                at > 0
                    && flat[..at]
                        .chars()
                        .all(|one| one.is_alphanumeric() || "_-.".contains(one))
                    && flat[at + 1..]
                        .chars()
                        .next()
                        .is_none_or(char::is_whitespace)
            })
        })
        .count();
    keyed * 2 > lines.len()
}

// `a * b * c`, `__init__`, `# TODO` or `- name: x` would turn into marks; a run of code keeps every sign as it was copied.
fn spoken(line: &str) -> String {
    let marked =
        line.contains(['*', '_', '`', '<', '[', '~', '=', '\\', '$']) || markdowned(line, true);
    if !marked {
        return line.to_string();
    }
    let longest = line
        .split(|one| one != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let ticks = "`".repeat(longest + 1);
    let pad = if line.starts_with('`') || line.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{line}{pad}{ticks}")
}

fn first<'a>(node: &Node<'a>, found: &dyn Fn(&Node) -> bool) -> Option<Node<'a>> {
    if found(node) {
        return Some(node.clone());
    }
    if skipped(node) {
        return None;
    }
    kids(node).find_map(|child| first(&child, found))
}

/// A node's children, each borrowed from the tree rather than from the node.
pub(super) fn kids<'a>(node: &Node<'a>) -> impl Iterator<Item = Node<'a>> {
    std::iter::successors(node.first_child(), Node::next_sibling)
}

fn tag<'a>(node: &'a Node) -> Option<&'a str> {
    node.tag_name()
}

fn attr(node: &Node, wanted: &str) -> Option<String> {
    node.attr(wanted).map(|one| one.to_string())
}

fn style(node: &Node) -> Option<String> {
    attr(node, "style")
}

fn skipped(node: &Node) -> bool {
    tag(node).is_some_and(|name| super::html::SKIPPED.contains(&name))
}

// A tag or a style that changes how words look is formatting; a paragraph, a line or a plain span is not.
fn formatted(node: &Node) -> bool {
    if skipped(node) {
        return false;
    }
    let shown = match tag(node) {
        Some("b") => !style(node).is_some_and(|said| {
            let flat = said.replace(' ', "");
            flat.contains("font-weight:normal") || flat.contains("font-weight:400")
        }),
        Some("span" | "font") => painted(node),
        Some(name) => FORMATTING.contains(&name),
        None => false,
    };
    shown || kids(node).any(|child| formatted(&child))
}

fn painted(node: &Node) -> bool {
    let style = style(node).unwrap_or_default();
    let flat = style.to_ascii_lowercase().replace(' ', "");
    let heavy = [
        "font-weight:600",
        "font-weight:700",
        "font-weight:800",
        "font-weight:bold",
    ]
    .iter()
    .any(|one| flat.contains(one));
    heavy
        || flat.contains("font-style:italic")
        || super::html::written_hue(&style, attr(node, "color").as_deref()).is_some()
}

fn words(node: &Node, pre: bool, out: &mut String) {
    match node.node_type() {
        NodeType::Text => {
            let Some(said) = node.text() else {
                return;
            };
            for one in said.chars() {
                match one {
                    '\u{a0}' => out.push(' '),
                    _ if pre => out.push(one),
                    _ if one.is_ascii_whitespace() => {
                        if !out.is_empty() && !out.ends_with([' ', '\n']) {
                            out.push(' ');
                        }
                    }
                    _ => out.push(one),
                }
            }
        }
        NodeType::Document => {
            for child in kids(node) {
                words(&child, pre, out);
            }
        }
        NodeType::Element => {
            let name = tag(node).unwrap_or_default();
            if skipped(node) {
                return;
            }
            if name == "br" {
                if !pre {
                    let kept = out.trim_end_matches(' ').len();
                    out.truncate(kept);
                }
                out.push('\n');
                return;
            }
            // A blank line before the rule keeps the words above from turning into a heading.
            if name == "hr" && !pre {
                lined(out, pre);
                out.push_str("\n---\n\n");
                return;
            }
            let block = BLOCKS.contains(&name);
            let pre = pre
                || name == "pre"
                || style(node).is_some_and(|said| {
                    let flat = said.to_ascii_lowercase().replace(' ', "");
                    flat.contains("white-space:pre")
                });
            if block {
                lined(out, pre);
            }
            for child in kids(node) {
                words(&child, pre, out);
            }
            if block {
                lined(out, pre);
                if !pre && SPACED.contains(&name) && !out.is_empty() && !out.ends_with("\n\n") {
                    out.push('\n');
                }
            }
        }
        _ => {}
    }
}

fn lined(out: &mut String, pre: bool) {
    if !pre {
        let kept = out.trim_end_matches(' ').len();
        out.truncate(kept);
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

fn evened(text: &str) -> String {
    let even = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut out: Vec<&str> = Vec::new();
    for line in even.lines() {
        let blank = line.trim().is_empty();
        if blank && out.last().is_none_or(|was: &&str| was.is_empty()) {
            continue;
        }
        out.push(if blank { "" } else { line.trim_end() });
    }
    while out.last().is_some_and(|one| one.is_empty()) {
        out.pop();
    }
    out.join("\n")
}

// A page that sets each line in its own paragraph leaves a blank between list items or table rows that splits them.
fn knitted(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    for (at, line) in lines.iter().enumerate() {
        let between = line.is_empty() && at > 0 && at + 1 < lines.len();
        if between {
            let (before, after) = (lines[at - 1], lines[at + 1]);
            if (itemed(before) && itemed(after)) || (rowed(before) && rowed(after)) {
                continue;
            }
        }
        out.push(line);
    }
    out.join("\n")
}

/// Whether text reads as Markdown: a block it opens, or with `blocks` false, a mark inside a line too.
pub(super) fn markdowned(text: &str, blocks: bool) -> bool {
    let fenced = text.lines().any(|one| {
        let flat = one.trim_start();
        flat.starts_with("```") || flat.starts_with("~~~")
    });
    let ruled = text.lines().any(|one| separator(one.trim()));
    let lines: Vec<&str> = text.lines().collect();
    // A script's `# comment` sits right on its command; a Markdown heading stands apart from what follows.
    let apart = |at: usize| !blocks || lines.get(at + 1).is_none_or(|next| next.trim().is_empty());
    fenced
        || lines.iter().enumerate().any(|(at, line)| {
            let flat = line.trim_start();
            (headed(flat) && apart(at))
                || itemed(line)
                || flat.starts_with("> ")
                || (ruled && rowed(line))
                || (!blocks && inline(flat))
        })
}

fn headed(flat: &str) -> bool {
    let hashes = flat.chars().take_while(|one| *one == '#').count();
    (1..=6).contains(&hashes)
        && flat[hashes..].starts_with(' ')
        && !flat[hashes..].trim().is_empty()
}

/// A line that opens a list item: a dash, a star or a plus, or a number, then a space and words.
pub(super) fn itemed(line: &str) -> bool {
    let flat = line.trim_start();
    let digits = flat.chars().take_while(char::is_ascii_digit).count();
    let rest = match digits {
        0 => flat.strip_prefix(['-', '*', '+']),
        1..=3 => flat[digits..].strip_prefix(['.', ')']),
        _ => None,
    };
    rest.is_some_and(|after| after.starts_with(' ') && !after.trim().is_empty())
}

fn rowed(line: &str) -> bool {
    let flat = line.trim();
    flat.len() > 2 && flat.starts_with('|') && flat.ends_with('|')
}

fn separator(flat: &str) -> bool {
    flat.contains("---")
        && flat.contains('|')
        && flat.chars().all(|one| matches!(one, '|' | '-' | ':' | ' '))
}

fn inline(flat: &str) -> bool {
    let paired = |mark: &str| {
        flat.find(mark).is_some_and(|open| {
            let rest = &flat[open + mark.len()..];
            !rest.starts_with(' ') && rest.find(mark).is_some_and(|shut| shut > 0)
        })
    };
    let linked = flat
        .find('[')
        .is_some_and(|open| flat[open..].find("](").is_some_and(|at| at > 1));
    paired("**") || paired("~~") || paired("`") || linked
}

fn outside_fences(text: &str) -> impl Iterator<Item = &str> {
    let mut fence: Option<String> = None;
    text.lines().filter(move |line| {
        let flat = line.trim_start();
        let mark = ['`', '~'].into_iter().find_map(|mark| {
            let run = flat.chars().take_while(|one| *one == mark).count();
            (run >= 3).then(|| mark.to_string().repeat(run))
        });
        match (&fence, mark) {
            (Some(open), Some(mark)) if mark.starts_with(open.as_str()) => {
                fence = None;
                false
            }
            (Some(_), _) => false,
            (None, Some(mark)) => {
                fence = Some(mark);
                false
            }
            (None, None) => true,
        }
    })
}
