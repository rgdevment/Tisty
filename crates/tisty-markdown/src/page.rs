use std::collections::HashMap;
use std::fmt::Write as _;

use crate::inline::{escaped, inline};
use crate::{Block, Callout, Column, Doc, Holds, Kind, Lean, values};

struct Drawing {
    anchors: HashMap<usize, String>,
}

pub fn html(doc: &Doc) -> String {
    let drawing = Drawing {
        anchors: anchors(doc),
    };
    let mut out = String::new();
    for block in doc.blocks() {
        draw(block, doc.text(), &drawing, &mut out);
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

fn draw(block: &Block, text: &str, drawing: &Drawing, out: &mut String) {
    let at = format!("data-at=\"{}-{}\"", block.at.start, block.at.end);
    match &block.kind {
        Kind::Heading(level) => {
            let id = drawing
                .anchors
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
            drawing,
            out,
        ),
        Kind::Callout(kind) => called(*kind, &at, block, text, drawing, out),
        Kind::List { start } => {
            let open = match start {
                None => format!("<ul {at}>"),
                Some(1) => format!("<ol {at}>"),
                Some(first) => format!("<ol start=\"{first}\" {at}>"),
            };
            let shut = if start.is_some() { "</ol>" } else { "</ul>" };
            wrapped(&open, shut, block, text, drawing, out);
        }
        Kind::Item { done } => item(*done, &at, block, text, drawing, out),
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
            wrapped(&head, "</details>", block, text, drawing, out);
        }
        Kind::Note { label } => {
            let head = format!(
                "<div class=\"footnote\" id=\"fn-{0}\" {at}><span class=\"fn-label\">{1}</span>",
                escaped(&label.to_lowercase()),
                escaped(label)
            );
            wrapped(&head, "</div>", block, text, drawing, out);
        }
        Kind::Row { .. } | Kind::Cell => {}
    }
}

fn wrapped(open: &str, shut: &str, block: &Block, text: &str, drawing: &Drawing, out: &mut String) {
    out.push_str(open);
    for inner in &block.inner {
        draw(inner, text, drawing, out);
    }
    out.push_str(shut);
}

fn called(kind: Callout, at: &str, block: &Block, text: &str, drawing: &Drawing, out: &mut String) {
    let name = match kind {
        Callout::Note => "note",
        Callout::Tip => "tip",
        Callout::Important => "important",
        Callout::Warning => "warning",
        Callout::Caution => "caution",
    };
    let open = format!("<blockquote class=\"callout\" data-callout=\"{name}\" {at}>");
    wrapped(&open, "</blockquote>", block, text, drawing, out);
}

fn item(
    done: Option<bool>,
    at: &str,
    block: &Block,
    text: &str,
    drawing: &Drawing,
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
        draw(inner, text, drawing, out);
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
        // The window draws it; the page carries what it is drawn from, readable until then.
        #[cfg(feature = "mermaid")]
        Some(tongue @ ("mermaid" | "mmd")) => {
            let _ = write!(
                out,
                "<figure class=\"diagram\" {at}><pre><code class=\"language-{tongue}\">{}</code></pre></figure>",
                escaped(said)
            );
            true
        }
        _ => false,
    }
}

// The block is a box of its own so a table wider than the sheet scrolls instead of squeezing its words.
fn table(columns: &[Column], at: &str, block: &Block, text: &str, out: &mut String) {
    let _ = write!(out, "<div class=\"table\" {at}><table><colgroup>");
    for column in columns {
        match column.wide {
            Some(wide) => {
                let _ = write!(out, "<col style=\"width:{wide}px\">");
            }
            None => out.push_str("<col>"),
        }
    }
    out.push_str("</colgroup>");
    let read = values::read(columns, block, text);
    let mut body = false;
    for (row_at, row) in block.inner.iter().enumerate() {
        let head = matches!(row.kind, Kind::Row { head: true });
        if !head && !body {
            out.push_str("<tbody>");
            body = true;
        }
        out.push_str(if head { "<thead><tr>" } else { "<tr>" });
        for (index, cell) in row.inner.iter().enumerate() {
            let column = columns.get(index);
            if head {
                headed(column, out);
                inline(&cell.spans, text, out);
                out.push_str("</th>");
            } else {
                let seen = row_at.checked_sub(1).and_then(|one| read.at(one, index));
                celled(column, seen, cell, text, out);
            }
        }
        out.push_str(if head { "</tr></thead>" } else { "</tr>" });
    }
    if body {
        out.push_str("</tbody>");
    }
    if let Some(foot) = &read.foot {
        out.push_str("<tfoot><tr>");
        for (index, said) in foot.iter().enumerate() {
            let _ = write!(
                out,
                "<td{}{}>{}</td>",
                whole(columns.get(index)),
                leaned(columns.get(index)),
                escaped(said)
            );
        }
        out.push_str("</tr></tfoot>");
    }
    out.push_str("</table></div>");
}

fn leaned(column: Option<&Column>) -> &'static str {
    match column.and_then(Column::leaning) {
        Some(Lean::Left) => " style=\"text-align:left\"",
        Some(Lean::Center) => " style=\"text-align:center\"",
        Some(Lean::Right) => " style=\"text-align:right\"",
        None => "",
    }
}

// The head says what its column holds, so the window can sort, add up and edit it as such.
fn headed(column: Option<&Column>, out: &mut String) {
    out.push_str("<th scope=\"col\"");
    if let Some(column) = column.filter(|one| one.typed.is_some()) {
        let _ = write!(out, " data-type=\"{}\"", column.holds.word());
        if let Holds::Money {
            currency: Some(currency),
        } = &column.holds
        {
            let _ = write!(out, " data-currency=\"{}\"", escaped(currency));
        }
        if let Some(total) = column.total {
            let _ = write!(out, " data-total=\"{}\"", total.word());
        }
    }
    let _ = write!(out, "{}>", leaned(column));
}

fn whole(column: Option<&Column>) -> &'static str {
    match column {
        Some(one) if one.typed.is_some() && one.holds.whole() => " class=\"whole\"",
        _ => "",
    }
}

fn celled(
    column: Option<&Column>,
    seen: Option<&values::Seen>,
    cell: &Block,
    text: &str,
    out: &mut String,
) {
    let lean = leaned(column);
    let Some(seen) = seen.filter(|_| column.is_some_and(|one| one.typed.is_some())) else {
        let _ = write!(out, "<td{lean}>");
        inline(&cell.spans, text, out);
        out.push_str("</td>");
        return;
    };
    out.push_str("<td");
    if let Some(key) = &seen.key {
        let _ = write!(out, " data-key=\"{}\"", escaped(key));
    }
    if seen.amiss {
        out.push_str(" data-amiss");
    }
    let _ = write!(out, "{}{lean}>", whole(column));
    match &seen.shown {
        Some(shown) => out.push_str(&escaped(shown)),
        None => inline(&cell.spans, text, out),
    }
    out.push_str("</td>");
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
