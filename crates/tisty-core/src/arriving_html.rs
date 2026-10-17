use htmd::element_handler::{HandlerResult, Handlers};
use htmd::options::{BulletListMarker, HrStyle, Options};
use htmd::{Element, HtmlToMarkdown};
use markup5ever_rcdom::{Node, NodeData};
use std::rc::Rc;

const SKIPPED: [&str; 11] = [
    "script", "style", "head", "noscript", "template", "iframe", "object", "embed", "button",
    "select", "textarea",
];

pub fn from_html(html: &str) -> String {
    let html = broken(html);
    HtmlToMarkdown::builder()
        .options(Options {
            bullet_list_marker: BulletListMarker::Dash,
            ul_bullet_spacing: 1,
            ol_number_spacing: 1,
            hr_style: HrStyle::Dashes,
            ..Options::default()
        })
        .skip_tags(SKIPPED.to_vec())
        .add_handler(vec!["mark"], marked)
        .add_handler(vec!["u", "ins"], underlined)
        .add_handler(vec!["span", "font"], painted)
        .add_handler(vec!["details"], folded)
        .add_handler(vec!["b"], bolded)
        .add_handler(vec!["img"], pictured)
        .add_handler(vec!["hr"], |_: &dyn Handlers, _: Element| {
            Some(HandlerResult::from("\n\n---\n\n"))
        })
        .add_handler(vec!["summary"], |_: &dyn Handlers, _: Element| {
            Some(HandlerResult::from(""))
        })
        .build()
        .convert(&html)
        .unwrap_or_default()
}

const TURNS: [&str; 4] = [
    "page-break-before:always",
    "page-break-after:always",
    "break-before:page",
    "break-after:page",
];

fn named(rest: &str, name: &str) -> bool {
    let bytes = rest.as_bytes();
    bytes
        .get(..name.len())
        .is_some_and(|said| said.eq_ignore_ascii_case(name.as_bytes()))
        && bytes
            .get(name.len())
            .is_some_and(|next| *next == b'>' || *next == b'/' || next.is_ascii_whitespace())
}

// The element ends at its own closing tag, past any of the same name opened inside it.
fn closing(tag: &str, after: &str) -> Option<usize> {
    let name: String = tag[1..]
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
    if name.is_empty() || tag.ends_with("/>") {
        return None;
    }
    let mut inside = 0usize;
    let mut at = 0;
    while let Some(open) = after[at..].find('<') {
        let start = at + open;
        let end = start + after[start..].find('>')? + 1;
        let rest = &after[start + 1..];
        if let Some(shut) = rest.strip_prefix('/') {
            if named(shut, &name) {
                if inside == 0 {
                    return Some(end);
                }
                inside -= 1;
            }
        } else if named(rest, &name) && !after[start..end].ends_with("/>") {
            inside += 1;
        }
        at = end;
    }
    None
}

// Word and the web end a sheet with a style on an element; the language ends it with a rule.
fn broken(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        let Some(shut) = rest[open..].find('>').map(|at| open + at + 1) else {
            break;
        };
        let tag = &rest[open..shut];
        let flat = tag
            .to_ascii_lowercase()
            .replace([' ', '\t', '\n', '\r'], "");
        let turn = TURNS.iter().position(|one| flat.contains(one));
        out.push_str(&rest[..open]);
        match turn {
            Some(at) if at % 2 == 0 => {
                out.push_str("<hr>");
                out.push_str(tag);
                rest = &rest[shut..];
            }
            Some(_) => {
                let ends = closing(tag, &rest[shut..]).map_or(shut, |at| shut + at);
                out.push_str(&rest[open..ends]);
                out.push_str("<hr>");
                rest = &rest[ends..];
            }
            None => {
                out.push_str(tag);
                rest = &rest[shut..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn attr(element: &Element, name: &str) -> Option<String> {
    element
        .attrs
        .iter()
        .find(|one| &*one.name.local == name)
        .map(|one| one.value.to_string())
}

fn inner(handlers: &dyn Handlers, element: &Element) -> String {
    handlers.walk_children(element.node).content
}

// Highlighted code paints its words with the same tags, and a mark inside code would change the code.
fn coded(node: &Rc<Node>) -> bool {
    within(node, &["pre", "code"])
}

fn within(node: &Rc<Node>, tags: &[&str]) -> bool {
    let mut at = node.clone();
    loop {
        let up = at.parent.take();
        at.parent.set(up.clone());
        let Some(up) = up.and_then(|one| one.upgrade()) else {
            return false;
        };
        if matches!(&up.data, NodeData::Element { name, .. } if tags.contains(&&*name.local)) {
            return true;
        }
        at = up;
    }
}

// A space just inside a tag belongs to the sentence, not to the mark, or the words around it run together.
fn wrapped(raw: &str, open: &str, close: &str) -> HandlerResult {
    let words = raw.trim();
    if words.is_empty() {
        return raw.to_string().into();
    }
    let lead = &raw[..raw.len() - raw.trim_start().len()];
    let trail = &raw[raw.trim_end().len()..];
    format!("{lead}{open}{words}{close}{trail}").into()
}

fn pen(raw: &str, hue: Option<&str>) -> HandlerResult {
    match hue {
        None | Some("yellow") => wrapped(raw, "==", "=="),
        Some(hue) => wrapped(raw, "==", &format!("=={{{hue}}}")),
    }
}

fn marked(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let words = inner(handlers, &element);
    if coded(element.node) {
        return Some(words.into());
    }
    let hue = attr(&element, "data-pen")
        .filter(|one| tisty_markdown::vocab::pen(one))
        .or_else(|| {
            styled(&element, &["background-color", "background"]).and_then(|one| nearest(one, true))
        });
    Some(pen(&words, hue.as_deref()))
}

fn underlined(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let words = inner(handlers, &element);
    Some(match coded(element.node) {
        true => words.into(),
        false => wrapped(&words, "++", "++"),
    })
}

// Pasted pages colour words with `style` or `<font color>`; the language has a pen of the nearest palette colour.
fn painted(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    if coded(element.node) {
        return handlers.fallback(element);
    }
    let style = attr(&element, "style")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .replace(' ', "");
    let heavy = [
        "font-weight:700",
        "font-weight:bold",
        "font-weight:600",
        "font-weight:800",
    ]
    .iter()
    .any(|one| style.contains(one));
    let slanted = style.contains("font-style:italic");
    // A link's own colour is how a page shows it is a link, not a colour anyone chose for its words.
    let hue = match within(element.node, &["a"]) {
        true => None,
        false => styled(&element, &["background-color", "background"])
            .and_then(|one| nearest(one, true))
            .or_else(|| {
                styled(&element, &["color"])
                    .or_else(|| attr(&element, "color").and_then(|one| rgb(&one)))
                    .and_then(|one| nearest(one, false))
            }),
    };
    if !heavy && !slanted && hue.is_none() {
        return handlers.fallback(element);
    }
    let mut out = inner(handlers, &element);
    if slanted {
        out = wrapped(&out, "*", "*").content;
    }
    if heavy {
        out = wrapped(&out, "**", "**").content;
    }
    Some(match hue {
        Some(hue) => pen(&out, Some(&hue)),
        None => out.into(),
    })
}

// Google Docs wraps a whole copy in a bold tag that says it is not bold.
fn bolded(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let style = attr(&element, "style").unwrap_or_default().replace(' ', "");
    if style.contains("font-weight:normal") || style.contains("font-weight:400") {
        return Some(inner(handlers, &element).into());
    }
    handlers.fallback(element)
}

// A picture written into the page itself would put megabytes of letters in the text.
fn pictured(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    match attr(&element, "src").is_some_and(|one| one.trim_start().starts_with("data:")) {
        true => Some(attr(&element, "alt").unwrap_or_default().into()),
        false => handlers.fallback(element),
    }
}

fn folded(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let title = element
        .node
        .children
        .borrow()
        .iter()
        .find(
            |one| matches!(&one.data, NodeData::Element { name, .. } if &*name.local == "summary"),
        )
        .map(|one| handlers.walk_children(one).content)
        .unwrap_or_default();
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let body = inner(handlers, &element);
    let body = body.trim();
    Some(format!("\n\n:::details {title}\n\n{body}\n\n:::\n\n").into())
}

fn styled(element: &Element, names: &[&str]) -> Option<(u8, u8, u8)> {
    let style = attr(element, "style")?;
    style.split(';').find_map(|one| {
        let (name, value) = one.split_once(':')?;
        names
            .contains(&name.trim().to_ascii_lowercase().as_str())
            .then(|| rgb(value).or_else(|| value.split_whitespace().find_map(rgb)))
            .flatten()
    })
}

fn rgb(said: &str) -> Option<(u8, u8, u8)> {
    let said = said.trim().trim_end_matches(';').to_ascii_lowercase();
    if let Some(hex) = said.strip_prefix('#') {
        let wide = |at: usize, len: usize| u8::from_str_radix(hex.get(at..at + len)?, 16).ok();
        return match hex.len() {
            3 | 4 => Some((wide(0, 1)? * 17, wide(1, 1)? * 17, wide(2, 1)? * 17)),
            6 | 8 => Some((wide(0, 2)?, wide(2, 2)?, wide(4, 2)?)),
            _ => None,
        };
    }
    if let Some(inside) = said
        .strip_prefix("rgba(")
        .or_else(|| said.strip_prefix("rgb("))
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inside
            .split([',', ' ', '/'])
            .filter(|one| !one.is_empty())
            .collect();
        let channel = |one: &str| match one.strip_suffix('%') {
            Some(share) => share.parse::<f32>().ok().map(|n| n * 2.55),
            None => one.parse::<f32>().ok(),
        };
        let gone = parts
            .get(3)
            .and_then(|one| channel(one))
            .is_some_and(|alpha| alpha == 0.0);
        if gone || parts.len() < 3 {
            return None;
        }
        let at = |n: usize| channel(parts[n]).map(|one| one.clamp(0.0, 255.0) as u8);
        return Some((at(0)?, at(1)?, at(2)?));
    }
    NAMED
        .iter()
        .find(|(name, _)| *name == said)
        .map(|(_, one)| *one)
}

const NAMED: [(&str, (u8, u8, u8)); 12] = [
    ("red", (255, 0, 0)),
    ("orange", (255, 165, 0)),
    ("yellow", (255, 255, 0)),
    ("gold", (255, 215, 0)),
    ("green", (0, 128, 0)),
    ("lime", (0, 255, 0)),
    ("teal", (0, 128, 128)),
    ("blue", (0, 0, 255)),
    ("navy", (0, 0, 128)),
    ("purple", (128, 0, 128)),
    ("pink", (255, 192, 203)),
    ("brown", (165, 42, 42)),
];

// Each pen by the hue a person would call it; pale and deep shades of one colour share its hue.
const PALETTE: [(&str, f32); 10] = [
    ("red", 0.0),
    ("orange", 25.0),
    ("amber", 38.0),
    ("yellow", 52.0),
    ("green", 142.0),
    ("teal", 173.0),
    ("blue", 217.0),
    ("indigo", 239.0),
    ("purple", 271.0),
    ("pink", 330.0),
];

// Black, white and greys are how text is set, not a colour anyone chose to mark it with.
fn nearest((r, g, b): (u8, u8, u8), lit: bool) -> Option<String> {
    let most = r.max(g).max(b) as i32;
    let least = r.min(g).min(b) as i32;
    if most - least < if lit { 24 } else { 48 } || (lit && least > 245) {
        return None;
    }
    let hue = hue_of(r, g, b);
    let light = (most + least) as f32 / 510.0;
    if (12.0..48.0).contains(&hue) && light < 0.32 {
        return Some("brown".to_string());
    }
    PALETTE
        .iter()
        .min_by(|(_, a), (_, b)| apart(hue, *a).total_cmp(&apart(hue, *b)))
        .map(|(name, _)| name.to_string())
}

fn apart(one: f32, other: f32) -> f32 {
    let gap = (one - other).abs() % 360.0;
    gap.min(360.0 - gap)
}

fn hue_of(r: u8, g: u8, b: u8) -> f32 {
    let (r, g, b) = (r as f32, g as f32, b as f32);
    let most = r.max(g).max(b);
    let span = most - r.min(g).min(b);
    let raw = if most == r {
        ((g - b) / span).rem_euclid(6.0)
    } else if most == g {
        (b - r) / span + 2.0
    } else {
        (r - g) / span + 4.0
    };
    raw * 60.0
}
