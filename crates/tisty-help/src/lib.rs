pub struct Page {
    pub page: &'static str,
    pub label: &'static str,
    pub section: &'static str,
    pub order: usize,
    pub twin: &'static str,
    pub body: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/pages.rs"));

pub const SCHEME: &str = "help:";

pub fn pages(spanish: bool) -> &'static [Page] {
    if spanish { ES } else { EN }
}

pub fn found(spanish: bool, slug: &str) -> Option<&'static Page> {
    pages(spanish).iter().find(|one| one.slug() == slug)
}

impl Page {
    pub fn slug(&self) -> String {
        if self.order == 0 {
            return String::new();
        }
        slugify(self.page)
    }

    pub fn title(&self) -> &'static str {
        let html = self
            .body
            .split_once("<h1")
            .and_then(|(_, rest)| rest.split_once('>'))
            .and_then(|(_, rest)| rest.split_once("</h1>"))
            .map(|(title, _)| title);
        html.or_else(|| self.body.lines().find_map(|line| line.strip_prefix("# ")))
            .map_or(self.label, str::trim)
    }

    pub fn readable(&self, spanish: bool) -> String {
        readable(self.body, spanish)
    }
}

// The same slug tisty.app gives a page, so a link here and the page online agree.
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    for one in text.chars().flat_map(char::to_lowercase) {
        let plain = match one {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        };
        match plain {
            'a'..='z' | '0'..='9' => out.push(plain),
            '.' => {}
            _ if !out.ends_with('-') => out.push('-'),
            _ => {}
        }
    }
    out.trim_matches('-').to_string()
}

fn target(spanish: bool, page: &str, anchor: Option<&str>) -> Option<String> {
    let page = page.trim().replace(' ', "-");
    let known = pages(spanish).iter().find(|one| one.page == page)?;
    Some(match anchor {
        Some(anchor) => format!("{SCHEME}{}#{anchor}", known.slug()),
        None => format!("{SCHEME}{}", known.slug()),
    })
}

fn wiki_links(body: &str, spanish: bool) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("[[") {
        out.push_str(&rest[..at]);
        let Some(end) = rest[at + 2..].find("]]") else {
            out.push_str(&rest[at..]);
            return out;
        };
        let raw = &rest[at + 2..at + 2 + end];
        let (label, aimed) = raw.split_once('|').unwrap_or((raw, raw));
        let (page, anchor) = aimed
            .split_once('#')
            .map_or((aimed, None), |(page, anchor)| (page, Some(anchor)));
        match target(spanish, page, anchor) {
            Some(url) => out.push_str(&format!("[{}]({url})", label.trim())),
            None => out.push_str(label.trim()),
        }
        rest = &rest[at + 2 + end + 2..];
    }
    out.push_str(rest);
    out
}

fn page_links(body: &str, spanish: bool) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..at + 2]);
        let after = &rest[at + 2..];
        let end = after.find(')').unwrap_or(after.len());
        let link = &after[..end];
        let (page, anchor) = link
            .split_once('#')
            .map_or((link, None), |(page, anchor)| (page, Some(anchor)));
        let local = !page.is_empty() && !page.contains([':', '/', ' ']);
        match local.then(|| target(spanish, page, anchor)).flatten() {
            Some(url) => out.push_str(&url),
            None => out.push_str(link),
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!("{name}=\""))? + name.len() + 2;
    tag[at..].split('"').next()
}

// A picture written as HTML becomes a Markdown one, so the window decides how to show it, here or not.
fn html_pictures(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("<img ") {
        out.push_str(&rest[..at]);
        let end = rest[at..].find('>').map_or(rest.len(), |end| at + end + 1);
        let tag = &rest[at..end];
        if let Some(src) = attribute(tag, "src") {
            out.push_str(&format!(
                "![{}]({src})",
                attribute(tag, "alt").unwrap_or_default()
            ));
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

// A home page opens with an HTML banner down to its first rule, which the window draws its own way.
fn unbannered(body: &str) -> &str {
    let body = body.trim_start_matches('\u{feff}');
    if let Some(rest) = body.strip_prefix("# ") {
        return rest.split_once('\n').map_or("", |(_, rest)| rest);
    }
    if !body.trim_start().starts_with('<') {
        return body;
    }
    match (body.find("\n---"), body.find("\n#")) {
        (Some(rule), heading) if heading.is_none_or(|heading| rule < heading) => &body[rule + 4..],
        _ => body,
    }
}

pub fn readable(body: &str, spanish: bool) -> String {
    let body = unbannered(body);
    page_links(&wiki_links(&html_pictures(body), spanish), spanish)
}

fn pictured(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg"]
        .iter()
        .any(|one| path.ends_with(one))
}

pub fn weight(url: &str) -> Option<u64> {
    WEIGHS
        .iter()
        .find(|(one, _)| *one == url)
        .map(|(_, bytes)| *bytes)
}

pub fn pictures(spanish: bool) -> Vec<String> {
    let mut all: Vec<String> = pages(spanish)
        .iter()
        .flat_map(|one| {
            let body = html_pictures(one.body);
            body.split("![")
                .skip(1)
                .filter_map(|rest| {
                    rest.split_once("](")
                        .map(|(_, url)| url.split(')').next().unwrap_or_default().to_string())
                })
                .filter(|url| url.starts_with("https://") && pictured(url))
                .collect::<Vec<_>>()
        })
        .collect();
    all.sort();
    all.dedup();
    all
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
