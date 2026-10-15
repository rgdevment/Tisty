use std::collections::HashMap;
use std::fmt::Write as _;

use crate::inline::{escaped, inline};
use crate::{Block, Callout, Column, Doc, Kind, Lean, Mark};

/// The document as plain html for the reader: no scripts, and every block tells where it was written.
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
            let _ = write!(out, "<div {at}>{}</div>", cleaned(&text[block.at.clone()]));
        }
        Kind::Rule => {
            let _ = write!(out, "<hr {at}>");
        }
        Kind::Details { summary, open } => {
            let open = if *open { " open" } else { "" };
            let head = format!(
                "<details{open} {at}><summary>{}</summary>",
                escaped(summary)
            );
            wrapped(&head, "</details>", block, text, anchors, out);
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
    let _ = write!(
        out,
        "<blockquote class=\"callout\" data-callout=\"{name}\" {at}>"
    );
    for (index, inner) in block.inner.iter().enumerate() {
        if index == 0 && inner.kind == Kind::Paragraph {
            let spans = unmarked(&inner.spans, text);
            if !spans.is_empty() {
                let _ = write!(out, "<p data-at=\"{}-{}\">", inner.at.start, inner.at.end);
                inline(&spans, text, out);
                out.push_str("</p>");
            }
            continue;
        }
        draw(inner, text, anchors, out);
    }
    out.push_str("</blockquote>");
}

// The tokenizer splits `[!WARNING]` around its brackets, so the marker is cut in the bytes.
fn unmarked(spans: &[crate::Span], text: &str) -> Vec<crate::Span> {
    let Some(start) = spans.iter().map(|one| one.at.start).min() else {
        return Vec::new();
    };
    let Some(close) = text[start..]
        .strip_prefix("[!")
        .and_then(|rest| rest.find(']'))
    else {
        return spans.to_vec();
    };
    let after = start + 2 + close + 1;
    let rest = &text[after..];
    let cut = after + (rest.len() - rest.trim_start_matches([' ', '\t']).len());
    let mut kept: Vec<crate::Span> = spans
        .iter()
        .filter(|one| one.at.end > cut)
        .cloned()
        .map(|mut one| {
            one.at.start = one.at.start.max(cut);
            one
        })
        .collect();
    if kept.first().is_some_and(|one| one.mark == Mark::Break) {
        kept.remove(0);
    }
    kept
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

// Formulas and diagrams are drawn here once; a fence that cannot be drawn falls back to its code.
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
                    "<picture class=\"diagram\" {at}><source media=\"(prefers-color-scheme: dark)\" srcset=\"{}\"><img alt=\"\" src=\"{}\"></picture>",
                    svg_url(&drawn.dark),
                    svg_url(&drawn.light)
                );
                true
            }
            None => false,
        },
        _ => {
            let _ = (said, at, &out);
            false
        }
    }
}

#[cfg(feature = "mermaid")]
fn svg_url(svg: &str) -> String {
    let mut url = String::from("data:image/svg+xml,");
    for byte in svg.bytes() {
        if byte.is_ascii_alphanumeric() || b" -_.~/:=;,'()".contains(&byte) {
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

fn cleaned(html: &str) -> String {
    #[cfg(feature = "html")]
    return ammonia::Builder::default()
        .add_tags([
            "details",
            "summary",
            "kbd",
            "mark",
            "colgroup",
            "col",
            "figure",
            "figcaption",
        ])
        .add_tag_attributes("span", ["data-ico", "data-hue"])
        .add_tag_attributes("mark", ["data-pen"])
        .add_tag_attributes("td", ["colspan", "rowspan"])
        .add_tag_attributes("th", ["colspan", "rowspan"])
        .clean(html)
        .to_string();
    #[cfg(not(feature = "html"))]
    format!("<pre>{}</pre>", escaped(html))
}
