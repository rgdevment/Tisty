pub(super) fn unpathed(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = absolute(rest) {
        out.push_str(&rest[..at]);
        out.push_str("[path]");
        let path = &rest[at..];
        rest = &path[path_end(path, rest[..at].chars().next_back())..];
    }
    out.push_str(rest);
    out
}

fn path_end(path: &str, opened: Option<char>) -> usize {
    let closer = match opened {
        Some(quote @ ('`' | '"' | '\'')) => Some(quote),
        Some('«') => Some('»'),
        Some('(') => Some(')'),
        _ => None,
    };
    if let Some(closer) = closer {
        return path.find(closer).unwrap_or(path.len());
    }
    let sep = match path.as_bytes().get(2) {
        Some(b'\\') => '\\',
        _ => '/',
    };
    let reach = match sep {
        '/' => path.find("://").unwrap_or(path.len()),
        _ => path.len(),
    };
    let reach = named_file_end(&path[..reach]).unwrap_or(reach);
    let marks: Vec<(usize, char)> = path[..reach].char_indices().collect();
    let last = marks
        .windows(3)
        .rev()
        .find(|w| w[1].1 == sep && !w[0].1.is_whitespace() && !w[2].1.is_whitespace())
        .map_or(0, |w| w[1].0);
    let tail = &path[last..];
    last + tail
        .char_indices()
        .find(|&(_, c)| c.is_whitespace() || "`\"')]>,;»".contains(c))
        .map_or(tail.len(), |(end, _)| end)
}

pub(super) fn absolute(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len()).find(|&at| {
        // A drive is one letter: without this, the "s" of "https://" reads as one and the rest
        // of the line goes with it.
        let alone = at == 0 || !bytes[at - 1].is_ascii_alphanumeric();
        let drive = alone
            && at + 2 < bytes.len()
            && bytes[at].is_ascii_alphabetic()
            && bytes[at + 1] == b':'
            && (bytes[at + 2] == b'/' || bytes[at + 2] == b'\\');
        let rooted = bytes[at] == b'/'
            && at > 0
            && bytes[at - 1] == b' '
            && bytes
                .get(at + 1)
                .is_some_and(|next| !next.is_ascii_whitespace());
        drive || rooted
    })
}

fn named_file_end(path: &str) -> Option<usize> {
    let marks: Vec<(usize, char)> = path.char_indices().collect();
    for (at, &(dot, c)) in marks.iter().enumerate() {
        if c != '.' || at == 0 || marks[at - 1].1.is_whitespace() {
            continue;
        }
        let ext = marks[at + 1..]
            .iter()
            .take_while(|(_, one)| one.is_ascii_alphanumeric())
            .count();
        if !(1..=8).contains(&ext) {
            continue;
        }
        let end = at + 1 + ext;
        let closed = marks
            .get(end)
            .is_none_or(|&(_, next)| next.is_whitespace() || "`\"')]>,;»".contains(next));
        if closed && path[..dot].contains(['/', '\\']) {
            return Some(marks.get(end).map_or(path.len(), |&(at, _)| at));
        }
    }
    None
}
