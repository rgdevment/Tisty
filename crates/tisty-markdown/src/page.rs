use std::collections::HashMap;
use std::fmt::Write as _;

use crate::inline::{escaped, inline};
use crate::{Block, Callout, Column, Doc, Kind, Lean};

pub fn html(doc: &Doc) -> String {
    let anchors = anchors(doc);
    let mut out = String::new();
    for block in doc.blocks() {
        draw(block, doc.text(), &anchors, &mut out);
    }
    out
}

fn anchors(doc: &Doc) -> HashMap<usize, String> {
    #[cfg(feature = "outline")]
    return crate::outline(doc)
        .into_iter()
        .map(|head| (head.at.start, head.id))
        .collect();
    #[cfg(not(feature = "outline"))]
    {
        let _ = doc;
        HashMap::new()
    }
}

fn front(said: &str, at: &str, out: &mut String) {
    let _ = write!(out, "<dl class=\"front\" {at}>");
    for field in crate::front::fields(said) {
        let _ = write!(out, "<div><dt>{}</dt><dd>", escaped(&field.key));
        let tagged = crate::front::tagging(&field.key);
        let values: Vec<String> = match tagged {
            true => crate::front::words(&field.values),
            false => field.values.clone(),
        };
        for (n, value) in values.iter().enumerate() {
            let named = tagged
                .then(|| crate::tags::named(value.trim_start_matches('#')))
                .flatten();
            match named {
                Some(name) => {
                    let _ = write!(
                        out,
                        "<span class=\"tag\" data-tag=\"{}\">#{}</span>",
                        escaped(&name),
                        escaped(value.trim_start_matches('#'))
                    );
                }
                None => {
                    if n > 0 {
                        out.push_str("<span class=\"front-sep\">, </span>");
                    }
                    out.push_str(&escaped(value));
                }
            }
        }
        out.push_str("</dd></div>");
    }
    out.push_str("</dl>");
}

fn draw(block: &Block, text: &str, anchors: &HashMap<usize, String>, out: &mut String) {
    let at = format!("data-at=\"{}-{}\"", block.at.start, block.at.end);
    match &block.kind {
        Kind::Heading(level) => {
            let id = anchors
                .get(&block.at.start)
                .map(|id| format!(" id=\"{}\"", escaped(id)));
            let _ = write!(out, "<h{level}{} {at}>", id.unwrap_or_default());
            inline(&block.spans, text, out);
            let _ = write!(out, "</h{level}>");
        }
        Kind::Paragraph => {
            let _ = write!(out, "<p {at}>");
            inline(&block.spans, text, out);
            out.push_str("</p>");
        }
        Kind::Quote => wrapped(
            &format!("<blockquote {at}>"),
            "</blockquote>",
            block,
            text,
            anchors,
            out,
        ),
        Kind::Callout(kind) => called(*kind, &at, block, text, anchors, out),
        Kind::List { start } => {
            let open = match start {
                None => format!("<ul {at}>"),
                Some(1) => format!("<ol {at}>"),
                Some(first) => format!("<ol start=\"{first}\" {at}>"),
            };
            let shut = if start.is_some() { "</ol>" } else { "</ul>" };
            wrapped(&open, shut, block, text, anchors, out);
        }
        Kind::Item { done } => item(*done, &at, block, text, anchors, out),
        Kind::Code { tongue, title } => {
            code(tongue.as_deref(), title.as_deref(), &at, block, text, out)
        }
        Kind::Table { columns } => table(columns, &at, block, text, out),
        Kind::Html => {
            let _ = write!(
                out,
                "<div {at}>{}</div>",
                cleaned(&unquoted_lines(&text[block.at.clone()]))
            );
        }
        Kind::Rule => {
            let _ = write!(out, "<hr {at}>");
        }
        Kind::Front => front(&text[block.at.clone()], &at, out),
        Kind::Definition { label, to } => {
            let _ = write!(
                out,
                "<p class=\"refdef\" {at}><span class=\"refdef-label\">[{}]</span> <span class=\"refdef-to\">{}</span></p>",
                escaped(label),
                escaped(to)
            );
        }
        Kind::Details { summary, open } => {
            let open = if *open { " open" } else { "" };
            let head = format!(
                "<details{open} {at}><summary>{}</summary>",
                escaped(summary)
            );
            wrapped(&head, "</details>", block, text, anchors, out);
        }
        Kind::Note { label } => {
            let head = format!(
                "<div class=\"footnote\" id=\"fn-{0}\" {at}><span class=\"fn-label\">{1}</span>",
                escaped(&label.to_lowercase()),
                escaped(label)
            );
            wrapped(&head, "</div>", block, text, anchors, out);
        }
        Kind::Row { .. } | Kind::Cell => {}
    }
}

fn wrapped(
    open: &str,
    shut: &str,
    block: &Block,
    text: &str,
    anchors: &HashMap<usize, String>,
    out: &mut String,
) {
    out.push_str(open);
    for inner in &block.inner {
        draw(inner, text, anchors, out);
    }
    out.push_str(shut);
}

fn called(
    kind: Callout,
    at: &str,
    block: &Block,
    text: &str,
    anchors: &HashMap<usize, String>,
    out: &mut String,
) {
    let name = match kind {
        Callout::Note => "note",
        Callout::Tip => "tip",
        Callout::Important => "important",
        Callout::Warning => "warning",
        Callout::Caution => "caution",
    };
    let open = format!("<blockquote class=\"callout\" data-callout=\"{name}\" {at}>");
    wrapped(&open, "</blockquote>", block, text, anchors, out);
}

fn item(
    done: Option<bool>,
    at: &str,
    block: &Block,
    text: &str,
    anchors: &HashMap<usize, String>,
    out: &mut String,
) {
    match done {
        Some(ticked) => {
            let _ = write!(
                out,
                "<li class=\"task\" {at}><span class=\"check\" data-done=\"{ticked}\"></span>"
            );
        }
        None => {
            let _ = write!(out, "<li {at}>");
        }
    }
    inline(&block.spans, text, out);
    for inner in &block.inner {
        draw(inner, text, anchors, out);
    }
    out.push_str("</li>");
}

fn code(
    tongue: Option<&str>,
    title: Option<&str>,
    at: &str,
    block: &Block,
    text: &str,
    out: &mut String,
) {
    let said: String = block
        .spans
        .iter()
        .map(|one| &text[one.at.clone()])
        .collect();
    if drawn(tongue, &said, at, out) {
        return;
    }
    let _ = write!(out, "<figure class=\"code\" {at}>");
    if let Some(title) = title {
        let _ = write!(out, "<figcaption>{}</figcaption>", escaped(title));
    }
    let class = tongue
        .map(|one| format!(" class=\"language-{}\"", escaped(one)))
        .unwrap_or_default();
    let _ = write!(
        out,
        "<pre><code{class}>{}</code></pre></figure>",
        escaped(&said)
    );
}

#[cfg(not(any(feature = "math", feature = "mermaid")))]
fn drawn(_: Option<&str>, _: &str, _: &str, _: &mut String) -> bool {
    false
}

#[cfg(any(feature = "math", feature = "mermaid"))]
fn drawn(tongue: Option<&str>, said: &str, at: &str, out: &mut String) -> bool {
    match tongue {
        #[cfg(feature = "math")]
        Some("math") => match crate::math::mathml(said, true) {
            Some(mathml) => {
                let _ = write!(out, "<div class=\"math\" {at}>{mathml}</div>");
                true
            }
            None => false,
        },
        #[cfg(feature = "mermaid")]
        Some("mermaid" | "mmd") => match crate::diagram::drawn(said) {
            Some(drawn) => {
                let _ = write!(
                    out,
                    "<figure class=\"diagram\" {at}><img alt=\"\" src=\"{}\"></figure>",
                    svg_url(&drawn)
                );
                true
            }
            None => false,
        },
        _ => false,
    }
}

#[cfg(feature = "mermaid")]
fn svg_url(svg: &str) -> String {
    let mut url = String::from("data:image/svg+xml,");
    let mut gap = false;
    for byte in svg.bytes() {
        // A run of spaces between attributes says nothing; one is kept so the words in a label stay apart.
        if byte.is_ascii_whitespace() {
            gap = true;
            continue;
        }
        if std::mem::take(&mut gap) {
            url.push_str("%20");
        }
        if byte.is_ascii_alphanumeric() || b"-_.~/:=;'(),!*+?@$".contains(&byte) {
            url.push(char::from(byte));
        } else {
            let _ = write!(url, "%{byte:02X}");
        }
    }
    url
}

fn table(columns: &[Column], at: &str, block: &Block, text: &str, out: &mut String) {
    let _ = write!(out, "<table {at}><colgroup>");
    for column in columns {
        match column.wide {
            Some(wide) => {
                let _ = write!(out, "<col style=\"width:{wide}px\">");
            }
            None => out.push_str("<col>"),
        }
    }
    out.push_str("</colgroup>");
    let mut body = false;
    for row in &block.inner {
        let head = matches!(row.kind, Kind::Row { head: true });
        if !head && !body {
            out.push_str("<tbody>");
            body = true;
        }
        out.push_str(if head { "<thead><tr>" } else { "<tr>" });
        for (index, cell) in row.inner.iter().enumerate() {
            let tag = if head { "th" } else { "td" };
            let lean = columns
                .get(index)
                .and_then(|one| one.lean)
                .map(|lean| match lean {
                    Lean::Left => " style=\"text-align:left\"",
                    Lean::Center => " style=\"text-align:center\"",
                    Lean::Right => " style=\"text-align:right\"",
                });
            let _ = write!(out, "<{tag}{}>", lean.unwrap_or_default());
            inline(&cell.spans, text, out);
            let _ = write!(out, "</{tag}>");
        }
        out.push_str(if head { "</tr></thead>" } else { "</tr>" });
    }
    if body {
        out.push_str("</tbody>");
    }
    out.push_str("</table>");
}

// Inside a quote the block still carries each line's marker, which is not part of its html.
fn unquoted_lines(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.trim_start().strip_prefix('>') {
            Some(_) => crate::unquoted(line),
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn cleaned(html: &str) -> String {
    #[cfg(feature = "html")]
    return ammonia::Builder::default()
        .url_schemes(crate::inline::SCHEMES.iter().copied().collect())
        .add_tag_attributes("span", ["data-ico", "data-hue"])
        .add_tag_attributes("mark", ["data-pen"])
        .clean(html)
        .to_string();
    #[cfg(not(feature = "html"))]
    format!("<pre>{}</pre>", escaped(html))
}
