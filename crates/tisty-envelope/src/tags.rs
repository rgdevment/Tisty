/// One opening or closing tag of HTML this crate's own translator wrote, or that ammonia wrote back: quotes always double, values escaped.
pub struct Tag<'a> {
    pub name: &'a str,
    pub closes: bool,
    pub attributes: Vec<(&'a str, &'a str)>,
    pub length: usize,
}

impl<'a> Tag<'a> {
    pub fn read(html: &'a str) -> Option<Self> {
        let inner = html.strip_prefix('<')?;
        let (closes, inner) = match inner.strip_prefix('/') {
            Some(rest) => (true, rest),
            None => (false, inner),
        };
        if !inner.starts_with(|one: char| one.is_ascii_alphabetic()) {
            return None;
        }
        let end = inner.find(|one: char| one.is_ascii_whitespace() || one == '/' || one == '>')?;
        let name = &inner[..end];
        let mut rest = &inner[end..];
        let mut attributes = Vec::new();
        loop {
            rest = rest.trim_start_matches(|one: char| one.is_ascii_whitespace() || one == '/');
            if let Some(after) = rest.strip_prefix('>') {
                rest = after;
                break;
            }
            let end =
                rest.find(|one: char| one.is_ascii_whitespace() || one == '=' || one == '>')?;
            let said = &rest[..end];
            rest = &rest[end..];
            match rest.strip_prefix("=\"") {
                Some(after) => {
                    let close = after.find('"')?;
                    attributes.push((said, &after[..close]));
                    rest = &after[close + 1..];
                }
                None => attributes.push((said, "")),
            }
        }
        Some(Self {
            name,
            closes,
            attributes,
            length: html.len() - rest.len(),
        })
    }

    pub fn said(&self, name: &str) -> Option<&'a str> {
        self.attributes
            .iter()
            .find(|(said, _)| *said == name)
            .map(|(_, value)| *value)
    }
}

pub fn unescaped(said: &str) -> String {
    said.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

pub fn escaped(said: &str) -> String {
    said.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Every tag in the HTML, in order, each with the words before it.
pub fn walk<'a>(html: &'a str, mut each: impl FnMut(&'a str, Option<Tag<'a>>)) {
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        each(&rest[..start], None);
        rest = &rest[start..];
        match Tag::read(rest) {
            Some(tag) => {
                rest = &rest[tag.length..];
                each("", Some(tag));
            }
            None => {
                each(&rest[..1], None);
                rest = &rest[1..];
            }
        }
    }
    each(rest, None);
}
