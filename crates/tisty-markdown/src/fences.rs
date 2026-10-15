/// Steps over fenced blocks line by line, for anything that reads a body and must not read code.
pub fn fencing() -> impl FnMut(&str) -> bool {
    let mut fence = Fencing::default();
    move |line| fence.inside(line)
}

pub fn ends_fenced(text: &str) -> bool {
    let mut fence = Fencing::default();
    for line in text.lines() {
        fence.inside(line);
    }
    fence.open.is_some()
}

pub fn fenced_spans(text: &str) -> Vec<(usize, usize)> {
    let mut fence = Fencing::default();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        if fence.inside(line.trim_end_matches(['\n', '\r'])) {
            match out.last_mut() {
                Some(last) if last.1 == at => last.1 = end,
                _ => out.push((at, end)),
            }
        }
        at = end;
    }
    out
}

// The quote prefix comes off first, or a fence written inside a quote is never seen.
#[derive(Default)]
pub struct Fencing {
    pub open: Option<(char, usize, usize, usize)>,
    pub base: usize,
    pub told: bool,
}

pub fn bullet(said: &str) -> Option<usize> {
    let bytes = said.as_bytes();
    let mut at = 0;
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
        at = 1;
    } else {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        if at == 0 || at > 9 || !matches!(bytes.get(at), Some(b'.' | b')')) {
            return None;
        }
        at += 1;
    }
    let gap = said[at..]
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .count();
    (gap > 0).then_some(at + gap)
}

pub fn listed(base: usize, wide: usize, said: &str) -> usize {
    if said.is_empty() {
        return base;
    }
    match bullet(said) {
        Some(after) if wide <= base => wide + after,
        _ if wide < base => 0,
        _ => base,
    }
}

pub fn nameless(said: &str) -> String {
    let mut from = 0;
    while let Some(found) = said[from..].find("title=\"") {
        let at = from + found;
        from = at + 7;
        if said[..at]
            .chars()
            .next_back()
            .is_some_and(|one| one.is_alphanumeric() || one == '_')
        {
            continue;
        }
        let rest = &said[from..];
        let bytes = rest.as_bytes();
        let mut over = 0;
        let end = loop {
            match bytes.get(over) {
                None => return said.to_string(),
                Some(b'\\') => over += 2,
                Some(b'"') => break over,
                _ => over += 1,
            }
        };
        return format!("{} {}", &said[..at], &rest[end + 1..]);
    }
    said.to_string()
}

fn spacing(said: &str) -> usize {
    said.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

pub fn quoted(line: &str) -> (usize, usize, &str) {
    let mut said = line;
    let mut deep = 0;
    let mut wide = spacing(said);

    while wide < 4 {
        let Some(rest) = said.trim_start().strip_prefix('>') else {
            break;
        };
        said = rest.strip_prefix(' ').unwrap_or(rest);
        deep += 1;
        wide = spacing(said);
    }
    (deep, wide, said.trim())
}

pub fn quoteless(line: &str) -> &str {
    quoted(line).2
}

impl Fencing {
    pub fn inside(&mut self, line: &str) -> bool {
        let (deep, wide, said) = quoted(line);
        if self.open.is_none() {
            self.base = listed(self.base, wide, said);
        }
        let (wide, said) = match bullet(said) {
            Some(after) => (wide + after, &said[after..]),
            None => (wide, said),
        };
        self.told = false;
        let mut marker = None;
        for mark in ['`', '~'] {
            let many = said.chars().take_while(|c| *c == mark).count();
            if many < 3 || wide >= self.base + 4 {
                continue;
            }
            let after = &said[many.min(said.len())..];
            if mark == '`' && after.contains('`') {
                self.told = self.open.is_none();
                continue;
            }
            marker = Some((mark, many));
            break;
        }

        if let Some((open, was, held, room)) = self.open {
            if said.is_empty() {
                return true;
            }
            if deep >= held && wide >= room {
                if let Some((mark, many)) = marker
                    && mark == open
                    && many >= was
                    && deep == held
                {
                    self.open = None;
                }
                return true;
            }
            self.open = None;
        }

        match marker {
            Some((mark, many)) => {
                self.told = nameless(&said[many..]).split_whitespace().count() > 1;
                self.open = Some((mark, many, deep, wide));
                true
            }
            None => false,
        }
    }
}
