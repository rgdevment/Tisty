use tisty_core::arriving::{from_pasted_html, from_plain, from_text, tidied};

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
const VSCODE_SCRIPT: &str = r##"<div style="white-space: pre;"><div><span style="color: #6a9955;"># deps</span></div><div><span>npm install</span></div><div><span style="color: #6a9955;"># build</span></div><div><span>npm run build</span></div></div>"##;
const WORD_LIST: &str = "<p class=MsoListParagraphCxSpFirst style='mso-list:l0 level1 lfo1'><span style='mso-list:Ignore'>·<span>&nbsp;&nbsp; </span></span>Primero</p><p class=MsoListParagraphCxSpLast style='mso-list:l0 level1 lfo1'><span style='mso-list:Ignore'>·<span>&nbsp; </span></span>Segundo</p>";
const CODE_EDITOR: &str = "<div style=\"white-space: pre;\"><div><span style=\"color:#569cd6;\"># Título</span></div></div>";

fn rendered(markdown: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(markdown))
}

fn unescaped(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn alphanumerics(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn tag_stripped(html: &str) -> String {
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
    text
}

/// What a person sees of the rendered page, reduced to its letters and digits.
fn visible(markdown: &str) -> String {
    alphanumerics(&unescaped(&tag_stripped(&rendered(markdown))))
}

fn without_between(html: &str, open: &str, close: &str) -> String {
    let lowered = html.to_ascii_lowercase();
    let mut kept = String::new();
    let mut at = 0;
    while let Some(start) = lowered[at..].find(open) {
        let start = at + start;
        kept.push_str(&html[at..start]);
        at = match lowered[start..].find(close) {
            Some(end) => start + end + close.len(),
            None => html.len(),
        };
    }
    kept.push_str(&html[at..]);
    kept
}

/// Word's bullet or plain number sits in a span marked `mso-list:Ignore`; nobody typed it.
/// Any other marker (`a)`, `1.1.`, `Artículo 1.`) stays in the page as words.
fn is_drawn_by_the_list(marker: &str) -> bool {
    let said = marker.trim();
    let digits = said.trim_end_matches(['.', ')']);
    let plain_number = said.len() == digits.len() + 1
        && !digits.is_empty()
        && digits.chars().all(|one| one.is_ascii_digit());
    let symbol = said.chars().count() == 1 && !said.chars().all(char::is_alphanumeric);
    plain_number || symbol || said == "o" || said.is_empty()
}

fn without_list_markers(html: &str) -> String {
    let mut html = html.to_string();
    loop {
        let lowered = html.to_ascii_lowercase();
        let Some(mark) = lowered.find("mso-list:ignore") else {
            return html;
        };
        let Some(start) = lowered[..mark].rfind("<span") else {
            return html;
        };
        let mut depth = 0usize;
        let mut at = start;
        let end = loop {
            let open = lowered[at..].find("<span").map(|n| at + n);
            let close = lowered[at..].find("</span").map(|n| at + n);
            match (open, close) {
                (Some(o), Some(c)) if o < c => {
                    depth += 1;
                    at = o + 5;
                }
                (_, Some(c)) => {
                    depth -= 1;
                    let after = lowered[c..].find('>').map_or(html.len(), |n| c + n + 1);
                    if depth == 0 {
                        break after;
                    }
                    at = after;
                }
                _ => break html.len(),
            }
        };
        let marker = unescaped(&tag_stripped(&html[start..end]));
        let kept = if is_drawn_by_the_list(&marker) {
            String::new()
        } else {
            marker
        };
        html.replace_range(start..end, &kept);
    }
}

/// The letters and digits a person sees in what was copied.
fn letters(html: &str) -> String {
    let body = without_between(html, "<head", "</head>");
    let body = without_between(&body, "<style", "</style>");
    let body = without_between(&body, "<script", "</script>");
    let body = without_between(&body, "<!--", "-->");
    let body = without_list_markers(&body);
    alphanumerics(&unescaped(&tag_stripped(&body)))
}

fn converted_html(html: &str) -> String {
    tidied(&from_pasted_html(html)).body
}

fn pasted_text(text: &str) -> String {
    from_text(text).unwrap_or_else(|| text.to_string())
}

const HTML_FIXTURES: [&str; 14] = [
    WORD_LISTS,
    CHROME_PAGE,
    VSCODE_RUST,
    VSCODE_MARKDOWN,
    VSCODE_PYTHON,
    TERMINAL_HTML,
    VSCODE_SCRIPT,
    WORD_LIST,
    CODE_EDITOR,
    "<p># Título</p><p>Texto **negrita**</p>",
    "<p># Título<br>Texto **negrita**</p>",
    r##"<div style="white-space: pre;"><div># Instalar</div><br><div>```</div><div>$ npm install</div><div>```</div></div>"##,
    "<div><p>- uno</p><p>- dos</p></div>",
    "<p>Calle 1<br>Ciudad</p>",
];

const PLAIN_FIXTURES: [&str; 12] = [
    MARKDOWN_TEXT,
    TERMINAL_CRLF,
    "Hola\r\n\r\nUn parrafo\r\ncon dos lineas\r\n",
    "- rojo, verde, azul\n- uno, dos, tres\n- a, b, c",
    "* rojo, verde, azul\n* uno, dos, tres\n* a, b, c",
    "- rojo, verde, azul\r\n- uno, dos, tres\r\n- a, b, c",
    "nombre,edad,ciudad\nAna,30,Lima\nLuis,25,Quito",
    "El total fue alto.\n$ 100 por persona.\nSe pago al final.",
    "PS C:\\Users\\ana> Get-Process",
    "user@host:~$ ls",
    "# Instalar\n\n```sh\n$ npm install\n```\n",
    "steps:\n  - name: build\n    run: cargo build",
];

#[test]
fn every_html_fixture_keeps_all_the_text_a_person_saw() {
    for html in HTML_FIXTURES {
        assert_eq!(
            letters(html),
            visible(&converted_html(html)),
            "html: {html:?}\nconverted: {:?}",
            converted_html(html)
        );
    }
}

#[test]
fn every_plain_fixture_keeps_all_the_text_a_person_saw() {
    for text in PLAIN_FIXTURES {
        let out = pasted_text(text);
        // Numbered markers and fence languages are marks, not text, once drawn.
        let said = if from_plain(text).is_some() {
            visible(&out)
        } else {
            alphanumerics(&out)
        };
        assert_eq!(
            alphanumerics(text),
            said,
            "plain: {text:?}\nconverted: {out:?}"
        );
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

const WORDS: [&str; 16] = [
    "canción",
    "año",
    "niño2",
    "pingüino",
    "ñandú",
    "café",
    "número",
    "tres3",
    "acción",
    "mañana",
    "cuarto",
    "señal7",
    "árbol",
    "último",
    "París",
    "lluvia",
];

fn phrase(rng: &mut Rng) -> String {
    let count = 1 + rng.below(4);
    (0..count)
        .map(|_| rng.pick(&WORDS))
        .collect::<Vec<_>>()
        .join(" ")
}

fn word_list(rng: &mut Rng) -> String {
    let rows = 2 + rng.below(5);
    let mut out = String::from(
        "<html><head><style><!-- @list l0:level1 {mso-level-number-format:bullet;} --></style></head><body><!--StartFragment-->",
    );
    for at in 0..rows {
        let level = 1 + rng.below(3);
        let glyph = rng.pick(&["·", "o", "§", "1.", "a)"]);
        let place = match at {
            0 => "First",
            n if n + 1 == rows => "Last",
            _ => "Middle",
        };
        out += &format!(
            "<p class=MsoListParagraphCxSp{place} style='mso-list:l0 level{level} lfo1'><![if !supportLists]><span style='mso-list:Ignore'>{glyph}<span style='font:7.0pt \"Times New Roman\"'>&nbsp;&nbsp;&nbsp; </span></span><![endif]>{}<o:p></o:p></p>",
            phrase(rng)
        );
    }
    out + "<!--EndFragment--></body></html>"
}

fn web_page(rng: &mut Rng) -> String {
    let mut out = String::from("<meta charset='utf-8'>");
    for _ in 0..1 + rng.below(3) {
        match rng.below(5) {
            0 => out += &format!("<h2 style=\"color: red;\">{}</h2>", phrase(rng)),
            1 => {
                out += &format!(
                    "<p>{} <strong>{}</strong> y {}</p>",
                    phrase(rng),
                    phrase(rng),
                    phrase(rng)
                )
            }
            2 => {
                out += &format!(
                    "<p>{} <a href=\"https://example.com/x\">{}</a>.</p>",
                    phrase(rng),
                    phrase(rng)
                )
            }
            3 => {
                out += "<ul>";
                for _ in 0..2 + rng.below(3) {
                    out += &format!("<li>{}</li>", phrase(rng));
                }
                out += "</ul>";
            }
            _ => {
                out += "<table><thead><tr><th>Nombre</th><th>Edad</th></tr></thead><tbody>";
                for _ in 0..1 + rng.below(3) {
                    out += &format!(
                        "<tr><td>{}</td><td>{}</td></tr>",
                        phrase(rng),
                        18 + rng.below(60)
                    );
                }
                out += "</tbody></table>";
            }
        }
    }
    out
}

fn markdown_lines(rng: &mut Rng) -> Vec<String> {
    let mut lines = vec![phrase(rng)];
    for _ in 0..2 + rng.below(5) {
        match rng.below(5) {
            0 => lines.push(format!("# {}", phrase(rng))),
            1 => lines.push(format!(
                "{} **{}** y {}",
                phrase(rng),
                phrase(rng),
                phrase(rng)
            )),
            2 => lines.push(format!("- {}", phrase(rng))),
            3 => lines.push(String::new()),
            _ => lines.push(phrase(rng)),
        }
    }
    lines
}

fn typed_markdown(rng: &mut Rng) -> String {
    let wrapper = rng.pick(&["p", "div"]);
    markdown_lines(rng)
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| format!("<{wrapper}>{line}</{wrapper}>"))
        .collect()
}

fn code_lines(rng: &mut Rng) -> Vec<String> {
    let kind = rng.below(4);
    let mut lines = Vec::new();
    for at in 0..3 + rng.below(4) {
        let said = phrase(rng);
        let line = match kind {
            0 => format!(
                "{}fn {}() {{ x &lt;= {at} }}",
                "    ".repeat(at % 2),
                said.replace(' ', "_")
            ),
            1 => markdown_lines(rng)
                .into_iter()
                .find(|l| !l.is_empty())
                .unwrap_or(said),
            2 => format!("{}{}: {at}", "  ".repeat(at % 3), said.replace(' ', "_")),
            _ => match at % 2 {
                0 => format!("# {said}"),
                _ => format!("npm run {}", said.replace(' ', "-")),
            },
        };
        lines.push(line);
    }
    lines
}

fn holder(lines: &[String]) -> String {
    let rows: String = lines
        .iter()
        .map(|one| match one.is_empty() {
            true => "<br>".to_string(),
            false => format!("<div><span style=\"color: #cccccc;\">{one}</span></div>"),
        })
        .collect();
    format!(
        "<meta charset='utf-8'><div style=\"font-family: Consolas;white-space: pre;\">{rows}</div>"
    )
}

fn terminal_lines(rng: &mut Rng) -> Vec<String> {
    let mut lines = vec![format!("PS C:\\Users\\ana&gt; Get-{}", rng.pick(&WORDS))];
    lines.push(format!("{}    Id", rng.pick(&WORDS)));
    lines.push("----    --".to_string());
    for _ in 0..1 + rng.below(3) {
        lines.push(format!("{}    {}", rng.pick(&WORDS), rng.below(900)));
    }
    lines
}

fn terminal_holder(lines: &[String]) -> String {
    let rows: String = lines
        .iter()
        .map(|one| format!("<SPAN STYLE=\"color:#CCCCCC;\">{one}</SPAN><BR>"))
        .collect();
    format!(
        "<DIV STYLE=\"display:inline-block;white-space:pre;font-family:'Cascadia Mono';\">{rows}</DIV>"
    )
}

fn plain_markdown(rng: &mut Rng) -> String {
    markdown_lines(rng).join("\n")
}

fn plain_prose(rng: &mut Rng) -> String {
    let mut text = String::new();
    for _ in 0..2 + rng.below(4) {
        text += &phrase(rng);
        text += rng.pick(&["\r\n", "\r\n\r\n"]);
    }
    text
}

fn plain_yaml(rng: &mut Rng) -> String {
    let mut text = String::from("steps:\n");
    for _ in 0..2 + rng.below(3) {
        text += &format!(
            "  - name: {}\n    run: {}\n",
            rng.pick(&WORDS),
            rng.pick(&WORDS)
        );
    }
    text
}

fn plain_terminal(rng: &mut Rng) -> String {
    terminal_lines(rng).join("\r\n").replace("&gt;", ">")
}

fn hard_breaks(rng: &mut Rng) -> String {
    format!(
        "{}  \n{}  \n{}\n\n- {}\n- {}",
        phrase(rng),
        phrase(rng),
        phrase(rng),
        phrase(rng),
        phrase(rng)
    )
}

enum Input {
    Html(String),
    Plain(String),
}

const KINDS: usize = 13;

fn generated(rng: &mut Rng, kind: usize) -> Input {
    match kind {
        0 => Input::Html(word_list(rng)),
        1 => Input::Html(web_page(rng)),
        2 => Input::Html(typed_markdown(rng)),
        3 => Input::Html(holder(&code_lines(rng))),
        4 => Input::Html(holder(&markdown_lines(rng))),
        5 => Input::Html(terminal_holder(&terminal_lines(rng))),
        6 => Input::Plain(plain_markdown(rng)),
        7 => Input::Plain(plain_prose(rng)),
        8 => Input::Plain(plain_yaml(rng)),
        9 => Input::Plain(plain_terminal(rng)),
        10 => Input::Plain(hard_breaks(rng)),
        11 => Input::Html(holder(&code_lines(rng))),
        _ => Input::Plain(plain_markdown(rng) + "\r\n"),
    }
}

const INPUTS: usize = 520;

#[test]
fn generated_pastes_never_lose_a_letter_or_a_digit() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut failures = Vec::new();
    for at in 0..INPUTS {
        match generated(&mut rng, at % KINDS) {
            Input::Html(html) => {
                let out = converted_html(&html);
                assert!(!letters(&html).is_empty(), "empty source: {html:?}");
                if letters(&html) != visible(&out) {
                    failures.push(format!("html: {html:?}\nconverted: {out:?}"));
                }
            }
            Input::Plain(text) => {
                let out = pasted_text(&text);
                if alphanumerics(&text) != visible(&out) {
                    failures.push(format!("plain: {text:?}\nconverted: {out:?}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {INPUTS} lost text, first ones:\n{}",
        failures.len(),
        failures
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n---\n")
    );
}

fn content_lines(text: &str) -> usize {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("```"))
        .count()
}

#[test]
fn code_editors_terminals_and_crlf_text_keep_their_line_count() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut failures = Vec::new();
    for at in 0..INPUTS {
        let (label, source, out) = match at % 5 {
            0 => {
                let lines = code_lines(&mut rng);
                let out = converted_html(&holder(&lines));
                ("code", lines.join("\n"), out)
            }
            1 => {
                let lines = markdown_lines(&mut rng);
                let out = converted_html(&holder(&lines));
                ("markdown", lines.join("\n"), out)
            }
            2 => {
                let lines = terminal_lines(&mut rng);
                let out = converted_html(&terminal_holder(&lines));
                ("terminal", lines.join("\n"), out)
            }
            3 => {
                let text = plain_prose(&mut rng);
                let out = pasted_text(&text);
                ("prose", text, out)
            }
            _ => {
                let text = plain_terminal(&mut rng);
                let out = pasted_text(&text);
                ("plain terminal", text, out)
            }
        };
        let expected = content_lines(&unescaped(&source));
        if content_lines(&out) != expected {
            failures.push(format!(
                "{label}: expected {expected} lines\nsource: {source:?}\nconverted: {out:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {INPUTS} changed their line count, first ones:\n{}",
        failures.len(),
        failures
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n---\n")
    );
}

#[test]
fn a_hard_break_in_a_code_editor_markdown_file_keeps_its_two_spaces() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..50 {
        let first = phrase(&mut rng);
        let second = phrase(&mut rng);
        let lines = vec![
            format!("{first}  "),
            second.clone(),
            String::new(),
            format!("- {second}"),
        ];
        let out = converted_html(&holder(&lines));
        assert!(
            out.contains(&format!("{first}  \n")),
            "lines: {lines:?}\nconverted: {out:?}"
        );
    }
}
