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

// A bullet with nothing after it still opens an item, whose words start one column past the mark.
fn emptied(said: &str) -> Option<usize> {
    let numbered = said.len() > 1
        && said.len() <= 10
        && said[..said.len() - 1]
            .bytes()
            .all(|one| one.is_ascii_digit())
        && said.ends_with(['.', ')']);
    (matches!(said, "-" | "*" | "+") || numbered).then_some(said.len() + 1)
}

pub fn listed(base: usize, wide: usize, said: &str) -> usize {
    if said.is_empty() {
        return base;
    }
    match bullet(said).or_else(|| emptied(said)) {
        Some(after) if wide <= base => wide + after,
        _ if wide < base => 0,
        _ => base,
    }
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
        let mut marker = None;
        for mark in ['`', '~'] {
            let many = said.chars().take_while(|c| *c == mark).count();
            if many < 3 || wide >= self.base + 4 {
                continue;
            }
            let after = &said[many.min(said.len())..];
            if mark == '`' && after.contains('`') {
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
                self.open = Some((mark, many, deep, self.base.min(wide)));
                true
            }
            None => false,
        }
    }
}
