use std::cell::Cell;

use super::{Bodies, Compared, Step, compared, key, spending, told, unmarked};
use crate::shared::CELLS_AT_MOST;

const VOID: [&str; 5] = ["br", "hr", "img", "col", "input"];

fn balanced(html: &str) -> bool {
    let mut open: Vec<&str> = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find('<') {
        let Some(end) = rest[at..].find('>') else {
            return false;
        };
        let tag = &rest[at + 1..at + end];
        rest = &rest[at + end + 1..];
        let name = tag
            .trim_start_matches('/')
            .split([' ', '/'])
            .next()
            .unwrap_or_default();
        if tag.starts_with('/') {
            if open.pop() != Some(name) {
                return false;
            }
        } else if !VOID.contains(&name) {
            open.push(name);
        }
    }
    open.is_empty()
}

fn unplaced(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find(" data-at=\"") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 10..];
        rest = &after[after.find('"').map_or(after.len(), |end| end + 1)..];
    }
    out + rest
}

fn page(was: &str, now: &str) -> Compared {
    let said = compared(was, now).expect("two short bodies are told apart");
    assert!(balanced(&said.html), "{}", said.html);
    Compared {
        html: unplaced(&said.html),
        ..said
    }
}

fn drawn(body: &str) -> String {
    crate::html(&crate::read(body))
}

#[test]
fn two_bodies_that_say_the_same_are_the_page_itself() {
    let body = "---\ntags: uno\n---\n\n# Acta\n\nUn párrafo.\n\n- uno\n- dos\n";

    let said = compared(body, body).unwrap();

    assert_eq!(said.html, drawn(body));
    assert_eq!((said.gone, said.came), (0, 0));
}

#[test]
fn a_block_that_went_and_one_that_came_say_so_where_they_stood() {
    let said = page("Uno.\n\nDos.\n\nTres.\n", "Uno.\n\nTres.\n\nCuatro.\n");

    assert_eq!(
        said.html,
        "<p>Uno.</p><p data-change=\"gone\">Dos.</p><p>Tres.</p><p data-change=\"came\">Cuatro.</p>"
    );
    assert_eq!((said.gone, said.came), (1, 1));
}

#[test]
fn each_block_keeps_the_place_it_has_in_its_own_body() {
    let said = compared("Uno.\n\nDos.\n\nTres.\n", "Tres.\n\nCuatro.\n").unwrap();

    assert_eq!(
        said.html,
        "<p data-at=\"0-5\" data-change=\"gone\">Uno.</p><p data-at=\"6-11\" data-change=\"gone\">Dos.</p>\
         <p data-at=\"0-6\">Tres.</p><p data-at=\"7-15\" data-change=\"came\">Cuatro.</p>"
    );
}

#[test]
fn the_few_words_that_changed_in_a_paragraph_are_marked_in_both() {
    let said = page(
        "El gato come pescado fresco hoy.\n",
        "El gato come pollo fresco hoy.\n",
    );

    assert_eq!(
        said.html,
        "<p data-change=\"gone\">El gato come <del>pescado</del> fresco hoy.</p>\
         <p data-change=\"came\">El gato come <ins>pollo</ins> fresco hoy.</p>"
    );
    assert_eq!((said.gone, said.came), (1, 1));
}

#[test]
fn words_side_by_side_that_changed_are_one_mark() {
    let said = page(
        "Vamos a la playa mañana temprano con todos.\n",
        "Vamos al cine mañana temprano con todos.\n",
    );

    assert_eq!(
        said.html,
        "<p data-change=\"gone\">Vamos <del>a la playa</del> mañana temprano con todos.</p>\
         <p data-change=\"came\">Vamos <ins>al cine</ins> mañana temprano con todos.</p>"
    );
}

#[test]
fn a_mark_stays_inside_the_bold_and_the_link_it_changed_in() {
    let said = page(
        "Lee **el informe** de [ayer](https://a.example) ya mismo.\n",
        "Lee **el resumen** de [hoy](https://a.example) ya mismo.\n",
    );

    assert_eq!(
        said.html,
        "<p data-change=\"gone\">Lee <strong>el <del>informe</del></strong> de \
         <a href=\"https://a.example\"><del>ayer</del></a> ya mismo.</p>\
         <p data-change=\"came\">Lee <strong>el <ins>resumen</ins></strong> de \
         <a href=\"https://a.example\"><ins>hoy</ins></a> ya mismo.</p>"
    );
}

#[test]
fn a_link_that_only_goes_elsewhere_still_says_its_block_changed() {
    let said = page(
        "Mira el [sitio](https://a.example) de siempre.\n",
        "Mira el [sitio](https://b.example) de siempre.\n",
    );

    assert_eq!(
        said.html,
        "<p data-change=\"gone\">Mira el <a href=\"https://a.example\"><del>sitio</del></a> de siempre.</p>\
         <p data-change=\"came\">Mira el <a href=\"https://b.example\"><ins>sitio</ins></a> de siempre.</p>"
    );
}

#[test]
fn inline_code_and_raw_html_that_changed_keep_the_page_well_formed() {
    let said = page(
        "Corre `cargo test` y mira <b>esto</b> con calma ahora.\n",
        "Corre `cargo check` y mira <b>aquello</b> con calma ahora.\n",
    );

    assert!(
        said.html.contains("<del><code>cargo test</code></del>"),
        "{}",
        said.html
    );
    assert!(
        said.html.contains("<ins><code>cargo check</code></ins>"),
        "{}",
        said.html
    );
}

#[test]
fn two_paragraphs_with_little_in_common_are_told_whole() {
    let said = page(
        "Uno.\n\nLa reunión quedó para el martes.\n",
        "Uno.\n\nCompramos pan, queso y vino.\n",
    );

    assert_eq!(
        said.html,
        "<p>Uno.</p><p data-change=\"gone\">La reunión quedó para el martes.</p>\
         <p data-change=\"came\">Compramos pan, queso y vino.</p>"
    );
}

#[test]
fn what_went_is_told_before_what_came_in_its_place() {
    let said = page(
        "# Título\n\nTexto.\n",
        "# Otro nombre\n\nAlgo nuevo y distinto.\n\nTexto.\n",
    );

    assert_eq!(
        said.html,
        "<h1 data-change=\"gone\">Título</h1><h1 id=\"otro-nombre\" data-change=\"came\">Otro nombre</h1>\
         <p data-change=\"came\">Algo nuevo y distinto.</p><p>Texto.</p>"
    );
}

#[test]
fn a_paragraph_that_changed_under_a_new_one_is_still_told_by_its_words() {
    let said = page(
        "Inicio.\n\nEl gato come pescado fresco hoy.\n\nFin.\n",
        "Inicio.\n\nUna línea nueva.\n\nEl gato come pollo fresco hoy.\n\nFin.\n",
    );

    assert_eq!(
        said.html,
        "<p>Inicio.</p><p data-change=\"came\">Una línea nueva.</p>\
         <p data-change=\"gone\">El gato come <del>pescado</del> fresco hoy.</p>\
         <p data-change=\"came\">El gato come <ins>pollo</ins> fresco hoy.</p><p>Fin.</p>"
    );
    assert_eq!((said.gone, said.came), (1, 2));
}

#[test]
fn an_item_that_came_first_in_a_numbered_list_marks_no_other() {
    let said = page(
        "1. uno\n2. dos\n3. tres\n",
        "1. cero\n2. uno\n3. dos\n4. tres\n",
    );

    assert_eq!(
        said.html,
        "<ol><li data-change=\"came\" value=\"1\">cero</li><li value=\"2\">uno</li>\
         <li value=\"3\">dos</li><li value=\"4\">tres</li></ol>"
    );
    assert_eq!((said.gone, said.came), (0, 1));
}

#[test]
fn a_numbered_item_that_changed_keeps_its_number_before_and_after() {
    let said = page(
        "3. uno\n4. dos de antes\n5. tres\n",
        "3. uno\n4. dos de ahora\n5. tres\n",
    );

    assert_eq!(
        said.html,
        "<ol start=\"3\"><li value=\"3\">uno</li>\
         <li data-change=\"gone\" value=\"4\">dos de <del>antes</del></li>\
         <li data-change=\"came\" value=\"4\">dos de <ins>ahora</ins></li>\
         <li value=\"5\">tres</li></ol>"
    );
}

#[test]
fn two_numbered_items_are_not_alike_for_sharing_a_number() {
    let said = page("1. uno\n2. dos\n3. tres\n", "1. uno\n2. two\n3. tres\n");

    assert_eq!(
        said.html,
        "<ol><li value=\"1\">uno</li><li data-change=\"gone\" value=\"2\">dos</li>\
         <li data-change=\"came\" value=\"2\">two</li><li value=\"3\">tres</li></ol>"
    );
}

#[test]
fn an_item_that_holds_paragraphs_is_drawn_once_around_the_one_that_changed() {
    let said = page(
        "- uno\n\n  sigue aqui\n- dos\n",
        "- uno\n\n  sigue alla\n- dos\n",
    );

    assert_eq!(
        said.html,
        "<ul><li><p>uno</p><p data-change=\"gone\">sigue <del>aqui</del></p>\
         <p data-change=\"came\">sigue <ins>alla</ins></p></li><li><p>dos</p></li></ul>"
    );
}

#[test]
fn a_quote_and_a_fold_are_drawn_once_around_what_changed_inside() {
    let quote = page(
        "> cita uno\n>\n> cita dos\n",
        "> cita uno\n>\n> cita tres\n",
    );
    let fold = page(
        "<details>\n<summary>Ver</summary>\n\nDentro uno.\n\n</details>\n",
        "<details>\n<summary>Ver</summary>\n\nDentro dos.\n\n</details>\n",
    );

    assert_eq!(
        quote.html,
        "<blockquote><p>cita uno</p><p data-change=\"gone\">cita <del>dos</del></p>\
         <p data-change=\"came\">cita <ins>tres</ins></p></blockquote>"
    );
    assert_eq!(
        fold.html,
        "<details><summary>Ver</summary><p data-change=\"gone\">Dentro <del>uno.</del></p>\
         <p data-change=\"came\">Dentro <ins>dos.</ins></p></details>"
    );
}

#[test]
fn a_fold_with_another_summary_is_told_whole() {
    let said = page(
        "<details>\n<summary>Ver esto</summary>\n\nDentro va lo mismo.\n\n</details>\n",
        "<details>\n<summary>Ver aquello</summary>\n\nDentro va lo mismo.\n\n</details>\n",
    );

    assert_eq!(
        said.html,
        "<details data-change=\"gone\"><summary>Ver esto</summary><p>Dentro va lo mismo.</p></details>\
         <details data-change=\"came\"><summary>Ver aquello</summary><p>Dentro va lo mismo.</p></details>"
    );
}

#[test]
fn a_footnote_keeps_its_place_around_the_words_that_changed() {
    let said = page(
        "Texto[^a].\n\n[^a]: La nota vieja de antes.\n",
        "Texto[^a].\n\n[^a]: La nota nueva de antes.\n",
    );

    assert!(
        said.html.contains(
            "<div class=\"footnote\" id=\"fn-a\"><span class=\"fn-label\">a</span>\
             <p data-change=\"gone\">La nota <del>vieja</del> de antes.</p>\
             <p data-change=\"came\">La nota <ins>nueva</ins> de antes.</p></div>"
        ),
        "{}",
        said.html
    );
}

#[test]
fn a_mark_of_the_byte_order_at_the_top_changes_nothing() {
    let now = "# Uno\n\nTexto.\n";

    let said = compared(&format!("\u{feff}{now}"), now).unwrap();

    assert_eq!(said.html, drawn(now));
}

#[test]
fn one_block_written_where_many_went_is_told_with_all_of_them() {
    let was = paragraphs(40, "línea vieja");
    let now = "Una sola línea nueva.\n";

    let said = page(&was, now);

    assert_eq!((said.gone, said.came), (40, 1));
    assert!(
        said.html
            .ends_with("<p data-change=\"came\">Una sola línea nueva.</p>")
    );
}

#[test]
fn an_item_under_another_that_changed_leaves_the_one_above_unmarked() {
    let said = page(
        "- padre\n  - hijo uno\n  - hijo dos\n- otro\n",
        "- padre\n  - hijo uno cambiado\n  - hijo dos\n- otro\n",
    );

    assert_eq!(
        said.html,
        "<ul><li>padre<ul><li data-change=\"gone\">hijo uno</li>\
         <li data-change=\"came\">hijo uno <ins>cambiado</ins></li><li>hijo dos</li></ul></li><li>otro</li></ul>"
    );
}

#[test]
fn a_step_ticked_is_told_as_the_box_it_was_and_the_box_it_is() {
    let said = page("- [ ] dos\n- tres\n", "- [x] dos\n- tres\n");

    assert_eq!(
        said.html,
        "<ul><li class=\"task\" data-change=\"gone\"><span class=\"check\" data-done=\"false\"></span>dos</li>\
         <li class=\"task\" data-change=\"came\"><span class=\"check\" data-done=\"true\"></span>dos</li>\
         <li>tres</li></ul>"
    );
}

#[test]
fn a_list_turned_numbered_is_told_whole() {
    let said = page("- uno\n- dos\n", "1. uno\n2. dos\n");

    assert_eq!(
        said.html,
        "<ul data-change=\"gone\"><li>uno</li><li>dos</li></ul>\
         <ol data-change=\"came\"><li>uno</li><li>dos</li></ol>"
    );
}

#[test]
fn a_table_tells_the_row_that_came_and_the_cell_that_changed() {
    let said = page(
        "| A | B {number} |\n|---|---|\n| x | 1 |\n| y | 2 |\n",
        "| A   | B {number} |\n|---|---|\n| x   | 1 |\n| y   | 5 |\n| z   | 3 |\n",
    );

    let cell = "class=\"whole\" style=\"text-align:right\"";
    assert_eq!(
        said.html,
        format!(
            "<div class=\"table\"><table><colgroup><col><col></colgroup>\
             <thead><tr><th scope=\"col\">A</th>\
             <th scope=\"col\" data-type=\"number\" style=\"text-align:right\">B</th></tr></thead>\
             <tbody><tr><td>x</td><td data-key=\"1\" {cell}>1</td></tr>\
             <tr data-change=\"gone\"><td>y</td><td data-key=\"2\" data-change=\"gone\" {cell}><del>2</del></td></tr>\
             <tr data-change=\"came\"><td>y</td><td data-key=\"5\" data-change=\"came\" {cell}><ins>5</ins></td></tr>\
             <tr data-change=\"came\"><td>z</td><td data-key=\"3\" {cell}>3</td></tr>\
             </tbody></table></div>"
        )
    );
    assert_eq!((said.gone, said.came), (1, 2));
}

#[test]
fn a_table_only_padded_again_shows_no_change() {
    let now = "| A   | B   |\n|---|---|\n| x   | 1   |\n";

    let said = compared("| A | B |\n|---|---|\n| x | 1 |\n", now).unwrap();

    assert_eq!(said.html, drawn(now));
    assert_eq!((said.gone, said.came), (0, 0));
}

#[test]
fn a_table_with_another_head_is_told_whole() {
    let said = page(
        "| A | B |\n|---|---|\n| x | 1 |\n",
        "| A | C |\n|---|---|\n| x | 1 |\n",
    );

    assert_eq!(said.html.matches("<table>").count(), 2, "{}", said.html);
    assert!(
        said.html
            .starts_with("<div class=\"table\" data-change=\"gone\">"),
        "{}",
        said.html
    );
    assert!(
        said.html
            .contains("</div><div class=\"table\" data-change=\"came\">"),
        "{}",
        said.html
    );
}

#[test]
fn code_is_told_by_the_words_of_the_line_that_changed() {
    let said = page(
        "```rust\nlet a = 1;\nlet b = 2;\n```\n",
        "```rust\nlet a = 1;\nlet b = 3;\n```\n",
    );

    assert_eq!(
        said.html,
        "<figure class=\"code\" data-change=\"gone\"><pre><code class=\"language-rust\">let a = 1;\nlet b = <del>2;</del>\n</code></pre></figure>\
         <figure class=\"code\" data-change=\"came\"><pre><code class=\"language-rust\">let a = 1;\nlet b = <ins>3;</ins>\n</code></pre></figure>"
    );
}

#[test]
fn a_callout_is_drawn_once_around_the_paragraph_that_changed() {
    let said = page(
        "> [!NOTE]\n> una nota vieja de prueba\n>\n> queda\n",
        "> [!NOTE]\n> una nota nueva de prueba\n>\n> queda\n",
    );

    assert_eq!(
        said.html,
        "<blockquote class=\"callout\" data-callout=\"note\">\
         <p data-change=\"gone\">una nota <del>vieja</del> de prueba</p>\
         <p data-change=\"came\">una nota <ins>nueva</ins> de prueba</p><p>queda</p></blockquote>"
    );
}

#[test]
fn the_properties_that_changed_are_told_as_they_were_and_as_they_are() {
    let said = page(
        "---\ntags: uno\n---\n\nTexto.\n",
        "---\ntags: uno, dos\n---\n\nTexto.\n",
    );

    assert_eq!(said.html.matches("<dl class=\"front\"").count(), 2);
    assert!(
        said.html
            .starts_with("<dl class=\"front\" data-change=\"gone\">"),
        "{}",
        said.html
    );
    assert_eq!((said.gone, said.came), (1, 1));
}

#[test]
fn what_went_names_no_place_the_page_already_names() {
    let said = page(
        "# Uno\n\nTexto[^a] de siempre.\n\n[^a]: La nota vieja de antes.\n",
        "# Otro\n\nTexto[^a] de siempre.\n\n[^a]: La nota nueva de antes.\n",
    );

    assert_eq!(said.html.matches("id=\"fn-a\"").count(), 1, "{}", said.html);
    assert_eq!(said.html.matches(" id=\"").count(), 2, "{}", said.html);
    assert!(said.html.contains("<h1 data-change=\"gone\">Uno</h1>"));
    assert!(
        said.html
            .contains("<h1 id=\"otro\" data-change=\"came\">Otro</h1>")
    );
}

#[test]
fn the_other_line_ending_changes_nothing() {
    let now = "# Acta\n\nUn párrafo\nde dos líneas.\n\n- uno\n- dos\n";

    let said = compared(&now.replace('\n', "\r\n"), now).unwrap();

    assert_eq!(said.html, drawn(now));
    assert_eq!((said.gone, said.came), (0, 0));
}

fn paragraphs(many: usize, said: &str) -> String {
    (0..many)
        .map(|at| format!("{said} {at}.\n\n"))
        .collect::<String>()
}

#[test]
fn two_long_bodies_with_nothing_in_common_are_too_long_to_tell() {
    let (was, now) = (paragraphs(4_100, "antes"), paragraphs(4_100, "ahora"));

    assert_eq!(compared(&was, &now), None);
}

#[test]
fn a_small_change_in_a_long_body_is_told_whatever_its_length() {
    let was = paragraphs(9_000, "línea");
    let now = was.replace("línea 4500.", "línea cambiada 4500.");

    let said = compared(&was, &now).unwrap();

    assert_eq!((said.gone, said.came), (1, 1));
    assert!(
        said.html.contains("línea <ins>cambiada</ins> 4500.</p>"),
        "the words of the one that changed"
    );
}

#[test]
fn a_body_of_the_longest_document_known_written_all_over_is_still_told() {
    let line =
        "es un párrafo de largo corriente, con sus comas y su punto final, que llena la línea";
    let (was, now) = (
        paragraphs(900, &format!("Antes {line}")),
        paragraphs(900, &format!("Ahora {line}")),
    );
    assert!(was.len() > 84_000, "{}", was.len());

    let said = compared(&was, &now).unwrap();

    assert_eq!((said.gone, said.came), (900, 900));
    assert!(balanced(&said.html));
    assert_eq!(said.html.matches("<del>Antes</del>").count(), 900);
}

fn rolled(seed: &mut u64) -> usize {
    crate::shared::tests::rolled(seed) as usize
}

fn some_body(seed: &mut u64) -> String {
    (0..rolled(seed) % 30)
        .map(|_| format!("Párrafo {}.\n\n", rolled(seed) % 7))
        .collect()
}

#[test]
fn what_stayed_and_went_is_the_first_body_and_what_stayed_and_came_the_second() {
    let mut seed = 11;
    for _ in 0..300 {
        let (was, now) = (some_body(&mut seed), some_body(&mut seed));
        let (old, new) = (crate::read(&was), crate::read(&now));
        let texts = (old.text(), new.text());
        let cells = Cell::new(CELLS_AT_MOST);
        let bodies = Bodies {
            was: texts.0,
            now: texts.1,
            cells: &cells,
        };
        let steps = told(old.blocks(), new.blocks(), bodies).unwrap();

        let (mut first, mut second) = (Vec::new(), Vec::new());
        for step in &steps {
            match step {
                Step::Same(block) => {
                    first.push(texts.1[block.at.clone()].trim_end());
                    second.push(texts.1[block.at.clone()].trim_end());
                }
                Step::Gone(block, _) => first.push(texts.0[block.at.clone()].trim_end()),
                Step::Came(block, _) => second.push(texts.1[block.at.clone()].trim_end()),
                Step::Within(gone, came, _) => {
                    first.push(texts.0[gone.at.clone()].trim_end());
                    second.push(texts.1[came.at.clone()].trim_end());
                }
            }
        }
        let said = |doc: &crate::Doc| -> Vec<String> {
            doc.blocks()
                .iter()
                .map(|one| doc.text()[one.at.clone()].trim_end().to_string())
                .collect()
        };
        assert_eq!(first, said(&old), "{was:?} against {now:?}");
        assert_eq!(second, said(&new), "{was:?} against {now:?}");
    }
}

#[test]
fn an_item_is_weighed_without_its_bullet_or_its_number() {
    assert_eq!(unmarked("12. doce"), "doce");
    assert_eq!(unmarked("3) tres"), "tres");
    assert_eq!(unmarked("- [ ] hacer"), "[ ] hacer");
    assert_eq!(unmarked("  * uno"), "uno");
    assert_eq!(unmarked("2024 fue un año"), "2024 fue un año");
}

#[test]
fn a_row_is_weighed_by_its_cells_and_not_by_its_padding() {
    let (tight, wide) = (
        crate::read("| A | B |\n|---|---|\n| x | 1 |\n"),
        crate::read("| A   | B   |\n|---|---|\n| x   | 1   |\n"),
    );
    let row = |doc: &crate::Doc| doc.blocks()[0].inner[1].clone();

    assert!(key(&row(&tight), tight.text()) == key(&row(&wide), wide.text()));
}

const THREE_BEFORE: &str = "uno b c d cinco\n\nseis g h i diez\n\nonce l m n quince\n";
const THREE_AFTER: &str = "UNO b c d CINCO\n\nSEIS g h i DIEZ\n\nONCE l m n QUINCE\n";

#[test]
fn a_page_out_of_cells_tells_the_rest_of_its_blocks_whole() {
    let rich = spending(THREE_BEFORE, THREE_AFTER, 16 + 3 * 36).unwrap();
    let poor = spending(THREE_BEFORE, THREE_AFTER, 16 + 36 + 35).unwrap();

    assert_eq!(rich.html.matches("<ins>").count(), 6, "{}", rich.html);
    assert_eq!(poor.html.matches("<ins>").count(), 2, "{}", poor.html);
    assert!(poor.html.contains("<ins>UNO</ins> b c d <ins>CINCO</ins>"));
    assert_eq!((poor.gone, poor.came), (3, 3));
    assert!(balanced(&poor.html));
}

#[test]
fn only_the_blocks_of_the_whole_page_decide_that_it_is_too_long() {
    assert!(spending(THREE_BEFORE, THREE_AFTER, 16).is_some());
    assert_eq!(spending(THREE_BEFORE, THREE_AFTER, 15), None);
}
