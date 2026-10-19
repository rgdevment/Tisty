use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;

use roxmltree::{Document, Node};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

// Past this a document is a book of pictures, and nothing a person pastes by hand.
pub const WORD_AT_MOST: u64 = 32 * 1024 * 1024;
// The words alone are read whole into memory several times over, pictures aside.
const WORDS_AT_MOST: u64 = 8 * 1024 * 1024;

pub struct Word {
    pub html: String,
    pub pictures: Vec<(String, Vec<u8>)>,
}

pub fn from_word(bytes: &[u8]) -> Option<Word> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut part = |name: &str| -> Option<String> {
        let file = zip.by_name(name).ok()?;
        if file.size() > WORDS_AT_MOST {
            return None;
        }
        let mut said = String::new();
        file.take(WORDS_AT_MOST).read_to_string(&mut said).ok()?;
        Some(said)
    };
    let body = part("word/document.xml")?;
    let styles = part("word/styles.xml").unwrap_or_default();
    let numbering = part("word/numbering.xml").unwrap_or_default();
    let rels = part("word/_rels/document.xml.rels").unwrap_or_default();

    let doc = Document::parse(&body).ok()?;
    let mut reading = Reading {
        styles: styled(&styles),
        ordered: ordered(&numbering),
        targets: targets(&rels),
        look: HashSet::new(),
    };
    reading.look = reading.dressed(&doc);
    let html = reading.body(&doc);
    let names: HashSet<String> = reading
        .targets
        .values()
        .filter_map(|one| media(one))
        .collect();
    let mut left = WORD_AT_MOST;
    let mut pictures = BTreeMap::new();
    for name in names {
        if left == 0 {
            break;
        }
        let Ok(file) = zip.by_name(&format!("word/media/{name}")) else {
            continue;
        };
        let mut bytes = Vec::new();
        if file.take(left).read_to_end(&mut bytes).is_err() {
            continue;
        }
        left -= bytes.len() as u64;
        pictures.insert(laid(&name), bytes);
    }
    Some(Word {
        html,
        pictures: pictures.into_iter().collect(),
    })
}

struct Reading {
    styles: HashMap<String, String>,
    ordered: HashMap<(String, String), bool>,
    targets: HashMap<String, String>,
    look: HashSet<String>,
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

// A picture's name becomes a file on disk, so only a plain name inside the document's media folder will do.
fn media(target: &str) -> Option<String> {
    let inside = target.strip_prefix("/word/").unwrap_or(target);
    let name = inside.strip_prefix("media/")?;
    let plain = !name.is_empty() && name != ".." && !name.contains(['/', '\\', ':']);
    plain.then(|| name.to_string())
}

// A zip holds names Windows will not write, and one such picture must not cost the whole document.
fn laid(name: &str) -> String {
    let plain = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if plain && !name.starts_with('.') && !name.ends_with('.') && !crate::attach::reserved(name) {
        return name.to_string();
    }
    let ext = name
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| {
            !ext.is_empty() && ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .map(|ext| format!(".{ext}"))
        .unwrap_or_default();
    let printed = crate::attach::printed(name.as_bytes());
    format!("picture-{}{ext}", &printed[..16])
}

fn hex(one: &str) -> Option<String> {
    (one.len() == 6 && one.chars().all(|c| c.is_ascii_hexdigit())).then(|| one.to_ascii_uppercase())
}

// Word has no code of its own; Courier, Consolas and the like are how a document shows it.
fn typewritten(run: Node) -> bool {
    named(run, "rPr")
        .and_then(|props| named(props, "rFonts"))
        .and_then(|fonts| {
            fonts
                .attribute((W, "ascii"))
                .or_else(|| fonts.attribute((W, "hAnsi")))
        })
        .is_some_and(|font| {
            let font = font.to_ascii_lowercase();
            [
                "courier",
                "consolas",
                "mono",
                "menlo",
                "monaco",
                "lucida console",
                "code",
            ]
            .iter()
            .any(|one| font.contains(one))
        })
}

// Content controls and custom XML wrap ordinary paragraphs without changing them.
fn unwrapped<'a>(node: Node<'a, 'a>) -> Vec<Node<'a, 'a>> {
    let mut out = Vec::new();
    for one in node.children().filter(Node::is_element) {
        if ["sdt", "sdtContent", "customXml"]
            .iter()
            .any(|tag| one.has_tag_name((W, *tag)))
        {
            out.extend(unwrapped(one));
        } else {
            out.push(one);
        }
    }
    out
}

fn heading(style: &str) -> Option<String> {
    match style {
        "title" => Some("h1".into()),
        "subtitle" => Some("h2".into()),
        one => match one
            .strip_prefix("heading ")
            .and_then(|n| n.parse::<u8>().ok())
        {
            Some(n @ 1..=6) => Some(format!("h{n}")),
            _ => None,
        },
    }
}

fn lined(block: Node) -> String {
    let mut out = String::new();
    for one in block
        .descendants()
        .filter(|one| one.tag_name().namespace() == Some(W))
    {
        match one.tag_name().name() {
            "t" => out.push_str(&escaped(one.text().unwrap_or_default())),
            "tab" => out.push('\t'),
            "br" => out.push('\n'),
            _ => {}
        }
    }
    out
}

fn fenced(code: &mut Vec<String>) -> String {
    while code.last().is_some_and(|one| one.trim().is_empty()) {
        code.pop();
    }
    if code.is_empty() {
        return String::new();
    }
    let out = format!("<pre><code>{}</code></pre>", code.join("\n"));
    code.clear();
    out
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
        let mut code: Vec<String> = Vec::new();
        for block in unwrapped(body) {
            let paragraph = block.has_tag_name((W, "p"));
            let blank = paragraph && !code.is_empty() && self.blank(block);
            if paragraph && (blank || self.coded(block)) {
                while !lists.is_empty() {
                    shut(&mut lists, &mut out);
                }
                code.push(lined(block));
                continue;
            }
            out.push_str(&fenced(&mut code));
            let item = self.listed(block);
            let depth = item.map_or(0, |(level, _)| level + 1);
            while lists.len() > depth {
                shut(&mut lists, &mut out);
            }
            let Some((_, numbered)) = item else {
                if block.has_tag_name((W, "tbl")) {
                    out.push_str(&self.table(block));
                } else if paragraph {
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
            out.push_str(&self.runs(block, false));
        }
        while !lists.is_empty() {
            shut(&mut lists, &mut out);
        }
        out.push_str(&fenced(&mut code));
        out
    }

    fn style(&self, block: Node) -> String {
        named(block, "pPr")
            .and_then(|props| val(props, "pStyle"))
            .and_then(|id| self.styles.get(&id).cloned())
            .unwrap_or_default()
    }

    fn coded(&self, block: Node) -> bool {
        let style = self.style(block);
        if style.contains("code") || matches!(style.as_str(), "html preformatted" | "plain text") {
            return true;
        }
        let mut worded = block
            .descendants()
            .filter(|one| one.has_tag_name((W, "r")))
            .filter(|one| {
                one.children()
                    .any(|t| t.has_tag_name((W, "t")) && !t.text().unwrap_or_default().is_empty())
            })
            .peekable();
        worded.peek().is_some() && worded.all(typewritten)
    }

    fn blank(&self, block: Node) -> bool {
        lined(block).trim().is_empty()
            && !block
                .descendants()
                .any(|one| one.has_tag_name((W, "drawing")))
    }

    // The colours a document dresses its headings in are its look, not a colour chosen for some words.
    fn dressed(&self, doc: &Document) -> HashSet<String> {
        doc.descendants()
            .filter(|one| one.has_tag_name((W, "p")) && heading(&self.style(*one)).is_some())
            .flat_map(|one| one.descendants())
            .filter(|one| one.has_tag_name((W, "rPr")))
            .filter_map(|props| val(props, "color").and_then(|one| hex(&one)))
            .collect()
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
        let style = self.style(block);
        let tag = heading(&style);
        let said = self.runs(block, tag.is_some());
        if said.trim().is_empty() {
            return String::new();
        }
        if matches!(style.as_str(), "quote" | "intense quote") {
            return format!("<blockquote><p>{said}</p></blockquote>");
        }
        let tag = tag.unwrap_or_else(|| "p".to_string());
        format!("<{tag}>{said}</{tag}>")
    }

    fn table(&self, table: Node) -> String {
        let mut out = String::from("<table>");
        for (at, row) in unwrapped(table)
            .into_iter()
            .filter(|one| one.has_tag_name((W, "tr")))
            .enumerate()
        {
            let cell = if at == 0 { "th" } else { "td" };
            out.push_str("<tr>");
            for one in unwrapped(row)
                .into_iter()
                .filter(|one| one.has_tag_name((W, "tc")))
            {
                let said: Vec<String> = unwrapped(one)
                    .into_iter()
                    .filter(|p| p.has_tag_name((W, "p")))
                    .map(|p| self.runs(p, false))
                    .filter(|said| !said.trim().is_empty())
                    .collect();
                out.push_str(&format!("<{cell}>{}</{cell}>", said.join("<br>")));
            }
            out.push_str("</tr>");
        }
        out.push_str("</table>");
        out
    }

    // A heading's weight and colour are what make it a heading, so the heading alone says them.
    fn runs(&self, block: Node, headed: bool) -> String {
        let mut out = String::new();
        for one in block.children() {
            if one.has_tag_name((W, "r")) {
                out.push_str(&self.run(one, headed));
            } else if one.has_tag_name((W, "hyperlink")) {
                let words = self.runs(one, headed);
                match one.attribute((R, "id")).and_then(|id| self.targets.get(id)) {
                    Some(to) => out.push_str(&format!("<a href=\"{}\">{words}</a>", escaped(to))),
                    None => out.push_str(&words),
                }
            } else if [
                "ins",
                "smartTag",
                "sdt",
                "sdtContent",
                "fldSimple",
                "customXml",
            ]
            .iter()
            .any(|tag| one.has_tag_name((W, *tag)))
            {
                out.push_str(&self.runs(one, headed));
            }
        }
        out
    }

    fn run(&self, run: Node, headed: bool) -> String {
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
                    .is_none_or(|val| !matches!(val, "0" | "false" | "none" | "off"))
            })
        };
        let mut style = String::new();
        let ink = val(props, "color")
            .and_then(|one| hex(&one))
            .filter(|one| !headed && !self.look.contains(one));
        if let Some(color) = ink {
            style.push_str(&format!("color:#{color};"));
        }
        let lit = val(props, "highlight")
            .and_then(|one| lit(&one).map(str::to_string))
            .or_else(|| {
                named(props, "shd")
                    .and_then(|one| one.attribute((W, "fill")))
                    .and_then(hex)
                    .filter(|one| one != "FFFFFF")
                    .map(|one| format!("#{one}"))
            })
            .filter(|_| !headed);
        if let Some(lit) = lit {
            style.push_str(&format!("background-color:{lit};"));
        }
        if !style.is_empty() {
            said = format!("<span style=\"{style}\">{said}</span>");
        }
        for (tag, html) in [("strike", "s"), ("u", "u"), ("i", "em"), ("b", "strong")] {
            if on(tag) && !(headed && tag == "b") {
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
            .and_then(|id| self.targets.get(id))
            .and_then(|one| media(one));
        let alt = drawing
            .descendants()
            .find(|one| one.tag_name().name() == "docPr")
            .and_then(|one| one.attribute("descr").or_else(|| one.attribute("name")))
            .unwrap_or_default();
        match target {
            Some(name) => format!(
                "<img src=\"media/{}\" alt=\"{}\">",
                escaped(&laid(&name)),
                escaped(alt)
            ),
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
