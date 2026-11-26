use tisty_core::arriving::{from_html, from_pasted_html, from_text, tidied};

fn page(markdown: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(markdown))
}

fn pasted_html(html: &str) -> String {
    tidied(&from_pasted_html(html)).body.trim().to_string()
}

fn word_item(list: &str, level: u8, marker: &str, text: &str) -> String {
    format!(
        "<p class=MsoListParagraphCxSpMiddle style='mso-list:{list} level{level} lfo1'><![if !supportLists]><span style='mso-list:Ignore'>{marker}<span style='font:7.0pt \"Times New Roman\"'>&nbsp;&nbsp; </span></span><![endif]>{text}</p>"
    )
}

fn word_note(text: &str) -> String {
    format!("<p class=MsoNormal>{text}</p>")
}

fn word(parts: &[String]) -> String {
    format!("<!--StartFragment-->{}<!--EndFragment-->", parts.concat())
}

#[test]
fn a_word_list_starting_at_three_keeps_its_numbers() {
    let out = pasted_html(&word(&[
        word_item("l0", 1, "3.", "tres"),
        word_item("l0", 1, "4.", "cuatro"),
    ]));
    assert_eq!(out, "3. tres\n4. cuatro");
}

#[test]
fn a_word_list_cut_by_a_paragraph_goes_on_counting() {
    let out = pasted_html(&word(&[
        word_item("l0", 1, "1.", "uno"),
        word_item("l0", 1, "2.", "dos"),
        word_note("Nota intermedia"),
        word_item("l0", 1, "3.", "tres"),
    ]));
    assert_eq!(out, "1. uno\n2. dos\n\nNota intermedia\n\n3. tres");
}

#[test]
fn a_word_outline_number_stays_as_words() {
    let out = pasted_html(&word(&[
        word_item("l0", 1, "1.1.", "Alcance"),
        word_item("l0", 1, "1.2.", "Plazos"),
    ]));
    assert!(out.contains("1.1. Alcance"), "{out:?}");
    assert!(out.contains("1.2. Plazos"), "{out:?}");
}

#[test]
fn a_word_article_marker_stays_with_every_word_and_digit() {
    let out = pasted_html(&word(&[
        word_item("l0", 1, "Artículo 1.", "Objeto"),
        word_item("l0", 1, "Artículo 2.", "Precio"),
    ]));
    assert!(out.contains("Artículo 1. Objeto"), "{out:?}");
    assert!(out.contains("Artículo 2. Precio"), "{out:?}");
}

#[test]
fn a_deep_bullet_after_a_words_paragraph_starts_at_the_top() {
    let md = pasted_html(&word(&[
        word_item("l0", 1, "a)", "primero"),
        word_item("l1", 3, "·", "hondo"),
    ]));
    let shown = page(&md);
    assert_eq!(shown.matches("<li").count(), 1, "{md:?} -> {shown:?}");
    assert!(md.contains("a) primero"), "{md:?}");
    assert!(md.contains("- hondo"), "{md:?}");
}

#[test]
fn a_nested_numbered_sublist_starting_at_two_gets_its_start() {
    let md = pasted_html(&word(&[
        word_item("l0", 1, "1.", "uno"),
        word_item("l0", 2, "2.", "sub dos"),
        word_item("l0", 2, "3.", "sub tres"),
    ]));
    let shown = page(&md);
    assert!(shown.contains("start=\"2\""), "{md:?} -> {shown:?}");
}

#[test]
fn a_readme_typed_as_text_is_not_a_terminal_session() {
    let text = "# Instalar\n\nPasos:\n\n- Clona el repo\n- Entra en la carpeta\n\n    $ npm install\n\nLuego **abre** la app.";
    let out = from_text(text);
    assert!(
        out.as_deref().is_none_or(|md| !md.contains("```")),
        "{out:?}"
    );
}

#[test]
fn a_prompt_inside_prose_is_not_a_terminal_session() {
    let text = "Para ver procesos:\n\nPS C:\\Users\\ana> Get-Process\n\nY listo, **fácil**.";
    let out = from_text(text);
    assert!(
        out.as_deref().is_none_or(|md| !md.contains("```")),
        "{out:?}"
    );
}

#[test]
fn an_arrow_after_a_folder_is_prose_not_a_prompt() {
    let text = "Notas del viaje\nC:\\fotos -> respaldo listo\nTodo bien";
    let out = from_text(text);
    assert!(
        out.as_deref().is_none_or(|md| !md.contains("```")),
        "{out:?}"
    );
}

#[test]
fn a_session_opening_with_a_prompt_is_one_fenced_block() {
    let text = "PS C:\\> Get-Process\nuno\ndos\ntres";
    let out = from_text(text).unwrap_or_default();
    assert!(
        out.starts_with("```") && out.contains("Get-Process"),
        "{out:?}"
    );
}

#[test]
fn two_prompt_lines_with_output_between_are_one_fenced_block() {
    let text = "Ejemplo\nPS C:\\> Get-Date\nlunes\nPS C:\\> Get-Location\nC:\\";
    let out = from_text(text).unwrap_or_default();
    assert!(out.contains("```"), "{out:?}");
}

#[test]
fn bullet_glyph_lines_with_commas_are_a_list_not_a_table() {
    let text = "• rojo, verde, azul\n• uno, dos, tres\n• sol, luna, mar";
    let out = from_text(text);
    assert!(out.as_deref().is_none_or(|md| !md.contains('|')), "{out:?}");
}

#[test]
fn comma_separated_rows_still_make_a_table() {
    let out = from_text("a,b,c\n1,2,3\n4,5,6").unwrap_or_default();
    assert!(out.contains('|'), "{out:?}");
}

fn chat(lines: &[&str]) -> String {
    let html: String = lines.iter().map(|l| format!("<div>{l}</div>")).collect();
    pasted_html(&html)
}

#[test]
fn a_comment_typed_in_chat_stays_visible() {
    let md = chat(&["- no borres &lt;!-- esto --&gt; por favor", "- fin"]);
    let shown = page(&md);
    assert!(shown.contains("&lt;!-- esto --&gt;"), "{md:?} -> {shown:?}");
    assert!(shown.contains("<ul"), "{shown:?}");
}

#[test]
fn tags_typed_in_chat_stay_letters() {
    let md = chat(&["- &lt;u&gt;x&lt;/u&gt;", "- &lt;kbd&gt;Ctrl&lt;/kbd&gt;"]);
    let shown = page(&md);
    assert!(
        !shown.contains("<u>") && !shown.contains("<kbd"),
        "{md:?} -> {shown:?}"
    );
    assert!(shown.contains("&lt;u&gt;x&lt;/u&gt;"), "{shown:?}");
    assert!(shown.contains("&lt;kbd&gt;Ctrl&lt;/kbd&gt;"), "{shown:?}");
}

#[test]
fn a_windows_path_typed_in_chat_keeps_its_backslashes() {
    let md = chat(&["- ruta C:\\new\\*.md", "- fin"]);
    let shown = page(&md);
    assert!(shown.contains("C:\\new\\*.md"), "{md:?} -> {shown:?}");
}

#[test]
fn dollar_prices_typed_in_chat_are_not_math() {
    let md = chat(&["- cuesta $5 y luego $10", "- fin"]);
    let shown = page(&md);
    assert!(shown.contains("$5 y luego $10"), "{md:?} -> {shown:?}");
    assert!(
        !shown.contains("math") && !shown.contains("katex"),
        "{shown:?}"
    );
}

#[test]
fn an_emoji_code_and_a_mark_typed_in_chat_stay_text() {
    let md = chat(&["- listo :tada: y a == b", "- fin"]);
    let shown = page(&md);
    assert!(shown.contains(":tada:"), "{md:?} -> {shown:?}");
    assert!(!shown.contains("<mark"), "{shown:?}");
}

#[test]
fn a_code_span_typed_in_chat_keeps_its_text_without_backslashes() {
    let md = chat(&["- usa `a&lt;b` ahora", "- fin"]);
    let shown = page(&md);
    assert!(shown.contains("<code>a&lt;b</code>"), "{md:?} -> {shown:?}");
}

fn vscode_line(text: &str) -> String {
    format!(
        "<div style=\"white-space: pre;\"><div><span style=\"color: #6a9955;\">{text}</span></div></div>"
    )
}

#[test]
fn a_vscode_line_opening_a_heading_stays_literal() {
    let md = pasted_html(&vscode_line("# TODO: arreglar"));
    let shown = page(&md);
    assert!(!shown.contains("<h1"), "{md:?} -> {shown:?}");
    assert!(shown.contains("# TODO: arreglar"), "{shown:?}");
}

#[test]
fn a_vscode_line_opening_a_list_stays_literal() {
    let md = pasted_html(&vscode_line("- name: x"));
    let shown = page(&md);
    assert!(!shown.contains("<li"), "{md:?} -> {shown:?}");
    assert!(shown.contains("- name: x"), "{shown:?}");
}

fn on_a_small_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(work)
        .expect("thread starts")
        .join()
        .expect("no overflow")
}

#[test]
fn three_thousand_nested_divs_fall_back_to_the_plain_text() {
    let html = format!("{}x{}", "<div>".repeat(3000), "</div>".repeat(3000));
    assert_eq!(on_a_small_stack(move || from_pasted_html(&html)), "");
}

#[test]
fn a_hundred_nested_divs_still_say_their_words() {
    let html = format!("{}hola{}", "<div>".repeat(100), "</div>".repeat(100));
    let out = on_a_small_stack(move || from_pasted_html(&html));
    assert!(out.contains("hola"), "{out:?}");
}

#[test]
fn an_imported_file_keeps_a_dash_and_stars_as_letters() {
    let md = tidied(&from_html("<p>- nota</p><p>2*3*4</p>")).body;
    let shown = page(&md);
    assert!(shown.contains("- nota"), "{md:?} -> {shown:?}");
    assert!(
        !shown.contains("<li") && !shown.contains("<em"),
        "{md:?} -> {shown:?}"
    );
    assert!(shown.contains("2*3*4"), "{md:?} -> {shown:?}");
}

#[test]
fn an_imported_file_never_reads_white_space_pre_as_code() {
    let html = "<div style=\"white-space:pre\">fn a() {}\n  x</div>";
    let imported = from_html(html);
    assert!(!imported.contains("```"), "{imported:?}");
    let pasted = from_pasted_html(html);
    assert!(pasted.contains("```"), "{pasted:?}");
}

#[test]
fn a_drive_path_with_an_arrow_typed_as_text_is_not_a_session() {
    let out = from_text("C:\\datos -> D:\\respaldo\nrevisar mañana");
    assert!(
        out.as_deref().is_none_or(|md| !md.contains("```")),
        "{out:?}"
    );
}

#[test]
fn a_picture_chosen_alone_is_an_image_link() {
    let body = tidied(&from_pasted_html(
        r#"<img src="https://ejemplo.cl/a.png" alt="Logo">"#,
    ))
    .body;
    assert_eq!(body.trim(), "![Logo](https://ejemplo.cl/a.png)");
}
