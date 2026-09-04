use crate::model::Tag;

/// A hash pinned to a letter or a digit. `# Heading` carries a space and stays a heading, a colour
/// or a fragment sits behind something that is not a separator, and a fenced block is skipped
/// whole — the same fencing the title reader steps over.
/// Past this a body is not being tagged, it is carrying something else — a stylesheet pasted
/// outside a fence reads every colour as a tag. The log is append-only: a line of thousands of
/// them is written once and kept forever, on every machine.
pub const AT_MOST: usize = 64;

pub fn tags_in(body: &str) -> Vec<Tag> {
    let mut found: Vec<Tag> = Vec::new();
    let mut seen: std::collections::HashSet<Tag> = std::collections::HashSet::new();
    let mut fenced = crate::docs::fencing();
    for line in body.lines() {
        if fenced(line) {
            continue;
        }
        // Composed first: text pasted from macOS carries «ñ» as two code points, and a reader
        // that stops at the first mark would file #diseño under «disen».
        for tag in tags_on(&crate::text::composed(line)) {
            if seen.insert(tag.clone()) {
                found.push(tag);
            }
            if found.len() == AT_MOST {
                return found;
            }
        }
    }
    found
}

fn tags_on(line: &str) -> Vec<Tag> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    // Only a backtick that closes marks code. One on its own is prose — a stray accent in a
    // sentence must not swallow every tag written after it.
    let mut code = false;
    let mut at = 0;
    while at < line.len() {
        if bytes[at] == b'`' {
            code = !code && line[at + 1..].contains('`');
            at += 1;
            continue;
        }
        if bytes[at] != b'#' || code {
            at += 1;
            continue;
        }
        let before = line[..at].chars().next_back();
        // Anything but a separator before the hash means it belongs to what came first: a colour
        // in `bg-#fff`, the fragment of an address, a word someone hyphenated.
        if before.is_some_and(|one| one.is_alphanumeric() || "#/-_.:".contains(one)) {
            at += 1;
            continue;
        }
        let rest = &line[at + 1..];
        // A letter or a digit against the hash, or it is not a tag: `#-` and `#_` are how code
        // and web addresses are written, not how anybody labels their own work.
        if !rest.chars().next().is_some_and(char::is_alphanumeric) {
            at += 1;
            continue;
        }
        let word: String = rest
            .chars()
            .take_while(|one| one.is_alphanumeric() || *one == '-' || *one == '_')
            .collect();
        if let Ok(tag) = Tag::new(&word) {
            found.push(tag);
        }
        at += 1 + word.len();
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn said(body: &str) -> Vec<String> {
        tags_in(body)
            .into_iter()
            .map(|one| one.as_str().to_string())
            .collect()
    }

    #[test]
    fn a_hash_against_a_word_is_a_tag() {
        assert_eq!(said("esto es #legal antes que nada"), ["legal"]);
    }

    #[test]
    fn a_heading_keeps_its_space_and_is_left_alone() {
        assert_eq!(
            said("# Alquiler del local\n\n## Lo que falta"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn the_fragment_of_an_address_belongs_to_the_address() {
        assert_eq!(
            said("mira https://ejemplo.com/pagina#seccion"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_colour_written_inline_is_not_a_tag_either() {
        assert_eq!(said("el fondo es `#ff0000` y ya"), Vec::<String>::new());
        assert_eq!(said("border: 1px solid #hair"), ["hair"]);
    }

    #[test]
    fn a_fenced_block_is_stepped_over_whole() {
        assert_eq!(
            said("antes #uno\n\n```css\ncolor: #rojo;\n```\n\ndespués #dos"),
            ["uno", "dos"]
        );
    }

    #[test]
    fn the_same_tag_twice_is_kept_once_and_in_the_order_it_was_written() {
        assert_eq!(
            said("#dinero y luego #legal y otra vez #dinero"),
            ["dinero", "legal"]
        );
    }

    #[test]
    fn it_is_normalised_the_way_a_task_normalises_its_own() {
        assert_eq!(said("#Contrato #CONTRATO #contrato"), ["contrato"]);
        assert_eq!(said("#pago_mensual"), ["pago-mensual"]);
    }

    #[test]
    fn a_hash_on_its_own_holds_nothing() {
        assert_eq!(said("un # suelto y un #- también"), Vec::<String>::new());
    }

    #[test]
    fn punctuation_around_it_does_not_travel_with_it() {
        assert_eq!(said("queda (#legal), sí."), ["legal"]);
    }

    #[test]
    fn several_on_one_line_are_all_read() {
        assert_eq!(said("#uno #dos #tres"), ["uno", "dos", "tres"]);
    }

    #[test]
    fn a_word_pasted_from_a_mac_is_the_same_word() {
        assert_eq!(said("revisar #disen\u{303}o hoy"), ["diseno"]);
        assert_eq!(said("#diseño"), said("revisar #disen\u{303}o hoy"));
    }

    #[test]
    fn a_backtick_on_its_own_is_prose_and_swallows_nothing() {
        assert_eq!(
            said("el operador ` marca código y esto es #legal"),
            ["legal"]
        );
        assert_eq!(said("`#rojo` no, pero #legal sí"), ["legal"]);
        assert_eq!(said("#uno `#dos` #tres"), ["uno", "tres"]);
    }

    #[test]
    fn a_hash_needs_a_letter_against_it() {
        assert_eq!(said("#_borrador y #-legal"), Vec::<String>::new());
        assert_eq!(said("#borrador y #legal"), ["borrador", "legal"]);
        assert_eq!(said("#2026 cuenta"), ["2026"]);
    }

    #[test]
    fn a_body_that_is_not_tagging_stops_at_the_cap() {
        let css: String = (0..2000)
            .map(|n| {
                format!(
                    ".c{n} {{ color: #a{n:04x}; }}
"
                )
            })
            .collect();

        assert_eq!(
            tags_in(&css).len(),
            AT_MOST,
            "ni una linea de miles en el registro"
        );
    }

    #[test]
    fn what_a_document_says_of_itself_carries_them() {
        let said = crate::event::Said::of("# Alquiler\n\nesto es #legal y #dinero");

        assert_eq!(said.title, "Alquiler");
        assert_eq!(
            said.tags
                .unwrap()
                .iter()
                .map(|one| one.as_str())
                .collect::<Vec<_>>(),
            ["legal", "dinero"]
        );
    }

    #[test]
    fn a_tag_that_changed_is_news_even_where_the_title_did_not() {
        let kept = crate::model::Kept {
            id: ulid::Ulid::generate(),
            file: "a3f1-0001".into(),
            order: "a0".into(),
            title: Some("Alquiler".into()),
            bytes: Some(31),
            wrote: None,
            folder: None,
            page_of: None,
            archived: false,
            locked: false,
            tags: vec![Tag::new("legal").unwrap()],
        };

        let same = crate::event::Said {
            title: "Alquiler".into(),
            bytes: Some(31),
            tags: Some(vec![Tag::new("legal").unwrap()]),
        };
        assert!(!same.news_for(&kept));

        let fresh = crate::event::Said {
            tags: Some(vec![
                Tag::new("legal").unwrap(),
                Tag::new("dinero").unwrap(),
            ]),
            ..same.clone()
        };
        let older = crate::event::Said { tags: None, ..same };
        assert!(
            fresh.news_for(&kept),
            "una etiqueta nueva es algo que contar"
        );
        assert!(
            !older.news_for(&kept),
            "y una nota de una version que no las leia no dice nada de ellas"
        );
    }
}
