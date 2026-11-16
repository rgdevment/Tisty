use super::{CROSSING, plan, reread};
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

#[test]
fn a_fence_opened_by_an_edit_reads_the_whole_text() {
    let text = "Uno.\n\nDos.\n\nTres.\n";
    let next = text.replace("Dos.", "```\nDos.");
    assert_eq!(reread(read(text), &next), read(&next));
}
