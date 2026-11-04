/// One opening or closing tag of HTML. Most come from this crate's translator or from ammonia, with double quotes; anything else a browser reads as a tag is read as one here too.
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
            // No name before an equals sign reads as no tag, rather than as going round here for ever.
            if end == 0 {
                return None;
            }
            let said = &rest[..end];
            rest = rest[end..].trim_start_matches(|one: char| one.is_ascii_whitespace());
            let Some(value) = rest.strip_prefix('=') else {
                attributes.push((said, ""));
                continue;
            };
            let value = value.trim_start_matches(|one: char| one.is_ascii_whitespace());
            match value.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let inner = &value[1..];
                    let close = inner.find(quote)?;
                    attributes.push((said, &inner[..close]));
                    rest = &inner[close + 1..];
                }
                _ => {
                    let end = value.find(|one: char| one.is_ascii_whitespace() || one == '>')?;
                    attributes.push((said, &value[..end]));
                    rest = &value[end..];
                }
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
