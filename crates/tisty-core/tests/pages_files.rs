use std::path::{Path, PathBuf};

use tisty_core::event::DocAdd;
use tisty_core::model::DocId;
use tisty_core::{DeviceId, Event, Op, State, attach, docs, order};
use ulid::Ulid;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn device(name: &str) -> DeviceId {
    DeviceId(name.into())
}

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn source(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let at = dir.join(name);
    std::fs::write(&at, bytes).unwrap();
    at
}

fn add_doc(
    state: &mut State,
    data: &Path,
    dev: &DeviceId,
    seq: &mut i64,
    body: &str,
) -> (DocId, String) {
    let made = docs::create(&data.join("docs"), dev, body).unwrap();
    let id = Ulid::generate();
    *seq += 1;
    state.apply(&Event::new(
        dev.clone(),
        at(*seq),
        Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id.clone(),
                order: order::first(),
                folder: None,
                page_of: None,
            },
        },
    ));
    (id, made.id)
}

fn add_page(
    state: &mut State,
    data: &Path,
    dev: &DeviceId,
    seq: &mut i64,
    parent: DocId,
    body: &str,
) -> (DocId, String) {
    let made = docs::create(&data.join("docs"), dev, body).unwrap();
    let id = Ulid::generate();
    let placed = order::last_of(state.pages_of(parent).iter().map(|one| one.order.as_str()));
    *seq += 1;
    state.apply(&Event::new(
        dev.clone(),
        at(*seq),
        Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id.clone(),
                order: placed,
                folder: None,
                page_of: Some(parent),
            },
        },
    ));
    (id, made.id)
}

fn drop_doc(state: &mut State, dev: &DeviceId, seq: &mut i64, id: DocId) {
    *seq += 1;
    state.apply(&Event::new(dev.clone(), at(*seq), Op::DocDelete { id }));
}

fn count_files(dir: &Path) -> usize {
    let mut found = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            found += count_files(&entry.path());
        } else {
            found += 1;
        }
    }
    found
}

#[test]
fn deleting_a_page_and_its_document_leaves_a_shared_attachment_named_by_another_document() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let kept = attach::keep(
        &source(src_dir.path(), "shared.bin", b"shared payload bytes"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();

    let (parent, parent_file) = add_doc(&mut state, data, &dev, &mut seq, "# Parent\n\ntext");
    let (page_id, page_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        parent,
        &format!("# Page\n\n{}", kept.written("shared")),
    );
    let (other, other_file) = add_doc(
        &mut state,
        data,
        &dev,
        &mut seq,
        &format!("# Other\n\n{}", kept.written("shared")),
    );

    drop_doc(&mut state, &dev, &mut seq, parent);

    assert!(state.shed.contains(&parent_file));
    assert!(state.shed.contains(&page_file));
    assert!(!state.shed.contains(&other_file));
    assert!(!state.docs.contains_key(&parent));
    assert!(!state.docs.contains_key(&page_id));
    assert!(state.docs.contains_key(&other));

    for file in [&parent_file, &page_file] {
        docs::remove(&data.join("docs"), file).unwrap();
    }

    assert!(docs::read(&data.join("docs"), &other_file).is_ok());
    assert!(docs::read(&data.join("docs"), &page_file).is_err());

    let stored = attach::resolve(&kept.at, data).unwrap();
    assert!(
        stored.exists(),
        "a document that still names it must not lose the file"
    );
    assert_eq!(std::fs::read(&stored).unwrap(), b"shared payload bytes");

    let referenced = docs::referenced(&data.join("docs"));
    assert!(referenced.contains(&kept.at));
    let loose = attach::loose(data, &referenced);
    assert!(
        !loose.items.iter().any(|one| one.at == kept.at),
        "still named by a surviving document, so it must not read as loose"
    );
}

#[test]
fn deleting_a_page_alone_leaves_its_document_and_a_shared_attachment_untouched() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let kept = attach::keep(
        &source(src_dir.path(), "together.bin", b"named by both"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();

    let (parent, parent_file) = add_doc(
        &mut state,
        data,
        &dev,
        &mut seq,
        &format!("# Parent\n\n{}", kept.written("together")),
    );
    let (page_id, page_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        parent,
        &format!("# Page\n\n{}", kept.written("together")),
    );

    drop_doc(&mut state, &dev, &mut seq, page_id);

    assert!(state.shed.contains(&page_file));
    assert!(!state.shed.contains(&parent_file));
    assert!(state.docs.contains_key(&parent));
    assert!(!state.docs.contains_key(&page_id));

    docs::remove(&data.join("docs"), &page_file).unwrap();
    assert!(docs::read(&data.join("docs"), &parent_file).is_ok());

    let referenced = docs::referenced(&data.join("docs"));
    let loose = attach::loose(data, &referenced);
    assert!(!loose.items.iter().any(|one| one.at == kept.at));
}

#[test]
fn deleting_a_documents_only_page_that_named_a_file_leaves_it_loose_until_something_sweeps_it() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let kept = attach::keep(
        &source(src_dir.path(), "solo.bin", b"only the page names this"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();

    let (parent, parent_file) =
        add_doc(&mut state, data, &dev, &mut seq, "# Parent\n\nnothing here");
    let (_, page_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        parent,
        &format!("# Page\n\n{}", kept.written("solo")),
    );

    drop_doc(&mut state, &dev, &mut seq, parent);
    for file in [&parent_file, &page_file] {
        docs::remove(&data.join("docs"), file).unwrap();
    }

    let referenced = docs::referenced(&data.join("docs"));
    assert!(!referenced.contains(&kept.at));

    let loose = attach::loose(data, &referenced);
    assert!(
        loose.items.iter().any(|one| one.at == kept.at),
        "nobody names it any more, so it has to surface as loose"
    );

    let stored = attach::resolve(&kept.at, data).unwrap();
    assert!(stored.exists(), "loose only reports, it never deletes");
}

#[test]
fn duplicating_a_document_with_a_page_reuses_the_same_attachment_file_without_copying_it() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let kept = attach::keep(
        &source(src_dir.path(), "figura.png", b"pixels pixels pixels"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();

    let (original, original_file) = add_doc(
        &mut state,
        data,
        &dev,
        &mut seq,
        &format!("# Original\n\n{}", kept.written("figura")),
    );
    let (_, page_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        original,
        &format!("# Original page\n\n{}", kept.written("figura")),
    );

    let body = docs::read(&data.join("docs"), &original_file).unwrap();
    let made = docs::create(&data.join("docs"), &dev, &body).unwrap();
    let twin = Ulid::generate();
    seq += 1;
    state.apply(&Event::new(
        dev.clone(),
        at(seq),
        Op::DocAdd {
            id: twin,
            d: DocAdd {
                file: made.id.clone(),
                order: order::first(),
                folder: None,
                page_of: None,
            },
        },
    ));

    for page in state
        .pages_of(original)
        .iter()
        .map(|one| one.file.clone())
        .collect::<Vec<_>>()
    {
        let body = docs::read(&data.join("docs"), &page).unwrap();
        let leaf = docs::create(&data.join("docs"), &dev, &body).unwrap();
        let placed = order::last_of(state.pages_of(twin).iter().map(|one| one.order.as_str()));
        seq += 1;
        state.apply(&Event::new(
            dev.clone(),
            at(seq),
            Op::DocAdd {
                id: Ulid::generate(),
                d: DocAdd {
                    file: leaf.id,
                    order: placed,
                    folder: None,
                    page_of: Some(twin),
                },
            },
        ));
    }

    let twin_pages = state.pages_of(twin);
    assert_eq!(twin_pages.len(), 1);
    let twin_page_file = twin_pages[0].file.clone();
    assert_ne!(
        twin_page_file, page_file,
        "the page's twin is a fresh file, not the same one"
    );

    let twin_body = docs::read(&data.join("docs"), &twin_page_file).unwrap();
    let original_page_body = docs::read(&data.join("docs"), &page_file).unwrap();
    assert_eq!(twin_body, original_page_body);
    assert!(
        twin_body.contains(&kept.at),
        "the same markdown link, not a fresh one"
    );

    let shelf = data.join("attachments").join(&kept.sha256[..2]);
    let on_disk: Vec<_> = std::fs::read_dir(&shelf).unwrap().collect();
    assert_eq!(
        on_disk.len(),
        1,
        "the twin points at the file, it does not copy it"
    );
}

#[test]
fn exporting_ten_pages_carries_the_cover_the_pages_in_order_and_every_attachment() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let x = attach::keep(
        &source(src_dir.path(), "x.png", b"x-bytes-1"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();
    let y = attach::keep(
        &source(src_dir.path(), "y.png", b"y-bytes-22"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();
    let z = attach::keep(
        &source(src_dir.path(), "z.png", b"z-bytes-333"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();

    let (book, book_file) = add_doc(
        &mut state,
        data,
        &dev,
        &mut seq,
        &format!("# Book\n\n{}", x.written("cover image")),
    );

    let title_for = |n: u32| {
        if n == 3 || n == 7 {
            "SharedTitle".to_string()
        } else {
            format!("Page{n}")
        }
    };
    for n in 1..=10u32 {
        let mut body = format!("# {}\n\nmarker-page-{n}\n", title_for(n));
        match n {
            1 => body.push_str(&x.written("inline")),
            3 | 7 => body.push_str(&y.written("inline")),
            5 => body.push_str(&z.written("inline")),
            _ => {}
        }
        add_page(&mut state, data, &dev, &mut seq, book, &body);
    }

    let page_files: Vec<String> = state
        .pages_of(book)
        .iter()
        .map(|one| one.file.clone())
        .collect();
    assert_eq!(page_files.len(), 10);

    let out = tmp();
    let taken = docs::with_pages(data, &book_file, &page_files, out.path()).unwrap();

    let folder = out.path().join("Book");
    assert!(folder.join("Book.md").exists());
    for n in 1..=10u32 {
        let name = format!("{:02} {}.md", n, title_for(n));
        let path = folder.join(&name);
        assert!(path.exists(), "{name} missing");
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.contains(&format!("marker-page-{n}")),
            "{name}: {contents}"
        );
    }

    let body3 = std::fs::read_to_string(folder.join("03 SharedTitle.md")).unwrap();
    let body7 = std::fs::read_to_string(folder.join("07 SharedTitle.md")).unwrap();
    assert_ne!(
        body3, body7,
        "the same title must not make one page overwrite the other"
    );

    assert!(folder.join(&x.at).exists());
    assert!(folder.join(&y.at).exists());
    assert!(folder.join(&z.at).exists());
    assert_eq!(std::fs::read(folder.join(&x.at)).unwrap(), b"x-bytes-1");
    assert_eq!(std::fs::read(folder.join(&y.at)).unwrap(), b"y-bytes-22");
    assert_eq!(std::fs::read(folder.join(&z.at)).unwrap(), b"z-bytes-333");
    assert_eq!(
        count_files(&folder.join("attachments")),
        3,
        "x is shared with the cover and y is shared between two pages, but each is one file"
    );
    // one copy per body that names it (cover+page1 for x, two pages for y, one for z), not one per unique file
    assert_eq!(taken, 5);
}

#[test]
fn two_pages_that_both_fall_back_to_the_generic_name_still_export_as_two_files() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (book, book_file) = add_doc(&mut state, data, &dev, &mut seq, "# Simbolos\n\nportada");
    add_page(&mut state, data, &dev, &mut seq, book, "# ¿?\n\nmarker-a\n");
    add_page(&mut state, data, &dev, &mut seq, book, "# ¡!\n\nmarker-b\n");

    let page_files: Vec<String> = state
        .pages_of(book)
        .iter()
        .map(|one| one.file.clone())
        .collect();
    let out = tmp();
    docs::with_pages(data, &book_file, &page_files, out.path()).unwrap();

    let folder = out.path().join("Simbolos");
    let a = std::fs::read_to_string(folder.join("01 documento.md")).unwrap();
    let b = std::fs::read_to_string(folder.join("02 documento.md")).unwrap();
    assert!(a.contains("marker-a"));
    assert!(b.contains("marker-b"));
}

#[test]
fn exporting_a_document_without_pages_still_works() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (_, file) = add_doc(
        &mut state,
        data,
        &dev,
        &mut seq,
        "# Solo\n\njust text, no pages",
    );

    let out = tmp();
    let taken = docs::exported(data, &file, out.path()).unwrap();

    assert_eq!(taken, 0);
    let folder = out.path().join("Solo");
    assert!(folder.join("Solo.md").exists());
    let entries: Vec<_> = std::fs::read_dir(&folder).unwrap().collect();
    assert_eq!(
        entries.len(),
        1,
        "only the cover, no numbered pages and no attachments folder"
    );
}

#[test]
fn exporting_with_pages_into_a_path_inside_the_store_is_refused_before_anything_is_written() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (book, book_file) = add_doc(&mut state, data, &dev, &mut seq, "# Guardado\n\ntexto");
    let (_, page_file) = add_page(&mut state, data, &dev, &mut seq, book, "# Pagina\n\ntexto");

    assert!(docs::with_pages(data, &book_file, std::slice::from_ref(&page_file), data).is_err());
    assert!(docs::with_pages(data, &book_file, &[page_file], &data.join("docs")).is_err());
}

#[test]
fn exporting_skips_a_page_whose_file_vanished_from_disk_without_aborting_the_rest() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (book, book_file) = add_doc(&mut state, data, &dev, &mut seq, "# Libro\n\nportada");
    let (_, gone_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        book,
        "# Perdida\n\nno deberia aparecer",
    );
    let (_, kept_file) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        book,
        "# Presente\n\nmarker-presente",
    );

    docs::remove(&data.join("docs"), &gone_file).unwrap();

    let out = tmp();
    let taken = docs::with_pages(data, &book_file, &[gone_file, kept_file], out.path()).unwrap();

    assert_eq!(taken, 0);
    let folder = out.path().join("Libro");
    assert!(folder.join("Libro.md").exists());
    assert!(!folder.join("01 Perdida.md").exists());
    assert!(
        folder.join("02 Presente.md").exists(),
        "the surviving page still has to come out, numbered as it was passed in"
    );
}

#[test]
fn an_attachment_over_the_document_ceiling_is_refused_before_anything_is_written() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();

    let big = src_dir.path().join("enorme.bin");
    std::fs::File::create(&big)
        .unwrap()
        .set_len(attach::COPIED_IN_DOC + 1)
        .unwrap();

    let refused = attach::keep(&big, data, attach::COPIED_IN_DOC);

    assert!(matches!(
        refused,
        Err(tisty_core::Error::AttachmentTooBig { bytes, limit })
            if bytes == attach::COPIED_IN_DOC + 1 && limit == attach::COPIED_IN_DOC
    ));
    assert!(
        !data.join("attachments").exists(),
        "refused on the metadata alone, before the store was ever touched"
    );
}

#[test]
#[ignore = "copies ~500 MB to disk; run explicitly with `cargo test -p tisty-core --test pages_files -- --ignored an_attachment_right_at_the_document_ceiling_is_still_accepted`"]
fn an_attachment_right_at_the_document_ceiling_is_still_accepted() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();

    let bytes = vec![9u8; attach::COPIED_IN_DOC as usize];
    let file = source(src_dir.path(), "justo.bin", &bytes);

    let kept = attach::keep(&file, data, attach::COPIED_IN_DOC).unwrap();

    let stored = attach::resolve(&kept.at, data).unwrap();
    assert_eq!(
        std::fs::metadata(&stored).unwrap().len(),
        attach::COPIED_IN_DOC
    );
}

#[test]
#[ignore = "writes ~51 MB to disk; run explicitly with `cargo test -p tisty-core --test pages_files -- --ignored a_pages_attachment_above_the_task_ceiling_still_fits_the_document_ceiling`"]
fn a_pages_attachment_above_the_task_ceiling_still_fits_the_document_ceiling() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();

    let bytes = vec![7u8; attach::COPIED_UP_TO as usize + 1024 * 1024];
    let file = source(src_dir.path(), "grande.bin", &bytes);

    let refused = attach::keep(&file, data, attach::COPIED_UP_TO);
    assert!(matches!(
        refused,
        Err(tisty_core::Error::AttachmentTooBig { .. })
    ));

    let kept = attach::keep(&file, data, attach::COPIED_IN_DOC).unwrap();
    let stored = attach::resolve(&kept.at, data).unwrap();
    assert_eq!(
        std::fs::metadata(&stored).unwrap().len(),
        bytes.len() as u64
    );
}

#[test]
fn a_pages_body_hits_the_same_attachment_count_ceiling_as_any_document() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (parent, _) = add_doc(&mut state, data, &dev, &mut seq, "# Cuaderno\n\nnotas");

    let link = "![x](<attachments/ab/x.png>)\n";
    let full = format!("# Pagina llena\n\n{}", link.repeat(attach::KEPT_IN_A_DOC));
    let (_, full_file) = add_page(&mut state, data, &dev, &mut seq, parent, &full);
    let stored = docs::read(&data.join("docs"), &full_file).unwrap();

    assert_eq!(
        attach::fits(&stored, "otro mas"),
        Err(attach::NoRoom::Crowded(attach::KEPT_IN_A_DOC))
    );

    let almost = format!(
        "# Pagina casi llena\n\n{}",
        link.repeat(attach::KEPT_IN_A_DOC - 1)
    );
    assert!(attach::fits(&almost, "otro mas").is_ok());
}

#[test]
fn a_very_long_accented_spaced_name_is_still_kept_and_named_sanely() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();

    let long_name = format!(
        "{}Informe Técnico Ñandú (final) - versión 2.pdf",
        "parte-larga-".repeat(4)
    );
    let file = source(src_dir.path(), &long_name, b"contenido del informe");

    let kept = attach::keep(&file, data, attach::COPIED_IN_DOC).unwrap();

    let stored = attach::resolve(&kept.at, data).unwrap();
    assert!(stored.exists());
    assert_eq!(std::fs::read(&stored).unwrap(), b"contenido del informe");
    let leaf = kept.at.rsplit('/').next().unwrap();
    assert!(leaf.len() < 80, "{leaf}");
    assert!(leaf.ends_with(".pdf"), "{leaf}");
}

#[test]
fn two_sibling_pages_can_attach_different_files_that_share_a_name() {
    let data_dir = tmp();
    let data = data_dir.path();
    let src_dir = tmp();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (parent, _) = add_doc(&mut state, data, &dev, &mut seq, "# Album\n\nfotos");

    let one = attach::keep(
        &source(src_dir.path(), "foto.jpg", b"bytes de la primera foto"),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();
    let two = attach::keep(
        &source(
            src_dir.path(),
            "foto.jpg",
            b"bytes de la segunda foto, distinta",
        ),
        data,
        attach::COPIED_IN_DOC,
    )
    .unwrap();
    assert_ne!(
        one.at, two.at,
        "same name, different bytes, different files"
    );

    let (_, page_a) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        parent,
        &format!("# Pagina A\n\n{}", one.written("foto.jpg")),
    );
    let (_, page_b) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        parent,
        &format!("# Pagina B\n\n{}", two.written("foto.jpg")),
    );

    assert!(
        docs::read(&data.join("docs"), &page_a)
            .unwrap()
            .contains(&one.at)
    );
    assert!(
        docs::read(&data.join("docs"), &page_b)
            .unwrap()
            .contains(&two.at)
    );
    assert_eq!(
        std::fs::read(attach::resolve(&one.at, data).unwrap()).unwrap(),
        b"bytes de la primera foto"
    );
    assert_eq!(
        std::fs::read(attach::resolve(&two.at, data).unwrap()).unwrap(),
        b"bytes de la segunda foto, distinta"
    );
}

#[test]
fn the_way_into_a_page_is_the_file_beside_it_once_the_book_is_out_of_tisty() {
    let data_dir = tmp();
    let data = data_dir.path();
    let dev = device("mac0");
    let mut state = State::default();
    let mut seq = 0i64;

    let (book, book_file) = add_doc(&mut state, data, &dev, &mut seq, "# Libro\n\nportada");
    let (_, one) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        book,
        "# Uno\n\nmarker-a\n",
    );
    let (_, two) = add_page(
        &mut state,
        data,
        &dev,
        &mut seq,
        book,
        "# Dos\n\nmarker-b\n",
    );

    let cover = format!(
        "# Libro\n\nportada\n\n{}\n\ny luego\n\n{}\n",
        tisty_core::refs::card(&one, "Uno"),
        tisty_core::refs::card(&two, "Dos")
    );
    docs::write(&data.join("docs"), &book_file, &cover).unwrap();

    let page_files: Vec<String> = state
        .pages_of(book)
        .iter()
        .map(|one| one.file.clone())
        .collect();
    let out = tmp();
    docs::with_pages(data, &book_file, &page_files, out.path()).unwrap();

    let said = std::fs::read_to_string(out.path().join("Libro").join("Libro.md")).unwrap();
    assert!(said.contains("![Uno](<01 Uno.md>)"), "{said}");
    assert!(said.contains("![Dos](<02 Dos.md>)"), "{said}");
    assert!(
        !said.contains("tisty:doc/"),
        "nothing may still point at a name only Tisty knows: {said}"
    );
}
