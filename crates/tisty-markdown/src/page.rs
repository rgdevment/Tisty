use std::collections::HashMap;
use std::fmt::Write as _;

use crate::inline::{Told, escaped, sourced};
use crate::{Block, Callout, Column, Doc, Holds, Kind, Lean, values};

#[derive(Clone, Copy)]
pub(crate) struct Drawing<'a> {
    pub(crate) anchors: &'a HashMap<usize, String>,
    pub(crate) aligned: bool,
    pub(crate) placed: bool,
    pub(crate) title: Option<usize>,
    // What a comparison shows beside its newer self would name the same place twice.
    pub(crate) named: bool,
    pub(crate) told: Told<'a>,
}

// A sibling rule reaching the block past the properties restyles every block of its kind as blocks come and go.
pub(crate) fn titled(blocks: &[Block]) -> Option<usize> {
    match blocks {
        [front, words, ..]
            if matches!(front.kind, Kind::Front)
                && matches!(words.kind, Kind::Paragraph | Kind::Heading(1..=3)) =>
        {
            Some(words.at.start)
        }
        _ => None,
    }
}

fn title_class(block: &Block, drawing: &Drawing) -> &'static str {
    if drawing.title == Some(block.at.start) {
        " class=\"title\""
    } else {
        ""
    }
}

pub fn html(doc: &Doc) -> String {
    page_of(doc, false)
}

/// The page an editor writes on: the same html, each piece of words naming the bytes it is written with.
pub fn html_placed(doc: &Doc) -> String {
    page_of(doc, true)
}

fn page_of(doc: &Doc, placed: bool) -> String {
    let anchors = anchors(doc);
    let drawing = Drawing {
        anchors: &anchors,
        aligned: false,
        placed,
        title: titled(doc.blocks()),
        named: true,
        told: Told::default(),
    };
    let mut out = String::new();
    for block in doc.blocks() {
        draw(block, doc.text(), &drawing, &mut out);
    }
    out
}

pub struct Near {
    pub count: usize,
    pub sum: [u32; 2],
    pub drawn: std::ops::Range<usize>,
    pub html: String,
    // A heading's name hangs on those before it, so only one the edit touches can rename those it did not.
    pub heads: Option<[u32; 2]>,
    // A link or a footnote reads by a label written anywhere, so a key that changes one can change any block.
    pub whole: bool,
}

fn summed(numbers: impl IntoIterator<Item = u32>) -> [u32; 2] {
    let mut fnv: u32 = 0x811c_9dc5;
    let mut mixed: u32 = 0x9747_b28c;
    for at in numbers {
        fnv = (fnv ^ at).wrapping_mul(0x0100_0193);
        mixed = (mixed ^ at).wrapping_mul(0x5bd1_e995);
        mixed ^= mixed >> 13;
    }
    [fnv, mixed]
}

/// The page sums the blocks it already shows the same way, so the two meet without every place crossing.
pub fn spans_sum(spans: impl IntoIterator<Item = [usize; 2]>) -> [u32; 2] {
    summed(spans.into_iter().flatten().map(|at| at as u32))
}

pub fn names_sum<'a>(names: impl IntoIterator<Item = &'a str>) -> [u32; 2] {
    summed(names.into_iter().flat_map(|name| {
        let units = name.encode_utf16().map(u32::from);
        std::iter::once(name.encode_utf16().count() as u32).chain(units)
    }))
}

fn touched_heading(blocks: &[Block], from: usize, to: usize) -> bool {
    blocks.iter().any(|one| {
        let heading = matches!(one.kind, Kind::Heading(_));
        (heading && one.at.start <= to && from <= one.at.end)
            || touched_heading(&one.inner, from, to)
    })
}

pub fn placed_near(doc: &Doc, from: usize, to: usize, shown: Option<[u32; 2]>) -> Near {
    let blocks = doc.blocks();
    let first = blocks.partition_point(|one| one.at.end < from);
    let last = blocks.partition_point(|one| one.at.start <= to);
    let start = first.saturating_sub(1);
    let end = if last > first { last + 1 } else { start + 1 };
    // The title past the properties is marked by where it stands, so a key at the top draws it too.
    let end = if start == 0 { end.max(2) } else { end }.min(blocks.len());
    let drawn = start.min(end)..end;
    let again = blocks.get(drawn.clone()).unwrap_or_default();
    let (anchors, heads) = if touched_heading(again, from, to) {
        let names = names(doc);
        let around = again.first().zip(again.last());
        let around = around.map_or(0..0, |(first, last)| first.at.start..last.at.end);
        let kept = names.iter().filter(|(at, _)| !around.contains(at));
        let heads = names_sum(kept.map(|(_, name)| name.as_str()));
        (names.into_iter().collect(), Some(heads))
    } else {
        (HashMap::new(), None)
    };
    let drawing = Drawing {
        anchors: &anchors,
        aligned: false,
        placed: true,
        title: titled(blocks),
        named: true,
        told: Told::default(),
    };
    let mut html = String::new();
    for block in again {
        draw(block, doc.text(), &drawing, &mut html);
    }
    let kept = blocks[..drawn.start].iter().chain(&blocks[drawn.end..]);
    Near {
        count: blocks.len(),
        sum: spans_sum(kept.map(|one| [one.at.start, one.at.end])),
        drawn,
        html,
        heads,
        whole: shown != Some(doc.labels()),
    }
}

// Cut only between top-level blocks, so each piece is whole HTML that can be laid after the last.
pub fn pieces(doc: &Doc, about: usize) -> Vec<String> {
    let anchors = anchors(doc);
    let drawing = Drawing {
        anchors: &anchors,
        aligned: true,
        placed: false,
        title: titled(doc.blocks()),
        named: true,
        told: Told::default(),
    };
    let mut all = Vec::new();
    let mut out = String::new();
    for block in doc.blocks() {
        draw(block, doc.text(), &drawing, &mut out);
        if out.len() >= about {
            all.push(std::mem::take(&mut out));
        }
    }
    if !out.is_empty() {
        all.push(out);
    }
    all
}

fn names(doc: &Doc) -> Vec<(usize, String)> {
    #[cfg(feature = "outline")]
    return crate::outline(doc)
        .into_iter()
        .map(|head| (head.at.start, head.id))
        .collect();
    #[cfg(not(feature = "outline"))]
    {
        let _ = doc;
        Vec::new()
    }
}

pub(crate) fn anchors(doc: &Doc) -> HashMap<usize, String> {
    names(doc).into_iter().collect()
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

pub(crate) fn draw(block: &Block, text: &str, drawing: &Drawing, out: &mut String) {
    draw_as(block, text, drawing, &placed_at(block, None), out);
}

pub(crate) fn placed_at(block: &Block, change: Option<&str>) -> String {
    let (start, end) = (block.at.start, block.at.end);
    match change {
        Some(how) => format!("data-at=\"{start}-{end}\" data-change=\"{how}\""),
        None => format!("data-at=\"{start}-{end}\""),
    }
}

/// What a block that only holds others opens and shuts with.
pub(crate) fn shell(
    block: &Block,
    text: &str,
    drawing: &Drawing,
    at: &str,
) -> Option<(String, &'static str)> {
    Some(match &block.kind {
        Kind::Quote => (format!("<blockquote {at}>"), "</blockquote>"),
        Kind::Callout(kind) => (called(*kind, at, block, text), "</blockquote>"),
        Kind::List { start: None } => (format!("<ul {at}>"), "</ul>"),
        Kind::List { start: Some(1) } => (format!("<ol {at}>"), "</ol>"),
        Kind::List { start: Some(first) } => (format!("<ol start=\"{first}\" {at}>"), "</ol>"),
        Kind::Details { summary, open } => {
            let open = if *open { " open" } else { "" };
            let head = format!(
                "<details{open} {at}><summary>{}</summary>",
                escaped(summary)
            );
            (head, "</details>")
        }
        Kind::Aligned { how } => (format!("<div align=\"{how}\" {at}>"), "</div>"),
        Kind::Note { label } => {
            let id = match drawing.named {
                true => format!(" id=\"fn-{}\"", escaped(&label.to_lowercase())),
                false => String::new(),
            };
            let head = format!(
                "<div class=\"footnote\"{id} {at}><span class=\"fn-label\">{}</span>",
                escaped(label)
            );
            (head, "</div>")
        }
        _ => return None,
    })
}

pub(crate) fn draw_as(block: &Block, text: &str, drawing: &Drawing, at: &str, out: &mut String) {
    if let Some((open, shut)) = shell(block, text, drawing, at) {
        out.push_str(&open);
        for inner in &block.inner {
            draw(inner, text, drawing, out);
        }
        out.push_str(shut);
        return;
    }
    match &block.kind {
        Kind::Heading(level) => {
            let id = drawing
                .anchors
                .get(&block.at.start)
                .map(|id| format!(" id=\"{}\"", escaped(id)));
            let class = title_class(block, drawing);
            let _ = write!(out, "<h{level}{}{class} {at}>", id.unwrap_or_default());
            sourced(&block.spans, text, drawing.placed, drawing.told, out);
            let _ = write!(out, "</h{level}>");
        }
        Kind::Paragraph => {
            let _ = write!(out, "<p{} {at}>", title_class(block, drawing));
            sourced(&block.spans, text, drawing.placed, drawing.told, out);
            out.push_str("</p>");
        }
        Kind::Item { done } => {
            item_open(*done, at, block, text, drawing, out);
            for inner in &block.inner {
                draw(inner, text, drawing, out);
            }
            out.push_str("</li>");
        }
        Kind::Code { tongue, title } => code(
            tongue.as_deref(),
            title.as_deref(),
            at,
            (block, text, drawing),
            out,
        ),
        Kind::Table { columns } => {
            let read = values::read(columns, block, text);
            table_open(columns, at, out);
            let mut body = false;
            for row_at in 0..block.inner.len() {
                row(
                    (columns, &read),
                    (block, row_at),
                    text,
                    drawing,
                    None,
                    &mut body,
                    out,
                );
            }
            table_shut(columns, &read, body, out);
        }
        Kind::Html => {
            let _ = write!(
                out,
                "<div {at}>{}</div>",
                cleaned(&unquoted_lines(&text[block.at.clone()]), drawing.aligned)
            );
        }
        Kind::Rule => {
            let _ = write!(out, "<hr {at}>");
        }
        Kind::Front => front(&text[block.at.clone()], at, out),
        Kind::Definition { label, to } => {
            let _ = write!(
                out,
                "<p class=\"refdef\" {at}><span class=\"refdef-label\">[{}]</span> <span class=\"refdef-to\">{}</span></p>",
                escaped(label),
                escaped(to)
            );
        }
        Kind::Quote
        | Kind::Callout(_)
        | Kind::List { .. }
        | Kind::Details { .. }
        | Kind::Aligned { .. }
        | Kind::Note { .. }
        | Kind::Row { .. }
        | Kind::Cell => {}
    }
}

fn called(kind: Callout, at: &str, block: &Block, text: &str) -> String {
    let name = match kind {
        Callout::Note => "note",
        Callout::Tip => "tip",
        Callout::Important => "important",
        Callout::Warning => "warning",
        Callout::Caution => "caution",
        Callout::Idea => "idea",
    };
    let mut said = String::new();
    for (key, value) in crate::IdeaHead::of(kind, &text[block.at.clone()]).said() {
        let _ = write!(said, " data-{key}=\"{value}\"");
    }
    format!("<blockquote class=\"callout\" data-callout=\"{name}\"{said} {at}>")
}

/// An item up to what it holds inside; whoever draws that shuts it with `</li>`.
pub(crate) fn item_open(
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
    sourced(&block.spans, text, drawing.placed, drawing.told, out);
}

fn code(
    tongue: Option<&str>,
    title: Option<&str>,
    at: &str,
    (block, text, drawing): (&Block, &str, &Drawing),
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
        "<pre><code{class}{}>",
        written_at(block, text, &said, drawing.placed)
    );
    let told = drawing.told;
    match told.at.is_empty() {
        true => out.push_str(&escaped(&said)),
        false => block.spans.iter().for_each(|one| {
            told.parted(one.at.clone(), |part, tells| {
                told.marked(tells, out, |out| out.push_str(&escaped(&text[part])));
            });
        }),
    }
    out.push_str("</code></pre></figure>");
}

// Code inside a quote loses each line's marker, so only code written in one piece can be written on.
fn written_at(block: &Block, text: &str, said: &str, placed: bool) -> String {
    let (Some(first), Some(last)) = (block.spans.first(), block.spans.last()) else {
        return String::new();
    };
    let whole = first.at.start..last.at.end;
    if !placed || text.get(whole.clone()) != Some(said) {
        return String::new();
    }
    format!(" data-src=\"{}-{}\"", whole.start, whole.end)
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
pub(crate) fn table_open(columns: &[Column], at: &str, out: &mut String) {
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
}

/// One row of a table, read as its own table reads it; `body` says whether the rows under the head began.
pub(crate) fn row(
    (columns, read): (&[Column], &values::Read),
    (table, row_at): (&Block, usize),
    text: &str,
    drawing: &Drawing,
    change: Option<&str>,
    body: &mut bool,
    out: &mut String,
) {
    let Some(row) = table.inner.get(row_at) else {
        return;
    };
    let head = matches!(row.kind, Kind::Row { head: true });
    if !head && !*body {
        out.push_str("<tbody>");
        *body = true;
    }
    let changed = change
        .map(|how| format!(" data-change=\"{how}\""))
        .unwrap_or_default();
    let _ = write!(out, "{}<tr{changed}>", if head { "<thead>" } else { "" });
    for (index, cell) in row.inner.iter().enumerate() {
        let column = columns.get(index);
        if head {
            headed(column, out);
            sourced(&cell.spans, text, false, drawing.told, out);
            out.push_str("</th>");
        } else {
            let seen = row_at.checked_sub(1).and_then(|one| read.at(one, index));
            // A typed value shows no word to mark, so the cell itself says its words changed.
            let touched = match drawing.told.touches(&cell.at) {
                true => changed.as_str(),
                false => "",
            };
            celled(column, seen, cell, text, (drawing.told, touched), out);
        }
    }
    out.push_str(if head { "</tr></thead>" } else { "</tr>" });
}

pub(crate) fn table_shut(columns: &[Column], read: &values::Read, body: bool, out: &mut String) {
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
    (told, touched): (Told, &str),
    out: &mut String,
) {
    let lean = leaned(column);
    let Some(seen) = seen.filter(|_| column.is_some_and(|one| one.typed.is_some())) else {
        let _ = write!(out, "<td{touched}{lean}>");
        sourced(&cell.spans, text, false, told, out);
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
    let _ = write!(out, "{touched}{}{lean}>", whole(column));
    match &seen.shown {
        Some(shown) => out.push_str(&escaped(shown)),
        None => sourced(&cell.spans, text, false, told, out),
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

#[cfg_attr(not(feature = "html"), allow(unused_variables))]
fn cleaned(html: &str, aligned: bool) -> String {
    #[cfg(feature = "html")]
    {
        let mut builder = ammonia::Builder::default();
        builder
            .url_schemes(crate::inline::SCHEMES.iter().copied().collect())
            .add_tag_attributes("span", ["data-ico", "data-hue"])
            .add_tag_attributes("mark", ["data-pen"]);
        if aligned {
            builder
                .add_tag_attributes("div", ["align"])
                .add_tag_attributes("p", ["align"]);
        }
        builder.clean(html).to_string()
    }
    #[cfg(not(feature = "html"))]
    format!("<pre>{}</pre>", escaped(html))
}

#[cfg(test)]
#[path = "page_test.rs"]
mod tests;
