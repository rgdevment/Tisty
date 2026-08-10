//! References are written in prose and pulled back out of it, never kept as a
//! field: what reaches the file is ordinary Markdown, readable without Tisty.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// `[[name]]` — something inside Tisty: a document, a ticket, a task.
    Doc,
    /// `[label](url)` or a bare address.
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ref {
    pub kind: Kind,
    pub target: String,
    pub label: Option<String>,
}

/// In order of appearance, without repeats; code spans are read as text.
pub fn extract(text: &str) -> Vec<Ref> {
    let mut found: Vec<Ref> = Vec::new();
    let mut keep = |one: Ref| {
        if !found
            .iter()
            .any(|held| held.target == one.target && held.kind == one.kind)
        {
            found.push(one);
        }
    };

    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let rest = &text[at..];
        at = match bytes[at] {
            b'`' => past_code(text, at),
            b'[' if rest.starts_with("[[") => named(rest, at, &mut keep),
            b'[' => linked(text, at, &mut keep),
            b'h' if rest.starts_with("http://") || rest.starts_with("https://") => {
                bare(rest, at, &mut keep)
            }
            _ => at + next_char(bytes, at),
        };
    }
    found
}

fn next_char(bytes: &[u8], at: usize) -> usize {
    let mut step = 1;
    while at + step < bytes.len() && bytes[at + step] & 0xC0 == 0x80 {
        step += 1;
    }
    step
}

/// An unclosed run is just backticks; only a matching run closes a code span.
fn past_code(text: &str, at: usize) -> usize {
    let fence = text[at..].bytes().take_while(|b| *b == b'`').count();
    let mut from = at + fence;
    while let Some(next) = text[from..].find('`') {
        let start = from + next;
        let run = text[start..].bytes().take_while(|b| *b == b'`').count();
        if run == fence {
            return start + run;
        }
        from = start + run;
    }
    at + fence
}

fn named(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let Some(end) = rest.find("]]") else {
        return at + 2;
    };
    let name = rest[2..end].trim();
    if !name.is_empty() && !name.contains(['\n', '[']) {
        keep(Ref {
            kind: Kind::Doc,
            target: name.to_string(),
            label: None,
        });
    }
    at + end + 2
}

fn linked(text: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let rest = &text[at + 1..];
    let Some(shut) = rest.find(']') else {
        return at + 1;
    };
    let label = &rest[..shut];
    if label.contains(['\n', '[']) {
        return at + 1;
    }

    let after = at + 1 + shut + 1;
    if text.as_bytes().get(after) != Some(&b'(') {
        return at + 1;
    }
    let tail = &text[after + 1..];
    let Some(close) = tail.find(')') else {
        return at + 1;
    };

    // A Markdown target may carry a title: [a](https://x "why").
    let target = tail[..close].split_whitespace().next().unwrap_or("");
    if !target.is_empty() {
        let label = label.trim();
        keep(Ref {
            kind: Kind::Link,
            target: target.to_string(),
            label: (!label.is_empty()).then(|| label.to_string()),
        });
    }
    after + 1 + close + 1
}

fn bare(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '<' || c == '>')
        .unwrap_or(rest.len());
    let url = unpunctuated(&rest[..end]);

    // The scheme alone points nowhere.
    if url.len() > url.find("//").map_or(0, |n| n + 2) {
        keep(Ref {
            kind: Kind::Link,
            target: url.to_string(),
            label: None,
        });
    }
    at + end
}

/// Prose punctuation clings to the end of an address; a closing bracket only
/// belongs to the address when something opened it.
fn unpunctuated(url: &str) -> &str {
    let mut end = url.len();
    while let Some(last) = url[..end].chars().next_back() {
        let prose = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' | ']' | '}' => true,
            ')' => url[..end].matches(')').count() > url[..end].matches('(').count(),
            _ => false,
        };
        if !prose {
            break;
        }
        end -= last.len_utf8();
    }
    &url[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(text: &str) -> Vec<String> {
        extract(text).into_iter().map(|one| one.target).collect()
    }

    #[test]
    fn the_prose_around_a_reference_is_what_gives_it_meaning() {
        let found = extract(
            "se corrigió en el ticket [[CUSLEG-3465]], MR en [gitlab](https://gl.example/mr/7)",
        );

        assert_eq!(
            found,
            vec![
                Ref {
                    kind: Kind::Doc,
                    target: "CUSLEG-3465".into(),
                    label: None
                },
                Ref {
                    kind: Kind::Link,
                    target: "https://gl.example/mr/7".into(),
                    label: Some("gitlab".into())
                },
            ]
        );
    }

    #[test]
    fn a_link_is_read_once_and_not_again_as_a_bare_address() {
        assert_eq!(
            targets("[the ticket](https://x.example/1)"),
            ["https://x.example/1"]
        );
    }

    #[test]
    fn an_address_written_plainly_still_counts() {
        assert_eq!(
            targets("mirar https://x.example/1 antes"),
            ["https://x.example/1"]
        );
    }

    #[test]
    fn prose_punctuation_is_not_part_of_the_address() {
        assert_eq!(
            targets("está en https://x.example/1."),
            ["https://x.example/1"]
        );
        assert_eq!(
            targets("(ver https://x.example/1)"),
            ["https://x.example/1"]
        );
        assert_eq!(targets("¿en https://x.example/1?"), ["https://x.example/1"]);
    }

    #[test]
    fn a_bracket_the_address_opened_belongs_to_it() {
        assert_eq!(
            targets("https://en.example.org/wiki/Foo_(bar)"),
            ["https://en.example.org/wiki/Foo_(bar)"]
        );
    }

    #[test]
    fn a_scheme_pointing_nowhere_is_not_a_reference() {
        assert!(targets("escribir https:// y ya").is_empty());
    }

    #[test]
    fn code_is_read_as_text_because_that_is_what_backticks_mean() {
        assert!(targets("usa `[[algo]]` para enlazar").is_empty());
        assert!(targets("``` \n [[algo]] \n ```").is_empty());
        assert_eq!(targets("`sin cerrar [[algo]]"), ["algo"]);
    }

    #[test]
    fn the_same_target_twice_is_one_reference() {
        assert_eq!(targets("[[A]] y otra vez [[A]]"), ["A"]);
    }

    #[test]
    fn a_reference_that_never_closes_is_not_one() {
        assert!(targets("[[sin cerrar").is_empty());
        assert!(targets("[etiqueta](sin cerrar").is_empty());
        assert!(targets("[[]]").is_empty());
        assert!(targets("[etiqueta]()").is_empty());
    }

    #[test]
    fn a_label_is_optional_and_a_title_is_not_the_target() {
        assert_eq!(
            extract("[](https://x.example/1) y [a](https://y.example/2 \"por qué\")"),
            vec![
                Ref {
                    kind: Kind::Link,
                    target: "https://x.example/1".into(),
                    label: None
                },
                Ref {
                    kind: Kind::Link,
                    target: "https://y.example/2".into(),
                    label: Some("a".into())
                },
            ]
        );
    }

    #[test]
    fn accents_do_not_shift_the_scan() {
        assert_eq!(
            targets("añadir ñandú über 🎉 [[mañana]] y https://x.example/ñ"),
            ["mañana", "https://x.example/ñ"]
        );
    }
}
