use std::ops::Range;

use super::{CROSSING, Changed, plan, planned, reread, reread_edit};
use crate::read;

fn plain(said: &str) -> String {
    said.lines()
        .filter(|line| !CROSSING.iter().any(|one| line.contains(one)))
        .map(|line| format!("{line}\n"))
        .collect()
}

const SAMPLES: [&str; 4] = [
    include_str!("../../../app/src-tauri/resources/guide/es/muestra.md"),
    include_str!("../../../app/src-tauri/resources/guide/en/sample.md"),
    "---\ntags: [rina]\n---\n\n# Uno\n\nTexto con #rina y `code`.\n\n- a\n- b\n\n  sigue\n\n> cita\n> [!NOTE]\n\n```rust\nfn x() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nTítulo\n===\n\nfin\n",
    "Uno\n\nDos\n\n<div>\nhtml\n</div>\n\n    indentado\n\n1. uno\n2. dos\n\n$$x$$\n\n- [ ] paso\n- [x] hecho\n\núltimo",
];

const PIECES: [&str; 26] = [
    "a", "ñ", " ", "\n", "\n\n", "#", "# ", "- ", "> ", "```", "```\n", "|", "| x |", "**", "*",
    "`", "---", "===", "<div>", "$", "[x](y)", "#tag", "~~~", "1. ", "    ", "\t",
];

struct Steps(u64);

impl Steps {
    fn next(&mut self, below: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % below.max(1) as u64) as usize
    }
}

fn boundary(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

fn edited(text: &str, steps: &mut Steps) -> String {
    let at = boundary(text, steps.next(text.len() + 1));
    if steps.next(3) == 0 {
        let to = boundary(text, (at + steps.next(12)).min(text.len()));
        return format!("{}{}", &text[..at], &text[to..]);
    }
    let piece = PIECES[steps.next(PIECES.len())];
    format!("{}{piece}{}", &text[..at], &text[at..])
}

#[test]
fn a_text_read_again_around_an_edit_reads_as_the_whole_text_does() {
    let mut short = 0;
    let mut all = 0;
    for (seed, sample) in SAMPLES.iter().enumerate() {
        let mut steps = Steps(0x9E37_79B9 + seed as u64);
        let mut text = plain(sample);
        let mut was = read(&text);
        for _ in 0..400 {
            let next = edited(&text, &mut steps);
            short += usize::from(plan(&was, &next).is_some());
            all += 1;
            let again = reread(was, &next);
            let whole = read(&next);
            if again != whole {
                let at = again
                    .blocks()
                    .iter()
                    .zip(whole.blocks())
                    .position(|(a, b)| a != b);
                let pick = |doc: &crate::Doc| at.and_then(|at| doc.blocks().get(at).cloned());
                panic!(
                    "block {at:?} of {} vs {}:
{:#?}
---
{:#?}",
                    again.blocks().len(),
                    whole.blocks().len(),
                    pick(&again),
                    pick(&whole)
                );
            }
            text = next;
            was = again;
        }
    }
    assert!(
        short * 2 > all,
        "only {short} of {all} edits were read around"
    );
}

fn an_edit(text: &str, steps: &mut Steps) -> (Range<usize>, &'static str) {
    let at = boundary(text, steps.next(text.len() + 1));
    let to = boundary(text, (at + steps.next(12)).min(text.len()));
    let insert = if steps.next(4) == 0 {
        ""
    } else {
        PIECES[steps.next(PIECES.len())]
    };
    (at..to, insert)
}

#[test]
fn an_edit_read_again_where_the_page_says_reads_as_the_whole_new_text_does() {
    let mut short = 0;
    let mut all = 0;
    for (seed, sample) in SAMPLES.iter().enumerate() {
        let mut steps = Steps(0x5EED_CAFE + seed as u64);
        let mut text = plain(sample);
        let mut was = read(&text);
        for _ in 0..400 {
            let (at, insert) = an_edit(&text, &mut steps);
            let next = format!("{}{insert}{}", &text[..at.start], &text[at.end..]);
            let changed = Changed {
                start: at.start,
                old_end: at.end,
                new_end: at.start + insert.len(),
            };
            short += usize::from(planned(&was, &next, &changed).is_some());
            all += 1;
            let again = reread_edit(was, at.clone(), insert)
                .unwrap_or_else(|_| panic!("{at:?} falls between the characters"));
            assert_eq!(again, read(&next), "{at:?} → {insert:?} in {text:?}");
            text = next;
            was = again;
        }
    }
    assert!(
        short * 2 > all,
        "only {short} of {all} edits were read around"
    );
}

#[test]
fn a_definition_put_together_by_an_edit_is_remembered_and_one_taken_out_is_forgotten() {
    let text = "Uno [x]x: y.\n\nDos.\n";
    let at = text.find("]x:").unwrap_or_default() + 1;
    let joined = reread_edit(read(text), at..at + 1, "");
    assert_eq!(joined, Ok(read("Uno [x]: y.\n\nDos.\n")));
    assert!(joined.is_ok_and(|doc| doc.crossing));
    let typed = reread_edit(read("Uno ]\n"), 5..5, ":");
    assert!(typed.is_ok_and(|doc| doc.crossing));
    let definition = "[r]: https://tisty.app\n\n";
    let defined = format!("Uno [x][r].\n\n{definition}Dos.\n");
    let line = defined.find(definition).unwrap_or_default();
    let Ok(taken) = reread_edit(read(&defined), line..line + definition.len(), "") else {
        panic!("the definition falls between characters");
    };
    assert_eq!(taken, read("Uno [x][r].\n\nDos.\n"));
    assert!(!taken.crossing);
    assert!(plan(&taken, "Uno [x][r].\n\nDos!\n").is_some());
}

#[test]
fn edits_that_make_or_break_crossings_read_as_the_whole_text_does() {
    const CROSSINGS: [&str; 8] = ["]", ":", "[^", "]:", ":::", "[x]: y", "[^1]", "\n"];
    for (seed, sample) in SAMPLES.iter().enumerate() {
        let mut steps = Steps(0xC0FF_EE00 + seed as u64);
        let mut text = plain(sample);
        let mut was = read(&text);
        for _ in 0..300 {
            let at = boundary(&text, steps.next(text.len() + 1));
            let to = boundary(&text, (at + steps.next(4)).min(text.len()));
            let insert = CROSSINGS[steps.next(CROSSINGS.len())];
            let next = format!("{}{insert}{}", &text[..at], &text[to..]);
            let again = reread_edit(was, at..to, insert)
                .unwrap_or_else(|_| panic!("{at}..{to} falls between the characters"));
            assert_eq!(again, read(&next), "{at}..{to} → {insert:?} in {text:?}");
            text = next;
            was = again;
        }
    }
}

#[test]
fn a_word_rewritten_whole_reads_as_the_minimal_change_does() {
    let text = "Uno dos.\n\nTres cuatro.\n\nCinco seis.\n";
    let at = text.find("cuatro").unwrap_or_default();
    let again = reread_edit(read(text), at..at + 6, "cuatros");
    assert_eq!(again, Ok(read(&text.replace("cuatro", "cuatros"))));
}

#[test]
fn an_edit_off_the_characters_gives_the_text_back_untouched() {
    let text = "Año con ñandú.\n";
    assert_eq!(reread_edit(read(text), 2..3, "x"), Err(read(text)));
    assert_eq!(reread_edit(read(text), 3..99, ""), Err(read(text)));
}

#[test]
fn words_typed_in_a_paragraph_are_read_again_alone() {
    let text = "Uno dos.\n\nTres cuatro.\n\nCinco seis.\n";
    let was = read(text);
    let next = text.replace("cuatro", "cuatros");
    assert!(plan(&was, &next).is_some());
    assert_eq!(reread(was, &next), read(&next));
}

#[test]
fn a_definition_or_a_footnote_anywhere_reads_the_whole_text() {
    let text = "Uno [x][r].\n\n[r]: https://tisty.app\n";
    assert!(plan(&read(text), &text.replace("Uno", "Unos")).is_none());
    let noted = "Uno[^1].\n\n[^1]: nota\n";
    assert!(plan(&read(noted), &noted.replace("Uno", "Unos")).is_none());
}

#[test]
fn a_definition_written_by_the_edit_reads_the_whole_text() {
    let text = "Uno [x][r].\n\nDos.\n";
    let next = text.replace("Dos.", "[r]: https://tisty.app");
    assert!(plan(&read(text), &next).is_none());
    assert_eq!(reread(read(text), &next), read(&next));
}

const TAGGED: &str = "---\ntags: [rina]\n---\n\nTexto con #rina y #roger.\n\n> ```\n> #citado\n> ```\n\n- uno #lista\n\n  ```\n  #dentro\n  ```\n\n- dos\n\n~~~\n#tilde\n~~~\n\nUn #n\u{303}andu y un #ñandú, `#nada` y #algo.\n\nfin #rina\n";

const TAG_PIECES: [&str; 20] = [
    "#rina",
    "#roger",
    "#",
    "a",
    "ñ",
    " ",
    "\n",
    "\n\n",
    "```",
    "```\n",
    "~~~\n",
    "`",
    "> ",
    "- ",
    "  ",
    "    ",
    "#Rina",
    "#n\u{303}",
    "\\",
    "#t65",
];

#[test]
fn tags_written_anywhere_are_filed_as_reading_the_whole_text_files_them() {
    let many: String = (0..70).map(|n| format!("#t{n} ")).collect();
    let texts = [
        plain(SAMPLES[0]),
        TAGGED.to_string(),
        format!("{many}\n\nUno #t3.\n"),
    ];
    let mut short = 0;
    let mut all = 0;
    for (seed, sample) in texts.iter().enumerate() {
        let mut steps = Steps(0x7A65_0000 + seed as u64);
        let mut text = sample.clone();
        let mut was = read(&text);
        for _ in 0..400 {
            let at = boundary(&text, steps.next(text.len() + 1));
            let to = boundary(&text, (at + steps.next(6)).min(text.len()));
            let insert = TAG_PIECES[steps.next(TAG_PIECES.len())];
            let next = format!("{}{insert}{}", &text[..at], &text[to..]);
            let changed = Changed {
                start: at,
                old_end: to,
                new_end: at + insert.len(),
            };
            let filing = was.filing.after(&text, &next, &changed).0;
            assert_eq!(
                filing,
                crate::filing::Filing::of(&next),
                "{at}..{to} → {insert:?} in {text:?}"
            );
            short += usize::from(planned(&was, &next, &changed).is_some());
            all += 1;
            let again = reread_edit(was, at..to, insert)
                .unwrap_or_else(|_| panic!("{at}..{to} falls between the characters"));
            assert_eq!(again, read(&next), "{at}..{to} → {insert:?} in {text:?}");
            text = next;
            was = again;
        }
    }
    assert!(
        short * 2 > all,
        "only {short} of {all} edits were read around"
    );
}

#[test]
fn a_tag_typed_on_a_long_page_is_read_around_at_each_key() {
    let mut text = format!(
        "{}Uno #\n\n{}",
        "Párrafo.\n\n".repeat(40),
        "Otro #rina.\n\n".repeat(40)
    );
    let mut was = read(&text);
    let typed = text.find("Uno #").unwrap_or_default() + 5;
    for (at, key) in (typed..).zip(["r", "o", "g", "e", "r", "x"]) {
        let next = format!("{}{key}{}", &text[..at], &text[at..]);
        assert!(plan(&was, &next).is_some(), "{key} after {:?}", &text[..at]);
        let Ok(again) = reread_edit(was, at..at, key) else {
            panic!("{key} falls between the characters");
        };
        assert_eq!(again, read(&next));
        text = next;
        was = again;
    }
}

#[test]
fn a_fence_opened_by_an_edit_reads_the_whole_text() {
    let text = "Uno.\n\nDos.\n\nTres.\n";
    let next = text.replace("Dos.", "```\nDos.");
    assert_eq!(reread(read(text), &next), read(&next));
}
