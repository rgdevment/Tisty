use unicode_normalization::UnicodeNormalization;

pub fn plainly(text: &str) -> String {
    composed(
        &text
            .chars()
            .filter(|c| {
                !c.is_control() && !matches!(*c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            })
            .collect::<String>(),
    )
    .trim()
    .chars()
    .take(120)
    .collect()
}

pub fn folded(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.to_lowercase()
        .nfd()
        .filter(|c| !matches!(*c as u32, 0x0300..=0x036F))
        .collect()
}

pub const TERMS_AT_MOST: usize = 12;

/// A search is words, not one string: nobody types a title back in the order it was written.
/// Quotes hold a phrase together for when they mean the order.
pub fn terms(query: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut word = String::new();
    let mut quoted = false;

    for c in folded(query).chars() {
        match c {
            '"' | '\u{201c}' | '\u{201d}' => {
                if !word.is_empty() {
                    found.push(std::mem::take(&mut word));
                }
                quoted = !quoted;
            }
            c if c.is_whitespace() && !quoted => {
                if !word.is_empty() {
                    found.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
        if found.len() >= TERMS_AT_MOST {
            return found;
        }
    }
    if !word.is_empty() {
        found.push(word);
    }
    found
}

pub fn composed(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    text.nfc().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_cannot_carry_something_that_rewrites_the_line_around_it() {
        assert_eq!(
            plainly("linea\u{7}uno\u{202e}txet\u{202c}"),
            "lineauno txet".replace(' ', "")
        );
    }

    #[test]
    fn a_name_is_cut_before_it_can_fill_the_rail() {
        assert_eq!(plainly(&"a".repeat(500)).chars().count(), 120);
    }

    #[test]
    fn an_ordinary_name_comes_back_untouched() {
        assert_eq!(plainly("  Diseño técnico  "), "Diseño técnico");
    }

    const DECOMPOSED: &str = "man\u{0303}ana revisar el disen\u{0303}o";
    const COMPOSED: &str = "mañana revisar el diseño";

    #[test]
    fn the_two_spellings_of_one_letter_become_one() {
        assert_eq!(composed(DECOMPOSED), COMPOSED);
        assert_ne!(DECOMPOSED, COMPOSED, "the case only exists if they differ");
    }

    #[test]
    fn what_is_already_composed_is_left_exactly_as_it_is() {
        assert_eq!(composed(COMPOSED), COMPOSED);
        assert_eq!(composed("plain ascii"), "plain ascii");
        assert_eq!(composed(""), "");
    }

    #[test]
    fn every_language_gets_the_same_treatment() {
        assert_eq!(composed("Gro\u{0308}\u{df}e"), "Größe");
        assert_eq!(composed("cafe\u{0301}"), "café");
        assert_eq!(composed("Đa\u{0300} Na\u{0306}\u{0303}ng"), "Đà Nẵng");
    }

    #[test]
    fn a_stranded_mark_survives_rather_than_being_dropped() {
        assert!(composed("\u{0303}").contains('\u{0303}'));
    }

    #[test]
    fn a_search_is_cut_into_words_without_their_accents() {
        assert_eq!(
            terms("  Análisis   del  Repositorio "),
            ["analisis", "del", "repositorio"]
        );
        assert_eq!(terms("MERKÉN"), ["merken"]);
    }

    #[test]
    fn nothing_typed_is_nothing_to_look_for() {
        assert!(terms("").is_empty());
        assert!(terms("   ").is_empty());
        assert!(terms("\"\"").is_empty());
    }

    #[test]
    fn quotes_hold_a_phrase_together() {
        assert_eq!(terms("\"casa de campo\" verde"), ["casa de campo", "verde"]);
        assert_eq!(terms("\u{201c}casa de campo\u{201d}"), ["casa de campo"]);
        assert_eq!(terms("\"sin cerrar"), ["sin cerrar"]);
    }

    #[test]
    fn a_search_cannot_grow_long_enough_to_scan_the_store_a_hundred_times() {
        let many = terms(&(1..40).map(|n| format!("w{n} ")).collect::<String>());

        assert_eq!(many.len(), TERMS_AT_MOST);
        assert_eq!(many[0], "w1");
    }

    #[test]
    fn composing_twice_changes_nothing_the_second_time() {
        let once = composed(DECOMPOSED);

        assert_eq!(composed(&once), once);
    }
}
