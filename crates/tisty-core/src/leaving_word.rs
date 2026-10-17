use std::io::Write;

use serde_json::Value;

use crate::arriving::WORD_AT_MOST;

const NAMESPACES: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture""#;

// Six inches, the width of a page's writing between its margins.
const WIDEST: u64 = 6 * 914_400;
const EMU_PER_PIXEL: u64 = 9_525;

/// A document and its pages as one Word file, read from the same tree the PDF is made from.
pub fn to_word(
    bodies: &[String],
    picture: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
) -> std::io::Result<Vec<u8>> {
    let mut writing = Writing {
        out: String::new(),
        links: Vec::new(),
        media: Vec::new(),
        numbered: Vec::new(),
        picture,
        left: WORD_AT_MOST,
    };
    for (at, body) in bodies.iter().enumerate() {
        if at > 0 {
            writing.out.push_str(BREAK);
        }
        let tree = tisty_markdown::printed(&tisty_markdown::read(body));
        writing.blocks(content(&tree), &Place::default());
    }
    writing.packed()
}

const BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

#[derive(Default, Clone)]
struct Place {
    depth: u32,
    shade: Option<&'static str>,
    list: Option<(u32, u32)>,
}

struct Writing<'a> {
    out: String,
    links: Vec<String>,
    media: Vec<(String, Vec<u8>, &'static str)>,
    numbered: Vec<u64>,
    picture: &'a mut dyn FnMut(&str) -> Option<Vec<u8>>,
    left: u64,
}

fn content(node: &Value) -> &[Value] {
    node["content"].as_array().map_or(&[], Vec::as_slice)
}

fn kind(node: &Value) -> &str {
    node["type"].as_str().unwrap_or_default()
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn shaded(callout: &str) -> &'static str {
    match callout {
        "tip" => "DCFCE7",
        "important" => "EDE9FE",
        "warning" => "FEF3C7",
        "caution" => "FEE2E2",
        _ => "DBEAFE",
    }
}

impl Writing<'_> {
    fn blocks(&mut self, nodes: &[Value], place: &Place) {
        for node in nodes {
            self.block(node, place);
        }
    }

    fn block(&mut self, node: &Value, place: &Place) {
        match kind(node) {
            "heading" => {
                let level = node["attrs"]["level"].as_u64().unwrap_or(1).clamp(1, 6);
                let said = self.runs(content(node));
                self.paragraph(&format!("Heading{level}"), place, &said);
            }
            "paragraph" => {
                let said = self.runs(content(node));
                self.paragraph("", place, &said);
            }
            "blockquote" | "callout" => {
                let shade = (kind(node) == "callout")
                    .then(|| shaded(node["attrs"]["kind"].as_str().unwrap_or_default()))
                    .or(place.shade);
                let inside = Place {
                    depth: place.depth + 1,
                    shade,
                    list: None,
                };
                self.blocks(content(node), &inside);
            }
            "codeBlock" => self.code(node, place),
            "horizontalRule" => self.out.push_str(BREAK),
            "bulletList" | "orderedList" | "taskList" => self.list(node, place),
            "table" => self.table(node),
            _ => {}
        }
    }

    fn paragraph(&mut self, style: &str, place: &Place, said: &str) {
        let mut props = String::new();
        if !style.is_empty() {
            props.push_str(&format!(r#"<w:pStyle w:val="{style}"/>"#));
        }
        if let Some((id, level)) = place.list {
            props.push_str(&format!(
                r#"<w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{id}"/></w:numPr>"#
            ));
        }
        if let Some(fill) = place.shade {
            props.push_str(&format!(
                r#"<w:shd w:val="clear" w:color="auto" w:fill="{fill}"/>"#
            ));
        }
        if place.depth > 0 && place.list.is_none() {
            props.push_str(&format!(r#"<w:ind w:left="{}"/>"#, place.depth * 567));
        }
        self.out
            .push_str(&format!("<w:p><w:pPr>{props}</w:pPr>{said}</w:p>"));
    }

    // Each line its own paragraph in Consolas, the way Word itself writes code and reads it back.
    fn code(&mut self, node: &Value, place: &Place) {
        let said: String = content(node)
            .iter()
            .filter_map(|one| one["text"].as_str())
            .collect();
        for line in said.trim_end_matches('\n').split('\n') {
            let run = format!(
                r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
                escaped(&line.replace('\t', "    "))
            );
            self.paragraph("Code", place, &run);
        }
    }

    fn list(&mut self, node: &Value, place: &Place) {
        let ticked = kind(node) == "taskList";
        let id = match kind(node) {
            "orderedList" => {
                self.numbered
                    .push(node["attrs"]["start"].as_u64().unwrap_or(1));
                self.numbered.len() as u32 + 1
            }
            _ => 1,
        };
        let level = place.list.map_or(0, |(_, level)| level + 1);
        for item in content(node) {
            let checked = item["attrs"]["checked"].as_bool().unwrap_or(false);
            for (at, inner) in content(item).iter().enumerate() {
                if at > 0 || kind(inner) != "paragraph" {
                    let under = Place {
                        list: Some((id, level)),
                        ..place.clone()
                    };
                    self.block(inner, &under);
                    continue;
                }
                let mut said = String::new();
                if ticked {
                    let box_ = if checked { "☑ " } else { "☐ " };
                    said.push_str(&format!(
                        r#"<w:r><w:t xml:space="preserve">{box_}</w:t></w:r>"#
                    ));
                }
                said.push_str(&self.runs(content(inner)));
                let here = Place {
                    list: (!ticked).then_some((id, level)),
                    depth: place.depth + u32::from(ticked),
                    ..place.clone()
                };
                self.paragraph("", &here, &said);
            }
        }
    }

    fn table(&mut self, node: &Value) {
        let edge = r#"w:val="single" w:sz="4" w:space="0" w:color="BFBFBF""#;
        self.out.push_str(&format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top {edge}/><w:left {edge}/><w:bottom {edge}/><w:right {edge}/><w:insideH {edge}/><w:insideV {edge}/></w:tblBorders></w:tblPr>"#
        ));
        for row in content(node) {
            self.out.push_str("<w:tr>");
            for cell in content(row) {
                let lean = cell["attrs"]["textAlign"].as_str().map(|one| match one {
                    "left" => "left",
                    "right" => "right",
                    _ => "center",
                });
                let mut said = String::new();
                for one in content(cell) {
                    said.push_str(&self.runs(content(one)));
                }
                let jc = lean
                    .map(|one| format!(r#"<w:jc w:val="{one}"/>"#))
                    .unwrap_or_default();
                self.out.push_str(&format!(
                    "<w:tc><w:p><w:pPr>{jc}</w:pPr>{said}</w:p></w:tc>"
                ));
            }
            self.out.push_str("</w:tr>");
        }
        self.out.push_str("</w:tbl><w:p/>");
    }

    fn runs(&mut self, nodes: &[Value]) -> String {
        let mut out = String::new();
        for node in nodes {
            match kind(node) {
                "text" => out.push_str(&self.text(node)),
                "hardBreak" => out.push_str("<w:r><w:br/></w:r>"),
                "image" => out.push_str(&self.image(node)),
                _ => {}
            }
        }
        out
    }

    fn text(&mut self, node: &Value) -> String {
        let said = node["text"].as_str().unwrap_or_default();
        let marks = node["marks"].as_array().map_or(&[][..], Vec::as_slice);
        let mut props = String::new();
        let mut link = None;
        for mark in marks {
            match kind(mark) {
                "bold" => props.push_str("<w:b/>"),
                "italic" => props.push_str("<w:i/>"),
                "strike" => props.push_str("<w:strike/>"),
                "underline" => props.push_str(r#"<w:u w:val="single"/>"#),
                "code" => props.push_str(
                    r#"<w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/>"#,
                ),
                "highlight" => props.push_str(&format!(
                    r#"<w:highlight w:val="{}"/>"#,
                    lit(mark["attrs"]["color"].as_str().unwrap_or_default())
                )),
                "link" => link = mark["attrs"]["href"].as_str(),
                _ => {}
            }
        }
        let outward = link.filter(|to| {
            let to = to.to_ascii_lowercase();
            to.starts_with("http://") || to.starts_with("https://") || to.starts_with("mailto:")
        });
        if outward.is_some() {
            props.insert_str(0, r#"<w:rStyle w:val="Hyperlink"/>"#);
        }
        let run = format!(
            r#"<w:r><w:rPr>{props}</w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
            escaped(said)
        );
        match outward {
            Some(to) => {
                self.links.push(to.to_string());
                format!(
                    r#"<w:hyperlink r:id="rL{}">{run}</w:hyperlink>"#,
                    self.links.len()
                )
            }
            None => run,
        }
    }

    fn image(&mut self, node: &Value) -> String {
        let src = node["attrs"]["src"].as_str().unwrap_or_default();
        let alt = node["attrs"]["alt"].as_str().unwrap_or_default();
        let said = || {
            format!(
                r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
                escaped(alt)
            )
        };
        let Some(bytes) = (self.picture)(src).filter(|one| one.len() as u64 <= self.left) else {
            return said();
        };
        let Some((wide, high, ext)) = measured(&bytes) else {
            return said();
        };
        self.left -= bytes.len() as u64;
        let n = self.media.len() + 1;
        self.media.push((format!("image{n}.{ext}"), bytes, ext));
        let (mut cx, mut cy) = (wide * EMU_PER_PIXEL, high * EMU_PER_PIXEL);
        if cx > WIDEST {
            cy = cy * WIDEST / cx;
            cx = WIDEST;
        }
        let alt = escaped(alt);
        format!(
            r#"<w:r><w:drawing><wp:inline><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="{n}" name="image{n}" descr="{alt}"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{n}" name="image{n}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rM{n}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#
        )
    }

    fn packed(self) -> std::io::Result<Vec<u8>> {
        let mut out = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut out);
        let deflated = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {NAMESPACES}><w:body>{}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/></w:sectPr></w:body></w:document>"#,
            self.out
        );
        let parts = [
            ("[Content_Types].xml", types(&self.media)),
            ("_rels/.rels", ROOT_RELS.to_string()),
            ("word/document.xml", document),
            ("word/styles.xml", STYLES.to_string()),
            ("word/numbering.xml", numbering(&self.numbered)),
            (
                "word/_rels/document.xml.rels",
                rels(&self.links, &self.media),
            ),
        ];
        for (name, said) in parts {
            zip.start_file(name, deflated)?;
            zip.write_all(said.as_bytes())?;
        }
        for (name, bytes, _) in &self.media {
            zip.start_file(format!("word/media/{name}"), stored)?;
            zip.write_all(bytes)?;
        }
        zip.finish()?;
        Ok(out.into_inner())
    }
}

// Word paints highlights from a list of names, so each pen goes to the nearest of them.
fn lit(pen: &str) -> &'static str {
    match pen {
        "red" => "red",
        "green" | "teal" => "green",
        "blue" | "indigo" => "cyan",
        "purple" | "pink" => "magenta",
        "brown" => "darkYellow",
        "gray" => "lightGray",
        _ => "yellow",
    }
}

// A picture's own header says its size; one Word cannot show is left as its words.
fn measured(bytes: &[u8]) -> Option<(u64, u64, &'static str)> {
    let be = |at: usize, wide: usize| -> Option<u64> {
        let part = bytes.get(at..at + wide)?;
        Some(part.iter().fold(0u64, |n, one| n << 8 | u64::from(*one)))
    };
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some((be(16, 4)?, be(20, 4)?, "png"));
    }
    if bytes.starts_with(b"GIF8") {
        let le = |at: usize| Some(u64::from(*bytes.get(at)?) | u64::from(*bytes.get(at + 1)?) << 8);
        return Some((le(6)?, le(8)?, "gif"));
    }
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut at = 2;
    while at + 9 < bytes.len() {
        if bytes[at] != 0xFF {
            return None;
        }
        let marker = bytes[at + 1];
        if marker == 0xFF {
            at += 1;
            continue;
        }
        let wide = usize::try_from(be(at + 2, 2)?).ok()?;
        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            return Some((be(at + 7, 2)?, be(at + 5, 2)?, "jpeg"));
        }
        at += 2 + wide;
    }
    None
}

fn types(media: &[(String, Vec<u8>, &'static str)]) -> String {
    let mut kinds: Vec<&str> = media.iter().map(|(_, _, ext)| *ext).collect();
    kinds.sort_unstable();
    kinds.dedup();
    let pictures: String = kinds
        .iter()
        .map(|ext| format!(r#"<Default Extension="{ext}" ContentType="image/{ext}"/>"#))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>{pictures}<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/></Types>"#
    )
}

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

fn rels(links: &[String], media: &[(String, Vec<u8>, &'static str)]) -> String {
    let base = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let mut out = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rS1" Type="{base}/styles" Target="styles.xml"/><Relationship Id="rN1" Type="{base}/numbering" Target="numbering.xml"/>"#
    );
    for (at, to) in links.iter().enumerate() {
        out.push_str(&format!(
            r#"<Relationship Id="rL{}" Type="{base}/hyperlink" Target="{}" TargetMode="External"/>"#,
            at + 1,
            escaped(to)
        ));
    }
    for (at, (name, _, _)) in media.iter().enumerate() {
        out.push_str(&format!(
            r#"<Relationship Id="rM{}" Type="{base}/image" Target="media/{name}"/>"#,
            at + 1
        ));
    }
    out.push_str("</Relationships>");
    out
}

fn numbering(ordered: &[u64]) -> String {
    let levels = |numbered: bool| -> String {
        (0..9)
            .map(|level| {
                let (fmt, said) = match numbered {
                    true => ("decimal", format!("%{}.", level + 1)),
                    false => ("bullet", ["•", "◦", "▪"][level % 3].to_string()),
                };
                format!(
                    r#"<w:lvl w:ilvl="{level}"><w:start w:val="1"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{said}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{}" w:hanging="360"/></w:pPr></w:lvl>"#,
                    720 * (level + 1)
                )
            })
            .collect()
    };
    let mut out = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:numbering {NAMESPACES}><w:abstractNum w:abstractNumId="1">{}</w:abstractNum><w:abstractNum w:abstractNumId="2">{}</w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="1"/></w:num>"#,
        levels(false),
        levels(true)
    );
    for (at, start) in ordered.iter().enumerate() {
        out.push_str(&format!(
            r#"<w:num w:numId="{}"><w:abstractNumId w:val="2"/><w:lvlOverride w:ilvl="0"><w:startOverride w:val="{start}"/></w:lvlOverride></w:num>"#,
            at + 2
        ));
    }
    out.push_str("</w:numbering>");
    out
}

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="280" w:after="100"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="30"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:sz w:val="22"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:b/><w:i/><w:sz w:val="22"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/></w:pPr><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/><w:sz w:val="19"/></w:rPr></w:style><w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style></w:styles>"#;
