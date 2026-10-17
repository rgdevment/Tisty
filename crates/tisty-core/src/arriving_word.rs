use std::collections::HashMap;
use std::io::Read;

use roxmltree::{Document, Node};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

// Past this a document is a book of pictures, and nothing a person pastes by hand.
const PART_AT_MOST: u64 = 32 * 1024 * 1024;

pub struct Word {
    pub html: String,
    pub pictures: Vec<(String, Vec<u8>)>,
}

pub fn from_word(bytes: &[u8]) -> Option<Word> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut part = |name: &str| -> Option<String> {
        let file = zip.by_name(name).ok()?;
        let mut said = String::new();
        file.take(PART_AT_MOST).read_to_string(&mut said).ok()?;
        Some(said)
    };
    let body = part("word/document.xml")?;
    let styles = part("word/styles.xml").unwrap_or_default();
    let numbering = part("word/numbering.xml").unwrap_or_default();
    let rels = part("word/_rels/document.xml.rels").unwrap_or_default();

    let reading = Reading {
        styles: styled(&styles),
        ordered: ordered(&numbering),
        targets: targets(&rels),
    };
    let doc = Document::parse(&body).ok()?;
    let html = reading.body(&doc);
    let pictures = reading
        .targets
        .values()
        .filter(|one| one.starts_with("media/"))
        .filter_map(|one| {
            let mut file = zip.by_name(&format!("word/{one}")).ok()?;
            let mut bytes = Vec::new();
            (&mut file)
                .take(PART_AT_MOST)
                .read_to_end(&mut bytes)
                .ok()?;
            Some((one.clone(), bytes))
        })
        .collect();
    Some(Word { html, pictures })
}

struct Reading {
    styles: HashMap<String, String>,
    ordered: HashMap<(String, String), bool>,
    targets: HashMap<String, String>,
}

fn named<'a>(node: Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|one| one.has_tag_name((W, tag)))
}

fn val(node: Node, tag: &str) -> Option<String> {
    named(node, tag).and_then(|one| one.attribute((W, "val")).map(str::to_string))
}

// Word keeps a style's English name whatever language the person wrote in, so headings are found by it.
fn styled(xml: &str) -> HashMap<String, String> {
    let Ok(doc) = Document::parse(xml) else {
        return HashMap::new();
    };
    doc.descendants()
        .filter(|one| one.has_tag_name((W, "style")))
        .filter_map(|one| {
            let id = one.attribute((W, "styleId"))?;
            Some((id.to_string(), val(one, "name")?.to_lowercase()))
        })
        .collect()
}

fn ordered(xml: &str) -> HashMap<(String, String), bool> {
    let Ok(doc) = Document::parse(xml) else {
        return HashMap::new();
    };
    let kinds: HashMap<String, HashMap<String, bool>> = doc
        .descendants()
        .filter(|one| one.has_tag_name((W, "abstractNum")))
        .filter_map(|one| {
            let id = one.attribute((W, "abstractNumId"))?.to_string();
            let levels = one
                .children()
                .filter(|lvl| lvl.has_tag_name((W, "lvl")))
                .filter_map(|lvl| {
                    let at = lvl.attribute((W, "ilvl"))?.to_string();
                    Some((at, val(lvl, "numFmt").is_some_and(|fmt| fmt != "bullet")))
                })
                .collect();
            Some((id, levels))
        })
        .collect();
    let mut out = HashMap::new();
    for num in doc.descendants().filter(|one| one.has_tag_name((W, "num"))) {
        let (Some(id), Some(base)) = (num.attribute((W, "numId")), val(num, "abstractNumId"))
        else {
            continue;
        };
        for (level, numbered) in kinds.get(&base).into_iter().flatten() {
            out.insert((id.to_string(), level.clone()), *numbered);
        }
    }
    out
}

fn targets(xml: &str) -> HashMap<String, String> {
    let Ok(doc) = Document::parse(xml) else {
        return HashMap::new();
    };
    doc.descendants()
        .filter(|one| one.tag_name().name() == "Relationship")
        .filter_map(|one| {
            Some((
                one.attribute("Id")?.to_string(),
                one.attribute("Target")?.to_string(),
            ))
        })
        .collect()
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl Reading {
    fn body(&self, doc: &Document) -> String {
        let Some(body) = doc.descendants().find(|one| one.has_tag_name((W, "body"))) else {
            return String::new();
        };
        let mut out = String::new();
        // Each open list keeps its last item open too, so a deeper list lands inside it, as HTML wants.
        let mut lists: Vec<&'static str> = Vec::new();
        let shut = |lists: &mut Vec<&'static str>, out: &mut String| {
            if let Some(tag) = lists.pop() {
                out.push_str(&format!("</li></{tag}>"));
            }
        };
        for block in body.children().filter(Node::is_element) {
            let item = self.listed(block);
            let depth = item.map_or(0, |(level, _)| level + 1);
            while lists.len() > depth {
                shut(&mut lists, &mut out);
            }
            let Some((_, numbered)) = item else {
                if block.has_tag_name((W, "tbl")) {
                    out.push_str(&self.table(block));
                } else if block.has_tag_name((W, "p")) {
                    out.push_str(&self.paragraph(block));
                }
                continue;
            };
            let tag = if numbered { "ol" } else { "ul" };
            if lists.len() == depth && lists.last() != Some(&tag) {
                shut(&mut lists, &mut out);
            }
            if lists.len() == depth {
                out.push_str("</li><li>");
            }
            while lists.len() < depth {
                out.push_str(&format!("<{tag}><li>"));
                lists.push(tag);
            }
            out.push_str(&self.runs(block));
        }
        while !lists.is_empty() {
            shut(&mut lists, &mut out);
        }
        out
    }

    fn listed(&self, block: Node) -> Option<(usize, bool)> {
        let props = named(block, "pPr")?;
        let num = named(props, "numPr")?;
        let level = val(num, "ilvl").unwrap_or_else(|| "0".into());
        let id = val(num, "numId")?;
        if id == "0" {
            return None;
        }
        let numbered = self
            .ordered
            .get(&(id, level.clone()))
            .copied()
            .unwrap_or(false);
        Some((level.parse().unwrap_or(0), numbered))
    }

    fn paragraph(&self, block: Node) -> String {
        let style = named(block, "pPr")
            .and_then(|props| val(props, "pStyle"))
            .and_then(|id| self.styles.get(&id).cloned())
            .unwrap_or_default();
        let said = self.runs(block);
        if said.trim().is_empty() {
            return String::new();
        }
        let tag = match style.as_str() {
            "title" => "h1".to_string(),
            "subtitle" => "h2".to_string(),
            "quote" | "intense quote" => return format!("<blockquote><p>{said}</p></blockquote>"),
            one => match one
                .strip_prefix("heading ")
                .and_then(|n| n.parse::<u8>().ok())
            {
                Some(n @ 1..=6) => format!("h{n}"),
                _ => "p".to_string(),
            },
        };
        format!("<{tag}>{said}</{tag}>")
    }

    fn table(&self, table: Node) -> String {
        let mut out = String::from("<table>");
        for (at, row) in table
            .children()
            .filter(|one| one.has_tag_name((W, "tr")))
            .enumerate()
        {
            let cell = if at == 0 { "th" } else { "td" };
            out.push_str("<tr>");
            for one in row.children().filter(|one| one.has_tag_name((W, "tc"))) {
                let said: Vec<String> = one
                    .children()
                    .filter(|p| p.has_tag_name((W, "p")))
                    .map(|p| self.runs(p))
                    .filter(|said| !said.trim().is_empty())
                    .collect();
                out.push_str(&format!("<{cell}>{}</{cell}>", said.join("<br>")));
            }
            out.push_str("</tr>");
        }
        out.push_str("</table>");
        out
    }

    fn runs(&self, block: Node) -> String {
        let mut out = String::new();
        for one in block.children() {
            if one.has_tag_name((W, "r")) {
                out.push_str(&self.run(one));
            } else if one.has_tag_name((W, "hyperlink")) {
                let words = self.runs(one);
                match one.attribute((R, "id")).and_then(|id| self.targets.get(id)) {
                    Some(to) => out.push_str(&format!("<a href=\"{}\">{words}</a>", escaped(to))),
                    None => out.push_str(&words),
                }
            } else if one.has_tag_name((W, "ins")) || one.has_tag_name((W, "smartTag")) {
                out.push_str(&self.runs(one));
            }
        }
        out
    }

    fn run(&self, run: Node) -> String {
        let mut said = String::new();
        for one in run.children() {
            match one.tag_name().name() {
                "t" => said.push_str(&escaped(one.text().unwrap_or_default())),
                "tab" => said.push(' '),
                "br" => said.push_str("<br>"),
                "drawing" => said.push_str(&self.picture(one)),
                _ => {}
            }
        }
        if said.is_empty() {
            return said;
        }
        let Some(props) = named(run, "rPr") else {
            return said;
        };
        let on = |tag: &str| {
            named(props, tag).is_some_and(|one| {
                one.attribute((W, "val"))
                    .is_none_or(|val| !matches!(val, "0" | "false" | "none"))
            })
        };
        let mut style = String::new();
        if let Some(color) = val(props, "color").filter(|one| one != "auto") {
            style.push_str(&format!("color:#{color};"));
        }
        let lit = val(props, "highlight")
            .and_then(|one| lit(&one).map(str::to_string))
            .or_else(|| {
                named(props, "shd")
                    .and_then(|one| one.attribute((W, "fill")))
                    .filter(|one| *one != "auto" && !one.eq_ignore_ascii_case("ffffff"))
                    .map(|one| format!("#{one}"))
            });
        if let Some(lit) = lit {
            style.push_str(&format!("background-color:{lit};"));
        }
        if !style.is_empty() {
            said = format!("<span style=\"{style}\">{said}</span>");
        }
        for (tag, html) in [("strike", "s"), ("u", "u"), ("i", "em"), ("b", "strong")] {
            if on(tag) {
                said = format!("<{html}>{said}</{html}>");
            }
        }
        said
    }

    fn picture(&self, drawing: Node) -> String {
        let blip = drawing
            .descendants()
            .find(|one| one.has_tag_name((A, "blip")));
        let target = blip
            .and_then(|one| one.attribute((R, "embed")))
            .and_then(|id| self.targets.get(id));
        let alt = drawing
            .descendants()
            .find(|one| one.tag_name().name() == "docPr")
            .and_then(|one| one.attribute("descr").or_else(|| one.attribute("name")))
            .unwrap_or_default();
        match target {
            Some(to) => format!("<img src=\"{}\" alt=\"{}\">", escaped(to), escaped(alt)),
            None => String::new(),
        }
    }
}

// Word names its highlights; these are the colours it paints them with.
fn lit(name: &str) -> Option<&'static str> {
    Some(match name {
        "yellow" => "#ffff00",
        "green" => "#00ff00",
        "cyan" => "#00ffff",
        "magenta" => "#ff00ff",
        "blue" => "#0000ff",
        "red" => "#ff0000",
        "darkBlue" => "#000080",
        "darkCyan" => "#008080",
        "darkGreen" => "#008000",
        "darkMagenta" => "#800080",
        "darkRed" => "#800000",
        "darkYellow" => "#808000",
        _ => return None,
    })
}
