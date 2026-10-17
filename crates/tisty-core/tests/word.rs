use std::io::Write;

const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing""#;

fn docx(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut out);
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in parts {
        zip.start_file(*name, stored).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    out.into_inner()
}

#[test]
fn a_word_document_comes_in_as_the_tisty_language() {
    let body = format!(
        r#"<w:document {W}><w:body>
<w:p><w:pPr><w:pStyle w:val="Ttulo1"/></w:pPr><w:r><w:t>Plan</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Dato</w:t></w:r><w:r><w:t xml:space="preserve"> y </w:t></w:r><w:r><w:rPr><w:i/><w:color w:val="DC2626"/></w:rPr><w:t>rojo</w:t></w:r><w:r><w:t xml:space="preserve"> y </w:t></w:r><w:r><w:rPr><w:highlight w:val="yellow"/></w:rPr><w:t>marcado</w:t></w:r><w:r><w:t xml:space="preserve"> y </w:t></w:r><w:hyperlink r:id="rId9"><w:r><w:t>enlace</w:t></w:r></w:hyperlink></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>uno</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>adentro</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="2"/></w:numPr></w:pPr><w:r><w:t>primero</w:t></w:r></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
<w:p><w:r><w:drawing><wp:inline><wp:docPr id="1" name="Imagen 1" descr="logo"/><a:graphic><a:graphicData><a:blip r:embed="rId7"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>
</w:body></w:document>"#
    );
    let styles = format!(
        r#"<w:styles {W}><w:style w:styleId="Ttulo1"><w:name w:val="heading 1"/></w:style></w:styles>"#
    );
    let numbering = format!(
        r#"<w:numbering {W}>
<w:abstractNum w:abstractNumId="10"><w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl><w:lvl w:ilvl="1"><w:numFmt w:val="bullet"/></w:lvl></w:abstractNum>
<w:abstractNum w:abstractNumId="20"><w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl></w:abstractNum>
<w:num w:numId="1"><w:abstractNumId w:val="10"/></w:num><w:num w:numId="2"><w:abstractNumId w:val="20"/></w:num>
</w:numbering>"#
    );
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Target="https://x.y"/><Relationship Id="rId7" Target="media/image1.png"/></Relationships>"#;
    let bytes = docx(&[
        ("word/document.xml", body.as_bytes()),
        ("word/styles.xml", styles.as_bytes()),
        ("word/numbering.xml", numbering.as_bytes()),
        ("word/_rels/document.xml.rels", rels.as_bytes()),
        ("word/media/image1.png", b"\x89PNG\r\n\x1a\n"),
    ]);

    let word = tisty_core::arriving::from_word(&bytes).unwrap();
    let said = tisty_core::arriving::tidied(&tisty_core::arriving::from_html(&word.html)).body;

    for piece in [
        "# Plan",
        "**Dato**",
        "*==rojo=={red}*",
        "==marcado==",
        "[enlace](https://x.y)",
        "- uno",
        "  - adentro",
        "1. primero",
        "| a | b |",
        "![logo](media/image1.png)",
    ] {
        assert!(said.contains(piece), "{piece:?} missing from {said}");
    }
    assert_eq!(word.pictures.len(), 1);
    assert_eq!(word.pictures[0].0, "media/image1.png");
}

#[test]
fn what_is_not_a_word_document_is_told_apart() {
    assert!(tisty_core::arriving::from_word(b"not a zip").is_none());
    assert!(tisty_core::arriving::from_word(&docx(&[("other.xml", b"<x/>")])).is_none());
}
