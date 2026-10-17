use htmd::element_handler::{HandlerResult, Handlers};
use htmd::options::{BulletListMarker, HrStyle, Options};
use htmd::{Element, HtmlToMarkdown};
use markup5ever_rcdom::NodeData;

const SKIPPED: [&str; 12] = [
    "script", "style", "head", "noscript", "template", "iframe", "object", "embed", "form",
    "button", "select", "textarea",
];

pub fn from_html(html: &str) -> String {
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
        .add_handler(vec!["summary"], |_: &dyn Handlers, _: Element| {
            Some(HandlerResult::from(""))
        })
        .build()
        .convert(html)
        .unwrap_or_default()
}

fn attr(element: &Element, name: &str) -> Option<String> {
    element
        .attrs
        .iter()
        .find(|one| &*one.name.local == name)
        .map(|one| one.value.to_string())
}

fn inner(handlers: &dyn Handlers, element: &Element) -> String {
    handlers
        .walk_children(element.node)
        .content
        .trim()
        .to_string()
}

fn pen(words: &str, hue: Option<&str>) -> HandlerResult {
    if words.is_empty() {
        return "".into();
    }
    match hue {
        None | Some("yellow") => format!("=={words}==").into(),
        Some(hue) => format!("=={words}=={{{hue}}}").into(),
    }
}

fn marked(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let words = inner(handlers, &element);
    let hue = attr(&element, "data-pen")
        .filter(|one| tisty_markdown::vocab::pen(one))
        .or_else(|| {
            styled(&element, &["background-color", "background"]).and_then(|one| nearest(one, true))
        });
    Some(pen(&words, hue.as_deref()))
}

fn underlined(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let words = inner(handlers, &element);
    Some(match words.is_empty() {
        true => "".into(),
        false => format!("++{words}++").into(),
    })
}

// Pasted pages colour words with `style` or `<font color>`; the language has a pen of the nearest palette colour.
fn painted(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
    let lit =
        styled(&element, &["background-color", "background"]).and_then(|one| nearest(one, true));
    let inked = || {
        styled(&element, &["color"])
            .or_else(|| attr(&element, "color").and_then(|one| rgb(&one)))
            .and_then(|one| nearest(one, false))
    };
    match lit.or_else(inked) {
        Some(hue) => Some(pen(&inner(handlers, &element), Some(&hue))),
        None => handlers.fallback(element),
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
    Some(format!("\n\n:::details {title}\n\n{body}\n\n:::\n\n").into())
}

fn styled(element: &Element, names: &[&str]) -> Option<(u8, u8, u8)> {
    let style = attr(element, "style")?;
    style.split(';').find_map(|one| {
        let (name, value) = one.split_once(':')?;
        names
            .contains(&name.trim().to_ascii_lowercase().as_str())
            .then(|| value.split_whitespace().find_map(rgb))
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
        let mut parts = inside
            .split([',', ' ', '/'])
            .filter(|one| !one.is_empty())
            .map(|one| one.parse::<f32>().ok().map(|n| n.clamp(0.0, 255.0) as u8));
        return Some((parts.next()??, parts.next()??, parts.next()??));
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

const PALETTE: [(&str, (u8, u8, u8)); 11] = [
    ("yellow", (250, 204, 21)),
    ("red", (239, 68, 68)),
    ("orange", (249, 115, 22)),
    ("amber", (245, 158, 11)),
    ("green", (34, 197, 94)),
    ("teal", (20, 184, 166)),
    ("blue", (59, 130, 246)),
    ("indigo", (99, 102, 241)),
    ("purple", (168, 85, 247)),
    ("pink", (236, 72, 153)),
    ("brown", (146, 64, 14)),
];

// Black, white and greys are how text is set, not a colour anyone chose to mark it with.
fn nearest((r, g, b): (u8, u8, u8), lit: bool) -> Option<String> {
    let most = r.max(g).max(b) as i32;
    let least = r.min(g).min(b) as i32;
    if most - least < if lit { 24 } else { 48 } || (lit && least > 245) {
        return None;
    }
    PALETTE
        .iter()
        .min_by_key(|(_, (pr, pg, pb))| {
            let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
            d(r, *pr) + d(g, *pg) + d(b, *pb)
        })
        .map(|(name, _)| name.to_string())
}
