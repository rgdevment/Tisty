use super::Filing;
use crate::rereading::Changed;
use crate::tags::tags_in;

fn edited(text: &str, at: usize, gone: usize, insert: &str) -> (String, Changed) {
    let next = format!("{}{insert}{}", &text[..at], &text[at + gone..]);
    let changed = Changed {
        start: at,
        old_end: at + gone,
        new_end: at + insert.len(),
    };
    (next, changed)
}

#[test]
fn the_names_filed_are_those_the_tags_of_the_text_are() {
    let many: String = (0..70).map(|n| format!("#t{n} ")).collect();
    let texts = [
        include_str!("../../../app/src-tauri/resources/guide/es/muestra.md"),
        include_str!("../../../app/src-tauri/resources/guide/en/sample.md"),
        "---\ntags: [rina]\n---\n\nUno #roger.\n\n```\n#codigo\n```\n",
        &many,
    ];
    for text in texts {
        let names: std::collections::HashSet<String> = tags_in(text).into_iter().collect();
        assert_eq!(Filing::of(text).filed, names);
    }
}

#[test]
fn a_fence_opened_by_an_edit_moves_the_lines_after_it_into_code() {
    let text = "Uno #rina.\n\nDos #roger.\n\nTres #roger.\n";
    let at = text.find("Dos").unwrap_or_default();
    let (next, changed) = edited(text, at, 0, "```\n");
    let (filing, settled) = Filing::of(text).after(text, &next, &changed);
    assert_eq!(filing, Filing::of(&next));
    assert!(!settled);
    assert!(!filing.filed.contains("roger"));
}

#[test]
fn a_name_also_written_in_code_keeps_an_edit_from_filing_it_alone() {
    let text = "Uno #ri.\n\n```\n#rina\n```\n";
    let at = text.find("#ri").unwrap_or_default() + 3;
    let (next, changed) = edited(text, at, 0, "na");
    let was = Filing::of(text);
    let (filing, settled) = was.after(text, &next, &changed);
    assert_eq!(filing, Filing::of(&next));
    assert!(settled);
    assert!(!filing.alone(&was, settled));
    let (other, _) = edited(text, at, 0, "ta");
    let (filing, settled) = was.after(text, &other, &changed);
    assert!(filing.alone(&was, settled));
}
