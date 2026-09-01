#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Doc,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Ref {
    pub kind: Kind,
    pub target: String,
    pub label: Option<String>,
}

pub const DOC: &str = "tisty:doc/";

pub fn card(file: &str, title: &str) -> String {
    let said: String = title
        .chars()
        .flat_map(|c| {
            let slash = matches!(c, '[' | ']' | '\\').then_some('\\');
            slash.into_iter().chain(std::iter::once(c))
        })
        .collect();
    format!("![{said}]({DOC}{file})")
}

pub fn papers(text: &str) -> Vec<String> {
    extract(text)
        .into_iter()
        .filter_map(|one| one.target.strip_prefix(DOC).map(str::to_string))
        .collect()
}

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
            b'<' => tagged(rest, at, &mut keep),
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

fn shuts(rest: &str) -> Option<usize> {
    let mut escaped = false;
    for (at, c) in rest.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            ']' => return Some(at),
            '[' | '\n' => return None,
            _ => {}
        }
    }
    None
}

fn linked(text: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let rest = &text[at + 1..];
    let Some(shut) = shuts(rest) else {
        return at + 1;
    };
    let label = &rest[..shut];

    let after = at + 1 + shut + 1;
    if text.as_bytes().get(after) != Some(&b'(') {
        return at + 1;
    }
    let tail = &text[after + 1..];

    let (target, close) = match tail.trim_start().strip_prefix('<') {
        Some(_) => {
            let opened = tail.find('<').unwrap_or(0);
            let Some(shut) = tail[opened..].find('>').map(|n| opened + n) else {
                return at + 1;
            };
            match tail[shut..].find(')') {
                Some(paren) => (&tail[opened + 1..shut], shut + paren),
                None => return at + 1,
            }
        }
        None => {
            let Some(close) = tail.find(')') else {
                return at + 1;
            };
            (tail[..close].split_whitespace().next().unwrap_or(""), close)
        }
    };
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

/// An aligned paragraph is written as html, so what it points at is in an attribute, not in
/// brackets. Missed here, a file nothing else names reads as loose and is swept away.
fn tagged(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let opens = rest[1..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '/');
    // A tag ends on its own line. Anything else — prose, a comparison, an autolink, a comment —
    // is read on, byte by byte, or one stray angle would swallow every reference after it.
    let shut = rest.find('\n').unwrap_or(rest.len());
    let Some(end) = opens.then(|| rest[..shut].find('>')).flatten() else {
        return at + 1;
    };
    let tag = &rest[..end];
    if !tag.contains('=') {
        return at + 1;
    }
    for name in [" href=\"", " src=\""] {
        let Some(from) = tag.find(name).map(|n| n + name.len()) else {
            continue;
        };
        let Some(stop) = tag[from..].find('"').map(|n| from + n) else {
            continue;
        };
        let target = tag[from..stop].trim();
        if !target.is_empty() {
            keep(Ref {
                kind: Kind::Link,
                target: target
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&"),
                label: None,
            });
        }
    }
    at + end + 1
}

fn bare(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '<' || c == '>')
        .unwrap_or(rest.len());
    let url = unpunctuated(&rest[..end]);

    if url.len() > url.find("//").map_or(0, |n| n + 2) {
        keep(Ref {
            kind: Kind::Link,
            target: url.to_string(),
            label: None,
        });
    }
    at + end
}

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
    fn a_wrapped_destination_loses_its_brackets() {
        assert_eq!(
            targets("![shot](<attachments/ab/cd.png>)"),
            ["attachments/ab/cd.png"]
        );
        assert_eq!(
            targets("[clip](<C:/My Docs/clip (1).mkv>)"),
            ["C:/My Docs/clip (1).mkv"]
        );
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
    fn the_documents_a_text_names_come_out_in_the_order_it_names_them() {
        assert_eq!(
            papers(
                "primero ![Uno](tisty:doc/mac0-0002)\n\nluego [Dos](tisty:doc/mac0-0001)\n\ny https://x.example/"
            ),
            ["mac0-0002", "mac0-0001"]
        );
    }

    #[test]
    fn a_document_named_twice_is_only_counted_where_it_is_first_named() {
        assert_eq!(
            papers("![A](tisty:doc/mac0-0001) ![B](tisty:doc/mac0-0002) ![A](tisty:doc/mac0-0001)"),
            ["mac0-0001", "mac0-0002"]
        );
    }

    #[test]
    fn a_document_named_inside_code_is_not_named_at_all() {
        assert_eq!(papers("`![A](tisty:doc/mac0-0001)`"), Vec::<String>::new());
    }

    #[test]
    fn a_title_with_brackets_is_still_read_back_from_the_card_written_for_it() {
        let said = card("mac0-0010", "Capitulo 1 [borrador]");

        assert_eq!(papers(&said), ["mac0-0010"], "{said}");
    }

    #[test]
    fn what_an_aligned_paragraph_points_at_is_still_pointed_at() {
        let said = "<p style=\"text-align: center\"><a href=\"attachments/ab/nota-1234.pdf\">el                     plano</a></p>";

        assert_eq!(targets(said), ["attachments/ab/nota-1234.pdf"]);
    }

    #[test]
    fn a_page_named_from_an_aligned_paragraph_is_named_all_the_same() {
        let said = "<p style=\"text-align: center\"><a href=\"tisty:doc/mac0-0010\">Uno</a></p>";

        assert_eq!(papers(said), ["mac0-0010"]);
    }

    #[test]
    fn an_angle_in_prose_does_not_swallow_what_comes_after_it() {
        let said = "si a < b mira [el plano](attachments/ab/plano-1234.pdf) y 5 > 3";

        assert_eq!(targets(said), ["attachments/ab/plano-1234.pdf"]);
    }

    #[test]
    fn an_angle_between_two_cards_leaves_both_where_they_are() {
        let said = "![Uno](tisty:doc/mac0-0001)

si a < b

![Dos](tisty:doc/mac0-0002)";

        assert_eq!(papers(said), ["mac0-0001", "mac0-0002"]);
    }

    #[test]
    fn an_address_written_between_angles_is_still_an_address() {
        assert_eq!(
            targets("<https://x.example/one>"),
            ["https://x.example/one"]
        );
    }

    #[test]
    fn a_reference_inside_a_comment_is_still_a_reference() {
        assert_eq!(
            targets("<!-- [x](attachments/ab/x-1111.pdf) -->"),
            ["attachments/ab/x-1111.pdf"]
        );
    }

    #[test]
    fn an_attribute_that_only_ends_in_href_names_nothing() {
        let said = "<img data-href=\"attachments/ab/fantasma-1111.pdf\" alt=\"x\">";

        assert!(targets(said).is_empty(), "{said}");
    }

    #[test]
    fn an_ampersand_written_for_html_is_read_back_as_one() {
        let said = "<p><a href=\"https://x.example/a?one=1&amp;two=2\">x</a></p>";

        assert_eq!(targets(said), ["https://x.example/a?one=1&two=2"]);
    }

    #[test]
    fn a_title_ending_in_a_slash_does_not_escape_the_bracket_that_closes_it() {
        let said = card("mac0-0010", "Rutas C:\\ y mas");

        assert_eq!(papers(&said), ["mac0-0010"], "{said}");
    }

    #[test]
    fn a_label_holding_a_link_is_still_no_label_at_all() {
        let found = extract("[uno [dos](https://x.example/2)](https://y.example/1)");
        let outer = found
            .iter()
            .find(|one| one.target.contains("y.example"))
            .unwrap();

        assert_eq!(outer.label, None, "the outer brackets label nothing");
    }

    #[test]
    fn accents_do_not_shift_the_scan() {
        assert_eq!(
            targets("añadir ñandú über 🎉 [[mañana]] y https://x.example/ñ"),
            ["mañana", "https://x.example/ñ"]
        );
    }
}
