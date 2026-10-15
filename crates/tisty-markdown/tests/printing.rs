use serde_json::Value;
use tisty_markdown::{printed, read};

fn kinds(body: &str) -> Vec<String> {
    let tree = printed(&read(body));
    tree["content"]
        .as_array()
        .map(|all| {
            all.iter()
                .map(|one| one["type"].as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn first(body: &str) -> Value {
    printed(&read(body))["content"][0].clone()
}

#[test]
fn a_page_off_disk_is_laid_out_in_the_nodes_the_sheet_reads() {
    let body = "# Marzo\n\nlo que se **dijo**.\n\n- uno\n- dos\n\n![plano](<attachments/ab/plano-a1b2.png>)";
    assert_eq!(
        kinds(body),
        ["heading", "paragraph", "bulletList", "paragraph"]
    );
    let tree = printed(&read(body));
    assert_eq!(tree["content"][0]["attrs"]["level"], 1);
    assert_eq!(tree["content"][1]["content"][1]["marks"][0]["type"], "bold");
    assert_eq!(tree["content"][3]["content"][0]["type"], "image");
}

#[test]
fn a_quote_and_a_table_make_a_book_of_minutes() {
    assert_eq!(
        kinds("> lo dijo el comité\n\n| a | b |\n| :- | -: |\n| 1 | 2 |\n"),
        ["blockquote", "table"]
    );
    let table = &printed(&read("| a | b |\n| :- | -: |\n| 1 | 2 |\n"))["content"][0];
    assert_eq!(
        table["content"][0]["content"][0]["attrs"]["textAlign"],
        "left"
    );
    assert_eq!(
        table["content"][0]["content"][1]["attrs"]["textAlign"],
        "right"
    );
}

#[test]
fn a_callout_keeps_its_kind_and_the_words_a_rule_under_them_would_swallow() {
    let note = first("> [!NOTE]\n> ---");
    assert_eq!(note["type"], "callout");
    assert_eq!(note["attrs"]["kind"], "note");
    let warning = first("> [!WARNING]\n> algo importante\n> ---");
    assert_eq!(warning["attrs"]["kind"], "warning");
    assert!(warning.to_string().contains("algo importante"));
}

#[test]
fn lists_say_whether_they_count_tick_or_bullet() {
    assert_eq!(kinds("3. tres\n4. cuatro"), ["orderedList"]);
    assert_eq!(first("3. tres\n4. cuatro")["attrs"]["start"], 3);
    let ticked = first("- [x] hecha\n- [ ] no");
    assert_eq!(ticked["type"], "taskList");
    assert_eq!(ticked["content"][0]["attrs"]["checked"], true);
    assert_eq!(ticked["content"][1]["attrs"]["checked"], false);
}

#[test]
fn inline_marks_reach_the_sheet_with_what_they_carry() {
    let line = first("~~no~~ `x` [web](https://a.b) ==sí== <mark data-pen=\"green\">v</mark>\nfin");
    let said = line.to_string();
    assert!(said.contains("\"strike\""));
    assert!(said.contains("\"code\""));
    assert!(said.contains("https://a.b"));
    assert!(said.contains("\"yellow\""));
    assert!(said.contains("\"green\""));
    assert!(said.contains("hardBreak"));
}

#[test]
fn code_keeps_its_language_and_a_rule_stays_a_rule() {
    let code = first("```rust\nfn main() {}\n```");
    assert_eq!(code["type"], "codeBlock");
    assert_eq!(code["attrs"]["language"], "rust");
    assert_eq!(code["content"][0]["text"], "fn main() {}\n");
    assert_eq!(
        kinds("uno\n\n---\n\ndos"),
        ["paragraph", "horizontalRule", "paragraph"]
    );
}
