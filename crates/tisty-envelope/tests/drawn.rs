//! What the translator draws, once its references are in place, is what the contract says. Colour, icons and drawings come after, in the window.

use std::path::Path;

use tisty_contracts::check_html;
use tisty_envelope::travelling::{Places, body_ready, named, ready};

const NOT_DRAWN_YET: &str = "A diagram travels drawn, as SVG.";

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn every_real_document() -> Vec<(String, String)> {
    let raw = std::fs::read_to_string(root().join("fixtures/markdown.json")).unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap();
    let mut out: Vec<(String, String)> = cases
        .into_iter()
        .map(|one| {
            let text = match one["file"].as_str() {
                Some(file) => std::fs::read_to_string(root().join(file)).unwrap(),
                None => one["text"].as_str().unwrap().to_owned(),
            };
            (one["name"].as_str().unwrap().to_owned(), text)
        })
        .collect();
    for folder in [
        "docs/help",
        "app/src-tauri/resources/guide/en",
        "app/src-tauri/resources/guide/es",
    ] {
        for file in std::fs::read_dir(root().join(folder)).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|one| one == "md") {
                out.push((
                    path.display().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    assert!(out.len() > 100);
    out
}

fn wrong_in(text: &str) -> Vec<String> {
    let html = tisty_markdown::html(&tisty_markdown::read(text));
    check_html(&ready(&html, &Places::default()))
        .into_iter()
        .filter(|one| one != NOT_DRAWN_YET)
        .collect()
}

#[test]
fn every_document_there_is_speaks_the_contract() {
    for (name, text) in every_real_document() {
        assert_eq!(wrong_in(&text), Vec::<String>::new(), "{name}");
    }
}

#[test]
fn every_formula_the_translator_can_write_speaks_the_contract() {
    let formulas = [
        r"\cancel{x}",
        r"\bcancel{x}",
        r"\xcancel{x}",
        r"\boxed{x}",
        r"\phantom{x}",
        r"\hphantom{x}",
        r"\vphantom{x}",
        r"\sideset{_a^b}{_c^d}\sum",
        r"{}^a_b X",
        r"\color{red}{x}",
        r"\textcolor{red}{x}",
        r"\mathbf{x}",
        r"\mathrm{x}",
        r"\mathbb{R}",
        r"\mathcal{R}",
        r"\mathfrak{R}",
        r"\overline{x}",
        r"\underline{x}",
        r"\widehat{x}",
        r"\vec{x}",
        r"\dot{x}",
        r"\frac{a}{b}",
        r"\dfrac{a}{b}",
        r"\tfrac{a}{b}",
        r"\binom{a}{b}",
        r"\sqrt[3]{x}",
        r"\left( x \right)",
        r"\big( x \big)",
        r"\sum_{i=1}^n",
        r"\int_0^1",
        r"\lim_{x\to 0}",
        r"\overset{a}{b}",
        r"\underset{a}{b}",
        r"\stackrel{a}{b}",
        r"\xrightarrow{a}",
        r"\text{hola}",
        r"\quad\qquad\,\;\!",
        r"\hspace{1em}",
        r"\operatorname{foo}",
        r"\pmod{n}",
        r"\not=",
        r"\substack{a\\b}",
        r"\displaystyle x",
        r"\scriptstyle x",
        r"\overbrace{x}",
        r"\underbrace{x}",
        r"\rule{1em}{1em}",
        r"\mathrel{x}",
        r"\sout{x}",
        r"\undefinedthing{x}",
        r"\tiny x",
        r"\Huge x",
        r"\textsc{x}",
        r"\textsf{x}",
        r"\texttt{x}",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
        r"\begin{bmatrix} a \end{bmatrix}",
        r"\begin{vmatrix} a \end{vmatrix}",
        r"\begin{cases} a & b \\ c & d \end{cases}",
        r"\begin{align} a &= b \\ c &= d \end{align}",
        r"\begin{aligned} a &= b \end{aligned}",
        r"\begin{array}{|c|l|r|} \hline a & b & c \\ \hline \end{array}",
        r"\begin{array}{:c:} \hdashline a \end{array}",
        r"\begin{gather} a \\ b \end{gather}",
        r"\begin{matrix} a \end{matrix}",
        r"\begin{split} a &= b \end{split}",
        r"\begin{alignat}{2} a &= b \end{alignat}",
        r"\begin{subarray}{c} a \end{subarray}",
        r"\begin{smallmatrix} a \end{smallmatrix}",
        r"\begin{multline} a \\ b \end{multline}",
        r"\begin{equation} a \end{equation}",
        r"\raisebox{1em}{x}",
        r"\mathllap{x}",
        r"\mathrlap{x}",
    ];
    for one in formulas {
        assert_eq!(wrong_in(&format!("${one}$")), Vec::<String>::new(), "{one}");
        assert_eq!(
            wrong_in(&format!("$$\n{one}\n$$")),
            Vec::<String>::new(),
            "{one}"
        );
    }
}

#[test]
fn what_a_person_can_write_by_hand_never_falls_outside_the_contract() {
    let written = [
        r#"<div align="center" class="mine" id="x" style="color:red"><img src="https://a.example/x.png" width="100" height="50" align="left"><a href="https://a.example" target="_blank" rel="me" title="t">a</a></div>"#,
        r#"<table align="center"><caption>c</caption><thead><tr><th colspan="2" scope="col">a</th></tr></thead><tbody><tr><td rowspan="1">b</td></tr></tbody><tfoot><tr><td>c</td></tr></tfoot></table>"#,
        "<!-- a comment <script>x</script> -->",
        "<svg><circle r=\"1\"></circle></svg>",
        r#"<video src="a.mp4"></video><iframe src="x"></iframe><style>p{}</style><script>a()</script><form></form><input><button>a</button>"#,
        r#"<map name="x"><area href="otro.html"><area href="https://a.example"></map><q cite="x.html">q</q>"#,
        "[win](C:\\Users\\a.md) [rel](a/b.md) [q](?a=1&b=2) [amp](a&b.md) [tel](tel:+56) [t](tisty:doc/abc) [f](file:///c:/a) [java](javascript:alert(1)) [data](data:text/html,x)",
        "![doc](tisty:doc/abc) ![task](tisty:task/abc) ![rel](rina.png) ![remote](https://a.example/x.png) ![file](manual.pdf \"file\") ![data](data:image/png;base64,AAAA)",
        "[[una referencia]] [[#2]] ==(red)pen== ++under++ :star:red: [^1]\n\n[^1]: nota\n\n[def]: https://a.example \"t\"",
        "---\ntitle: a\ntask: 01M6\ntranslationKey: x\nauthor: b\n---\n# a",
    ];
    for one in written {
        assert_eq!(wrong_in(one), Vec::<String>::new(), "{one}");
    }
}

#[test]
fn every_document_there_is_reads_the_same_once_its_words_travel() {
    for (name, text) in every_real_document() {
        let words = body_ready(&text, &Places::default());
        assert_eq!(wrong_in(&words), Vec::<String>::new(), "{name}");
        assert!(
            !words.contains("tisty:doc/") || words.contains("```"),
            "{name}"
        );
    }
}

#[test]
fn it_finds_the_pictures_a_real_document_names() {
    let html = tisty_markdown::html(&tisty_markdown::read("![a](rina.png) [b](cartilla.pdf)"));
    let found = named(&html);
    assert_eq!(found.pictures, ["rina.png"]);
    assert_eq!(found.links, ["cartilla.pdf"]);
}
