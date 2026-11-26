use tisty_core::arriving::{from_pasted_html, from_plain, from_text, tidied};
use tisty_core::docs::speaks;

const WORD_LISTS: &str = r#"<html xmlns:o="urn:schemas-microsoft-com:office:office"
xmlns:w="urn:schemas-microsoft-com:office:word"
xmlns="http://www.w3.org/TR/REC-html40">
<head>
<meta charset=utf-8>
<style>
<!--
 /* List Definitions */
 @list l0:level1
	{mso-level-number-format:bullet;
	mso-level-text:\F0B7;
	mso-level-tab-stop:36.0pt;
	mso-level-number-position:left;
	text-indent:-18.0pt;
	font-family:Symbol;}
 @list l0:level2
	{mso-level-number-format:bullet;
	mso-level-text:o;
	text-indent:-18.0pt;
	font-family:"Courier New";}
 @list l1:level1
	{mso-level-number-format:alpha-lower;
	mso-level-text:"%1\)";}
-->
</style>
</head>
<body lang=ES>
<!--StartFragment-->
<p class=MsoListParagraphCxSpFirst style='text-indent:-18.0pt;mso-list:l0 level1 lfo1'><![if !supportLists]><span style='font-family:Symbol;mso-fareast-font-family:Symbol;mso-bidi-font-family:Symbol'><span style='mso-list:Ignore'>·<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span></span><![endif]>Primero<o:p></o:p></p>
<p class=MsoListParagraphCxSpMiddle style='margin-left:72.0pt;mso-add-space:auto;text-indent:-18.0pt;mso-list:l0 level2 lfo1'><![if !supportLists]><span style='font-family:"Courier New"'><span style='mso-list:Ignore'>o<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span></span><![endif]>Anidado<o:p></o:p></p>
<p class=MsoListParagraphCxSpMiddle style='text-indent:-18.0pt;mso-list:l0 level1 lfo1'><![if !supportLists]><span style='font-family:Symbol'><span style='mso-list:Ignore'>·<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span></span><![endif]>Con <b>negrita</b> y <span style='color:red'>rojo</span><o:p></o:p></p>
<p class=MsoListParagraphCxSpLast style='text-indent:-18.0pt;mso-list:l0 level1 lfo1'><![if !supportLists]><span style='font-family:Symbol'><span style='mso-list:Ignore'>·<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span></span><![endif]>Ultimo<o:p></o:p></p>
<p class=MsoNormal><o:p>&nbsp;</o:p></p>
<p class=MsoListParagraphCxSpFirst style='text-indent:-18.0pt;mso-list:l1 level1 lfo2'><![if !supportLists]><span style='mso-list:Ignore'>1.<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span><![endif]>Uno<o:p></o:p></p>
<p class=MsoListParagraphCxSpLast style='text-indent:-18.0pt;mso-list:l1 level1 lfo2'><![if !supportLists]><span style='mso-list:Ignore'>2.<span style='font:7.0pt "Times New Roman"'>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; </span></span><![endif]>Dos<o:p></o:p></p>
<!--EndFragment-->
</body>
</html>"#;

const CHROME_PAGE: &str = r#"<meta charset='utf-8'><h2 style="color: rgb(34, 34, 34); font-family: Arial; font-size: 24px;">Guia rapida</h2><p style="margin: 0px;">Esto es <strong>importante</strong> y aqui hay un <a href="https://example.com/doc">enlace</a>.</p><ul><li>uno</li><li>dos</li></ul><table><thead><tr><th>Nombre</th><th>Edad</th></tr></thead><tbody><tr><td>Ana</td><td>30</td></tr><tr><td>Luis</td><td>25</td></tr></tbody></table>"#;

const VSCODE_RUST: &str = r#"<meta charset='utf-8'><div style="color: #cccccc;background-color: #1f1f1f;font-family: Consolas, 'Courier New', monospace;font-weight: normal;font-size: 14px;line-height: 19px;white-space: pre;"><div><span style="color: #569cd6;">fn</span><span style="color: #dcdcaa;"> main</span><span style="color: #cccccc;">() {</span></div><div><span style="color: #cccccc;">    </span><span style="color: #569cd6;">let</span><span style="color: #cccccc;"> x = </span><span style="color: #b5cea8;">1</span><span style="color: #cccccc;">;</span></div><div><span style="color: #cccccc;">}</span></div></div>"#;

const VSCODE_MARKDOWN: &str = r#"<meta charset='utf-8'><div style="color: #cccccc;background-color: #1f1f1f;font-family: Consolas, 'Courier New', monospace;font-weight: normal;font-size: 14px;line-height: 19px;white-space: pre;"><div><span style="color: #569cd6;font-weight: bold;"># Título</span></div><br><div><span style="color: #6796e6;">- </span><span style="color: #cccccc;">uno</span></div><div><span style="color: #6796e6;">- </span><span style="color: #cccccc;">dos</span></div></div>"#;

const VSCODE_PYTHON: &str = r#"<meta charset='utf-8'><div style="color: #cccccc;background-color: #1f1f1f;font-family: Consolas, 'Courier New', monospace;font-weight: normal;font-size: 14px;line-height: 19px;white-space: pre;"><div><span style="color: #6a9955;"># suma dos numeros</span></div><div><span style="color: #569cd6;">def</span><span style="color: #dcdcaa;"> f</span><span style="color: #cccccc;">():</span></div><div><span style="color: #cccccc;">    </span><span style="color: #c586c0;">return</span><span style="color: #cccccc;"> </span><span style="color: #b5cea8;">1</span></div></div>"#;

const TERMINAL_HTML: &str = r#"<DIV STYLE="display:inline-block;white-space:pre;background-color:#0C0C0C;font-family:'Cascadia Mono',monospace;font-size:12pt;padding:4px;"><SPAN STYLE="color:#CCCCCC;background-color:#0C0C0C;">PS C:\Users\ana&gt; Get-Process</SPAN><BR><SPAN STYLE="color:#CCCCCC;background-color:#0C0C0C;">Name    Id</SPAN><BR><SPAN STYLE="color:#CCCCCC;background-color:#0C0C0C;">----    --</SPAN><BR><SPAN STYLE="color:#CCCCCC;background-color:#0C0C0C;">code    42</SPAN><BR></DIV>"#;

const MARKDOWN_TEXT: &str = "# Titulo\n\nUn parrafo con **negrita** y un [enlace](https://example.com).\n\n- uno\n- dos\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n";

const TERMINAL_CRLF: &str =
    "PS C:\\> dir\r\n\r\n    Directory: C:\\\\\r\n\r\nMode   Name\r\n----   ----\r\nd----  x\r\n";

fn spoken(said: &str) -> String {
    assert!(
        speaks(said).is_ok(),
        "still refused: {said:?} ({:?})",
        speaks(said)
    );
    said.to_string()
}

fn html(said: &str) -> String {
    spoken(&tidied(&from_pasted_html(said)).body)
}

#[test]
fn a_word_list_comes_in_as_nested_markdown_lists_without_its_bullet_glyphs() {
    let out = html(WORD_LISTS);
    assert!(out.contains("- Primero\n"), "{out:?}");
    assert!(out.contains("\n  - Anidado\n"), "{out:?}");
    assert!(out.contains("- Con **negrita** y"), "{out:?}");
    assert!(out.contains("- Ultimo"), "{out:?}");
    assert!(out.contains("1. Uno\n"), "{out:?}");
    assert!(out.contains("2. Dos"), "{out:?}");
    assert!(!out.contains('·') && !out.contains("\\·"), "{out:?}");
    assert!(
        !out.contains("mso-list") && !out.contains("StartFragment"),
        "{out:?}"
    );
}

#[test]
fn a_web_page_copied_from_chrome_comes_in_as_markdown_with_its_formatting() {
    let out = html(CHROME_PAGE);
    assert!(out.contains("## Guia rapida"), "{out:?}");
    assert!(out.contains("**importante**"), "{out:?}");
    assert!(out.contains("[enlace](https://example.com/doc)"), "{out:?}");
    assert!(out.contains("- uno\n- dos"), "{out:?}");
    assert!(
        out.contains(
            "| Nombre | Edad |
| ------ | ---- |"
        ),
        "{out:?}"
    );
    assert!(out.contains("| Ana    | 30   |"), "{out:?}");
}

#[test]
fn markdown_typed_in_chat_divs_stays_markdown_with_contiguous_items() {
    let out = from_pasted_html(
        "<meta charset='utf-8'><div># Título</div><div>- uno</div><div>- dos</div>",
    );
    assert_eq!(out.trim_end(), "# Título\n- uno\n- dos");
    for escaped in ["\\#", "\\*", "\\[", "\\-"] {
        assert!(!out.contains(escaped), "{out:?}");
    }
    spoken(&out);
}

#[test]
fn markdown_typed_in_chat_paragraphs_keeps_its_table_rows_together() {
    let out = from_pasted_html("<p>| a | b |</p><p>|---|---|</p><p>| 1 | 2 |</p>");
    assert!(out.contains("| a | b |\n|---|---|\n| 1 | 2 |"), "{out:?}");
    assert!(!out.contains('\\'), "{out:?}");
}

#[test]
fn list_items_in_separate_paragraphs_lose_the_blank_lines_between_them() {
    let out = from_pasted_html("<p>- uno</p><p>- dos</p><p>- tres</p>");
    assert!(out.contains("- uno\n- dos\n- tres"), "{out:?}");
    assert!(!out.contains('\\'), "{out:?}");
}

#[test]
fn code_copied_from_vs_code_is_fenced_with_its_language_and_indentation() {
    let out = from_pasted_html(VSCODE_RUST);
    assert!(out.trim_start().starts_with("```rust\n"), "{out:?}");
    assert!(out.contains("fn main() {\n    let x = 1;\n}"), "{out:?}");
    assert!(!out.contains("=="), "{out:?}");
    spoken(&out);
}

#[test]
fn a_markdown_file_copied_from_vs_code_stays_markdown() {
    let out = from_pasted_html(VSCODE_MARKDOWN);
    assert!(out.contains("# Título"), "{out:?}");
    assert!(out.contains("- uno\n- dos"), "{out:?}");
    assert!(!out.contains("=="), "{out:?}");
    assert!(!out.contains("```"), "{out:?}");
    assert!(!out.contains('\\'), "{out:?}");
}

#[test]
fn python_copied_from_vs_code_is_fenced_and_its_comment_is_not_a_heading() {
    let out = from_pasted_html(VSCODE_PYTHON);
    assert!(out.trim_start().starts_with("```"), "{out:?}");
    assert!(
        out.contains("# suma dos numeros\ndef f():\n    return 1"),
        "{out:?}"
    );
    assert!(out.trim_end().ends_with("```"), "{out:?}");
    assert!(!out.contains("=="), "{out:?}");
}

#[test]
fn output_copied_from_windows_terminal_is_one_fenced_block() {
    let out = from_pasted_html(TERMINAL_HTML);
    assert!(out.trim_start().starts_with("```"), "{out:?}");
    assert!(
        out.contains("PS C:\\Users\\ana> Get-Process\nName    Id\n----    --\ncode    42"),
        "{out:?}"
    );
    assert!(!out.contains("=="), "{out:?}");
    let outside_the_fence = out
        .split("```")
        .enumerate()
        .filter(|(at, _)| at % 2 == 0)
        .all(|(_, part)| !part.contains("---"));
    assert!(outside_the_fence, "{out:?}");
    spoken(&out);
}

#[test]
fn markdown_as_plain_text_is_pasted_as_written() {
    let out = from_text(MARKDOWN_TEXT);
    assert!(
        out.as_deref().is_none_or(|said| said == MARKDOWN_TEXT),
        "{out:?}"
    );
    assert_eq!(from_plain(MARKDOWN_TEXT), None);
}

#[test]
fn terminal_output_with_crlf_is_fenced_without_carriage_returns() {
    let out = from_plain(TERMINAL_CRLF).expect("terminal output is fenced");
    assert!(out.trim_start().starts_with("```"), "{out:?}");
    assert!(!out.contains('\r'), "{out:?}");
    assert!(
        out.contains("Mode   Name\n----   ----\nd----  x"),
        "{out:?}"
    );
}

#[test]
fn prose_with_crlf_comes_out_without_carriage_returns() {
    let out = from_text("Hola\r\n\r\nUn parrafo\r\ncon dos lineas\r\n");
    assert!(
        out.as_deref().is_none_or(|said| !said.contains('\r')),
        "{out:?}"
    );
}

#[test]
fn lists_with_commas_are_not_mistaken_for_a_table() {
    let lists = [
        "- rojo, verde, azul\n- uno, dos, tres\n- a, b, c",
        "1. rojo, verde, azul\n2. uno, dos, tres\n3. a, b, c",
        "* rojo, verde, azul\n* uno, dos, tres\n* a, b, c",
        "- rojo, verde, azul\r\n- uno, dos, tres\r\n- a, b, c",
    ];
    for said in lists {
        assert_eq!(from_plain(said), None, "{said:?}");
    }
}

#[test]
fn a_real_csv_still_becomes_a_table() {
    let out = from_plain("nombre,edad,ciudad\nAna,30,Lima\nLuis,25,Quito").expect("a table");
    assert!(out.contains("| nombre | edad | ciudad |"), "{out:?}");
    assert!(out.contains("| Ana | 30 | Lima |"), "{out:?}");
}

#[test]
fn prose_that_starts_a_line_with_a_dollar_sign_is_not_fenced() {
    let out = from_plain("El total fue alto.\n$ 100 por persona.\nSe pago al final.");
    assert!(
        out.as_deref().is_none_or(|said| !said.contains("```")),
        "{out:?}"
    );
}

#[test]
fn a_single_prompt_line_is_not_fenced() {
    assert_eq!(from_plain("PS C:\\Users\\ana> Get-Process"), None);
    assert_eq!(from_plain("user@host:~$ ls"), None);
}

const VSCODE_SCRIPT: &str = r##"<div style="white-space: pre;"><div><span style="color: #6a9955;"># deps</span></div><div><span>npm install</span></div><div><span style="color: #6a9955;"># build</span></div><div><span>npm run build</span></div></div>"##;

#[test]
fn a_script_whose_comments_sit_on_their_commands_comes_in_fenced() {
    let out = from_pasted_html(VSCODE_SCRIPT);
    assert_eq!(
        out,
        "```sh\n# deps\nnpm install\n# build\nnpm run build\n```"
    );
}

#[test]
fn markdown_that_quotes_a_prompt_in_its_own_fence_stays_markdown() {
    let readme = "# Instalar\n\n```sh\n$ npm install\n```\n";
    assert_eq!(from_plain(readme), None);
    let html = r##"<div style="white-space: pre;"><div># Instalar</div><br><div>```sh</div><div>$ npm install</div><div>```</div></div>"##;
    assert_eq!(
        from_pasted_html(html),
        "# Instalar\n\n```sh\n$ npm install\n```"
    );
}

#[test]
fn one_line_of_code_keeps_its_signs_and_one_line_of_words_is_left_as_it_reads() {
    let line = |said: &str| {
        from_pasted_html(&format!(
            r#"<div style="white-space: pre;"><div><span style="color:#9cdcfe">{said}</span></div></div>"#
        ))
    };
    assert_eq!(line("x = a * b * c"), "`x = a * b * c`");
    assert_eq!(line("def __init__(self):"), "`def __init__(self):`");
    assert_eq!(line("a `b` c"), "``a `b` c``");
    assert_eq!(line("echo `pwd"), "``echo `pwd``");
    assert_eq!(line("x`"), "`` x` ``");
    assert_eq!(line("hola mundo"), "hola mundo");
}

fn laid_out(lines: &[&str]) -> String {
    let rows: String = lines
        .iter()
        .map(|one| match one.is_empty() {
            true => "<br>".to_string(),
            false => format!("<div><span>{one}</span></div>"),
        })
        .collect();
    from_pasted_html(&format!(r#"<div style="white-space: pre;">{rows}</div>"#))
}

#[test]
fn yaml_copied_from_a_code_editor_is_code_and_not_a_list() {
    let out = laid_out(&["steps:", "  - name: build", "    run: cargo build"]);
    assert_eq!(
        out,
        "```\nsteps:\n  - name: build\n    run: cargo build\n```"
    );
}

#[test]
fn markdown_from_a_code_editor_keeps_its_hard_line_breaks() {
    let out = laid_out(&["Calle 1  ", "Ciudad", "", "- uno", "- dos"]);
    assert_eq!(out, "Calle 1  \nCiudad\n\n- uno\n- dos");
}

#[test]
fn a_break_inside_a_paragraph_is_one_line_break() {
    let out = from_pasted_html("<p># Título<br>Texto **negrita**</p>");
    assert_eq!(out, "# Título\nTexto **negrita**");
}

const LITERAL: &str = r#"# Título
- uno
  - dos
12. doce
3) tres
son 12. ok
> cita
| a | b |
==p=={red} ++u++ ~~s~~
::star:: y :::tip
$x$ <b>t</b> [l](u) ![i](u)
`c` &amp; \ fin
**hola** _e_"#;

const ESCAPED: &str = r#"\# Título
\- uno
  \- dos
12\. doce
3\) tres
son 12. ok
\> cita
\| a \| b \|
\=\=p\=\={red} \+\+u\+\+ \~\~s\~\~
\::star\:: y \:\::tip
\$x\$ \<b\>t\</b\> \[l\](u) \!\[i\](u)
\`c\` \&amp; \\ fin
\*\*hola\*\* \_e\_"#;

fn rendered(markdown: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(markdown))
}

fn visible_text(html: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;
    for c in html.chars() {
        match c {
            '<' => {
                inside_tag = true;
                text.push(' ');
            }
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(c),
            _ => {}
        }
    }
    let unescaped = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    unescaped.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

const MARKUP: [&str; 19] = [
    "<strong",
    "<em>",
    "<h1",
    "<h2",
    "<h3",
    "<h4",
    "<h5",
    "<h6",
    "<ul",
    "<ol",
    "<blockquote",
    "<table",
    "<code",
    "<a ",
    "<img",
    "<mark",
    "<u>",
    "<ins",
    "<del",
];

fn markup_found(html: &str) -> Vec<&'static str> {
    let mut found: Vec<&'static str> = MARKUP
        .iter()
        .copied()
        .filter(|m| html.contains(m))
        .collect();
    for extra in ["<s>", "math", "icon", "<b>", "<li"] {
        if html.contains(extra) {
            found.push(extra);
        }
    }
    found
}

#[test]
fn text_pasted_without_formatting_is_drawn_exactly_as_it_was_copied() {
    let html = rendered(ESCAPED);
    assert_eq!(markup_found(&html), Vec::<&str>::new(), "rendered: {html}");
    assert_eq!(visible_text(&html), collapsed(LITERAL));
}

#[test]
fn bold_and_heading_marks_pasted_without_formatting_stay_as_letters() {
    let html = rendered(r"\*\*hola\*\* \# x");
    assert!(!html.contains("<strong"), "rendered: {html}");
    assert!(!html.contains("<h1"), "rendered: {html}");
    assert_eq!(visible_text(&html), "**hola** # x");
}

#[test]
fn text_indented_four_columns_and_pasted_without_formatting_is_not_code() {
    let html = rendered("\u{a0}\u{a0}\u{a0}\u{a0}hola\n\u{a0}\u{a0}\u{a0}\u{a0}mundo");
    assert!(!html.contains("<pre"), "rendered: {html}");
    assert!(!html.contains("<code"), "rendered: {html}");
}

#[test]
fn a_link_scheme_pasted_without_formatting_is_not_linked() {
    let html = rendered(r"ver https\://x.y");
    assert!(!html.contains("<a "), "rendered: {html}");
    assert_eq!(visible_text(&html), "ver https://x.y");
}
