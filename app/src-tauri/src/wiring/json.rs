const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub struct Member {
    pub whole: (usize, usize),
    pub value: (usize, usize),
}

fn blank(text: &[u8], mut i: usize) -> usize {
    loop {
        while text.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if text[i..].starts_with(b"//") {
            while i < text.len() && text[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if text[i..].starts_with(b"/*") {
            i += 2;
            while i < text.len() && !text[i..].starts_with(b"*/") {
                i += 1;
            }
            i = (i + 2).min(text.len());
            continue;
        }
        return i;
    }
}

fn quoted(text: &[u8], at: usize) -> Option<usize> {
    let mut i = at + 1;
    while i < text.len() {
        match text[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn ends(text: &[u8], at: usize) -> Option<usize> {
    let at = blank(text, at);
    match *text.get(at)? {
        b'"' => quoted(text, at),
        open @ (b'{' | b'[') => {
            let shut = if open == b'{' { b'}' } else { b']' };
            let mut deep = 0usize;
            let mut i = at;
            while i < text.len() {
                match text[i] {
                    b'"' => i = quoted(text, i)?,
                    b'/' => {
                        let past = blank(text, i);
                        i = if past > i { past } else { i + 1 };
                    }
                    c if c == open => {
                        deep += 1;
                        i += 1;
                    }
                    c if c == shut => {
                        deep -= 1;
                        i += 1;
                        if deep == 0 {
                            return Some(i);
                        }
                    }
                    _ => i += 1,
                }
            }
            None
        }
        _ => {
            let mut i = at;
            while text
                .get(i)
                .is_some_and(|c| !matches!(c, b',' | b'}' | b']') && !c.is_ascii_whitespace())
            {
                i += 1;
            }
            (i > at).then_some(i)
        }
    }
}

pub fn opens(text: &[u8]) -> Option<usize> {
    let from = if text.starts_with(BOM) { BOM.len() } else { 0 };
    let at = blank(text, from);
    (text.get(at) == Some(&b'{')).then_some(at)
}

pub fn member(text: &[u8], obj: usize, name: &str) -> Option<Member> {
    let mut i = blank(text, obj + 1);
    while text.get(i) == Some(&b'"') {
        let shut = quoted(text, i)?;
        let key = std::str::from_utf8(&text[i + 1..shut - 1]).ok()?;
        let colon = blank(text, shut);
        if text.get(colon) != Some(&b':') {
            return None;
        }
        let from = blank(text, colon + 1);
        let to = ends(text, from)?;
        if key == name {
            return Some(Member {
                whole: (i, to),
                value: (from, to),
            });
        }
        let past = blank(text, to);
        if text.get(past) != Some(&b',') {
            return None;
        }
        i = blank(text, past + 1);
    }
    None
}

/// The value as written, so a caller can read a string back out of a file serde would refuse for
/// its comments.
pub fn said(text: &str, span: (usize, usize)) -> Option<String> {
    serde_json::from_str(&text[span.0..span.1]).ok()
}

pub fn reads(text: &str, key: &str, name: &str) -> Option<String> {
    let raw = text.as_bytes();
    let under = member(raw, opens(raw)?, key)?;
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    let it = member(raw, under.value.0, name)?;
    let command = member(raw, it.value.0, "command")?;
    said(text, command.value)
}

pub fn set(text: &str, key: &str, name: &str, entry: &str) -> Option<String> {
    if text.trim().is_empty() {
        return Some(format!(
            "{{\n  \"{key}\": {{\n    \"{name}\": {entry}\n  }}\n}}\n"
        ));
    }
    let raw = text.as_bytes();
    let root = opens(raw)?;
    let Some(under) = member(raw, root, key) else {
        let pad = deeper(text, root);
        let piece = format!("\"{key}\": {{\n{pad}\"{name}\": {entry}\n}}");
        return Some(put(text, root, &piece));
    };
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    match member(raw, under.value.0, name) {
        Some(it) => Some(format!(
            "{}{entry}{}",
            &text[..it.value.0],
            &text[it.value.1..]
        )),
        None => Some(put(text, under.value.0, &format!("\"{name}\": {entry}"))),
    }
}

pub fn unset(text: &str, key: &str, name: &str) -> Option<String> {
    let raw = text.as_bytes();
    let root = opens(raw)?;
    let under = member(raw, root, key)?;
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    let it = member(raw, under.value.0, name)?;
    let past = blank(raw, it.whole.1);
    let (mut from, mut to) = if raw.get(past) == Some(&b',') {
        (it.whole.0, past + 1)
    } else {
        (before(raw, it.whole.0), it.whole.1)
    };
    while from > 0 && matches!(raw[from - 1], b' ' | b'\t') {
        from -= 1;
    }
    if from > 0 && raw[from - 1] == b'\n' {
        while matches!(raw.get(to), Some(b' ' | b'\t' | b'\r')) {
            to += 1;
        }
        if raw.get(to) == Some(&b'\n') {
            to += 1;
        }
    }
    Some(format!("{}{}", &text[..from], &text[to..]))
}

fn before(text: &[u8], at: usize) -> usize {
    let mut i = at;
    while i > 0 && text[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    if i > 0 && text[i - 1] == b',' {
        i - 1
    } else {
        at
    }
}

fn put(text: &str, obj: usize, piece: &str) -> String {
    let raw = text.as_bytes();
    let first = blank(raw, obj + 1);
    let bare = raw.get(first) == Some(&b'}');
    let pad = deeper(text, obj);
    let laid: Vec<String> = piece.lines().map(|line| format!("{pad}{line}")).collect();
    let laid = laid.join("\n");

    let mut out = String::with_capacity(text.len() + laid.len() + 4);
    out.push_str(&text[..obj + 1]);
    out.push('\n');
    out.push_str(&laid);
    if bare {
        out.push('\n');
        out.push_str(&nudge(text, obj));
        out.push_str(&text[first..]);
    } else {
        out.push(',');
        out.push_str(&text[obj + 1..]);
    }
    out
}

fn nudge(text: &str, at: usize) -> String {
    let line = text[..at].rfind('\n').map_or(0, |n| n + 1);
    text[line..at]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

fn deeper(text: &str, at: usize) -> String {
    let here = nudge(text, at);
    let step = if here.contains('\t') || text.contains("\n\t") {
        "\t"
    } else {
        "  "
    };
    format!("{here}{step}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = r#"{ "command": "C:\\Tisty\\tisty.exe", "args": ["mcp"] }"#;

    fn value(text: &str) -> serde_json::Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn a_server_lands_beside_the_ones_already_there() {
        let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"sereno\" }\n  }\n}\n";

        let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

        let read = value(&now);
        assert_eq!(read["mcpServers"]["tisty"]["args"][0], "mcp");
        assert_eq!(read["mcpServers"]["sereno"]["command"], "sereno");
    }

    #[test]
    fn everything_else_in_the_file_is_left_where_it_was() {
        let was = "{\n  \"numStartups\": 53,\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"sereno\" }\n  },\n  \"projects\": { \"d:\\\\code\": { \"trust\": true } }\n}\n";

        let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

        assert!(now.starts_with("{\n  \"numStartups\": 53,"), "{now}");
        assert!(now.contains("\"projects\": { \"d:\\\\code\": { \"trust\": true } }"));
        assert_eq!(
            now.match_indices("\"sereno\":").count(),
            1,
            "the other server was written twice"
        );
    }

    #[test]
    fn a_server_that_is_already_there_is_repointed_and_not_doubled() {
        let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"C:\\\\Old\\\\tisty.exe\", \"args\": [\"mcp\"] },\n    \"sereno\": { \"command\": \"sereno\" }\n  }\n}\n";

        let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

        assert_eq!(now.match_indices("\"tisty\"").count(), 1);
        assert_eq!(
            value(&now)["mcpServers"]["tisty"]["command"],
            "C:\\Tisty\\tisty.exe"
        );
        assert!(value(&now)["mcpServers"]["sereno"].is_object());
    }

    #[test]
    fn a_file_without_the_key_gets_it() {
        let was = "{\n  \"theme\": \"dark\"\n}\n";

        let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

        assert_eq!(value(&now)["theme"], "dark");
        assert_eq!(value(&now)["mcpServers"]["tisty"]["args"][0], "mcp");
    }

    #[test]
    fn an_empty_object_is_filled_rather_than_broken() {
        let was = "{\n  \"servers\": {}\n}\n";

        let now = set(was, "servers", "tisty", ENTRY).unwrap();

        assert_eq!(value(&now)["servers"]["tisty"]["args"][0], "mcp");
    }

    #[test]
    fn a_file_that_is_not_there_yet_is_written_whole() {
        let now = set("", "mcpServers", "tisty", ENTRY).unwrap();

        assert_eq!(value(&now)["mcpServers"]["tisty"]["args"][0], "mcp");
    }

    #[test]
    fn comments_and_tabs_survive_the_visit() {
        let was = "{\n\t// what code puts here\n\t\"servers\": {\n\t\t\"Figma\": { \"url\": \"https://x\" }\n\t},\n\t\"inputs\": []\n}\n";

        let now = set(was, "servers", "tisty", ENTRY).unwrap();

        assert!(now.contains("// what code puts here"), "{now}");
        assert!(now.contains("\n\t\t\"tisty\": {"), "{now}");
        assert!(now.contains("\"Figma\""));
    }

    #[test]
    fn a_key_that_is_not_an_object_is_refused_rather_than_replaced() {
        let was = "{\n  \"mcpServers\": \"none\"\n}\n";

        assert_eq!(set(was, "mcpServers", "tisty", ENTRY), None);
    }

    #[test]
    fn what_is_not_json_at_all_is_refused() {
        assert_eq!(set("just words", "mcpServers", "tisty", ENTRY), None);
        assert_eq!(set("[1, 2]", "mcpServers", "tisty", ENTRY), None);
    }

    #[test]
    fn the_command_is_read_back_even_through_comments() {
        let was = "{\n\t/* kept */\n\t\"servers\": {\n\t\t\"tisty\": { \"command\": \"C:\\\\Tisty\\\\tisty.exe\", \"args\": [\"mcp\"] }\n\t}\n}\n";

        assert_eq!(
            reads(was, "servers", "tisty").unwrap(),
            "C:\\Tisty\\tisty.exe"
        );
        assert_eq!(reads(was, "servers", "sereno"), None);
    }

    #[test]
    fn taking_it_out_leaves_the_neighbours_alone() {
        let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"t\" },\n    \"sereno\": { \"command\": \"s\" }\n  },\n  \"theme\": \"dark\"\n}\n";

        let now = unset(was, "mcpServers", "tisty").unwrap();

        assert!(value(&now)["mcpServers"]["tisty"].is_null());
        assert_eq!(value(&now)["mcpServers"]["sereno"]["command"], "s");
        assert_eq!(value(&now)["theme"], "dark");
    }

    #[test]
    fn taking_out_the_last_one_leaves_a_file_that_still_parses() {
        let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" },\n    \"tisty\": { \"command\": \"t\" }\n  }\n}\n";

        let now = unset(was, "mcpServers", "tisty").unwrap();

        assert_eq!(value(&now)["mcpServers"]["sereno"]["command"], "s");
        assert!(value(&now)["mcpServers"]["tisty"].is_null());
    }

    #[test]
    fn taking_out_the_only_one_leaves_an_empty_room() {
        let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"t\" }\n  }\n}\n";

        let now = unset(was, "mcpServers", "tisty").unwrap();

        assert!(value(&now)["mcpServers"].as_object().unwrap().is_empty());
    }

    #[test]
    fn taking_out_what_was_never_there_says_so() {
        let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" }\n  }\n}\n";

        assert_eq!(unset(was, "mcpServers", "tisty"), None);
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_document() {
        let was = "\u{feff}{\n  \"mcpServers\": {}\n}\n";

        let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

        assert!(now.starts_with('\u{feff}'));
        assert_eq!(
            value(now.trim_start_matches('\u{feff}'))["mcpServers"]["tisty"]["args"][0],
            "mcp"
        );
    }
}
