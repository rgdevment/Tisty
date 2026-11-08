//! Each rule the contract gives Tisty as a writer, on a document kept in a real store. Every envelope sealed here is opened by the contract's sample reader and by Tisty's own, and both must say the same.

#[allow(dead_code)]
mod common;
#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::{Home, a_picture, asked};
use serde_json::Value;
use tisty_contracts::{EnvelopeKind, Opening, Served, check, check_html};
use tisty_core::Op;
use tisty_envelope::reading::{files_for, reading};
use tisty_sharing::{Finished, Remembered, Sealed, Told, Unshared, remembered, signer};

struct Read {
    id: String,
    held: Value,
    pictures: Vec<Vec<u8>>,
    files: Vec<Vec<u8>>,
}

fn home() -> (tempfile::TempDir, Home) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    (tmp, home)
}

fn nothing_served() -> Served {
    Served {
        received: None,
        revocations: None,
    }
}

fn key_of(sealed: &Sealed, id: &str) -> Option<String> {
    sealed
        .remembering
        .iter()
        .find(|(_, one)| one.id == id)
        .and_then(|(_, one)| one.link_key.clone())
}

fn asked_of(id: &str, kind: EnvelopeKind, key: Option<&String>) -> Opening {
    Opening {
        id: id.to_owned(),
        kind,
        link_key: key.cloned(),
    }
}

// What goes out has to open for anyone the contract's words reach, not only for Tisty.
fn every_reader_opens(sealed: &Sealed) -> Vec<Read> {
    sealed
        .going
        .iter()
        .map(|one| {
            let key = key_of(sealed, &one.id);
            let bytes = key
                .as_ref()
                .map(|said| URL_SAFE_NO_PAD.decode(said).unwrap());
            let main = (one.id.as_str(), "main");
            let (sample, held) =
                opening::reading(&one.sealed.main, main, bytes.as_deref()).unwrap();
            let ours = reading(
                &one.sealed.main,
                &asked_of(&one.id, EnvelopeKind::Main, key.as_ref()),
                &nothing_served(),
            )
            .unwrap();
            assert_eq!(ours.document, held);
            assert_eq!(check("document", &held), Vec::<String>::new());
            assert_eq!(
                check_html(held["html"].as_str().unwrap()),
                Vec::<String>::new()
            );
            for (bytes, said) in ours
                .pictures
                .iter()
                .zip(held["pictures"].as_array().into_iter().flatten())
            {
                assert!(opening::picture_holds(bytes, said), "{said}");
                assert!(
                    tisty_envelope::reading::picture_holds(bytes, said),
                    "{said}"
                );
            }
            let files = one.sealed.files.as_ref().map_or_else(Vec::new, |files| {
                let asked = (one.id.as_str(), "files");
                let theirs = opening::files_for(&held, files, asked, bytes.as_deref()).unwrap();
                let ours = files_for(
                    &held,
                    files,
                    &asked_of(&one.id, EnvelopeKind::Files, key.as_ref()),
                    &nothing_served(),
                )
                .unwrap();
                assert_eq!(theirs.parts, ours.parts);
                ours.parts
            });
            assert_eq!(sample.parts[2..], ours.pictures[..]);
            Read {
                id: one.id.clone(),
                held,
                pictures: ours.pictures,
                files,
            }
        })
        .collect()
}

fn by_a_window_with_nothing_to_draw(gathered: &tisty_sharing::Gathered) -> Vec<Finished> {
    gathered
        .unfinished()
        .into_iter()
        .map(|one| Finished {
            name: one.name,
            html: one.html,
            not_drawn: 0,
        })
        .collect()
}

fn sealed(home: &Home, asked: &tisty_sharing::Asked) -> Sealed {
    let gathered = home.gathered(asked);
    let answers = by_a_window_with_nothing_to_draw(&gathered);
    home.sealed(gathered, answers)
}

fn sealed_within(home: &Home, asked: &tisty_sharing::Asked, top: usize) -> Sealed {
    let gathered = home.gathered_within(asked, top);
    let answers = by_a_window_with_nothing_to_draw(&gathered);
    home.sealed(gathered, answers)
}

fn all_text(read: &Read) -> String {
    let mut out = String::new();
    out.push_str(&read.held.to_string());
    for page in read.held["pages"].as_array().into_iter().flatten() {
        out.push_str(&page.to_string());
    }
    out
}

#[test]
fn what_names_it_says_nothing_of_the_machine_it_came_from() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n", None);
    home.wrote("# Enero\n", Some(doc));
    let file = home.state.docs[&doc].file.clone();

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    assert!(!read.id.contains(&file) && !read.id.contains(&doc.to_string()));
    assert!(!read.id.contains(&home.device.0));
    assert_eq!(read.held["pages"][0]["id"], "p1");
    assert_eq!(read.held["pages"][0]["page_of"], read.id.as_str());
    let text = all_text(read);
    assert!(
        !text.contains(&file) && !text.contains(&doc.to_string()),
        "{text}"
    );
}

#[test]
fn nobody_who_did_not_choose_it_is_named_and_no_date_goes_along() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n", None);
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    for gone in [
        "by",
        "born_by",
        "edited_by",
        "owner",
        "properties",
        "made",
        "wrote",
    ] {
        assert!(read.held.get(gone).is_none(), "{gone}");
    }
}

#[test]
fn what_stays_home_stays_home_in_any_capitals() {
    let (_tmp, mut home) = home();
    let body = "---\ntitle: Paseos\nTask: 01M6\nTranslationKey: paseos\nLIST: Trabajo privado\nscheduled: 2026-11-03\ndue: 2026-11-04\nremind: [2026-11-03T09:00]\npriority: do\nrepeat: every week from due\n---\n\n# Paseos\n";
    let doc = home.wrote(body, None);
    let text = all_text(&every_reader_opens(&sealed(&home, &asked(doc)))[0]).to_lowercase();
    for gone in [
        "01m6",
        "translationkey",
        "trabajo privado",
        "scheduled",
        "2026-11-04",
        "remind",
        "priority",
        "every week",
    ] {
        assert!(!text.contains(gone), "{gone} went along in {text}");
    }
}

#[test]
fn a_title_with_breaks_or_what_is_unseen_is_cleaned_wherever_it_goes() {
    let (_tmp, mut home) = home();
    let es = home.wrote(
        "---\ntitle: \"Paseos \u{202E}exe.fdp\"\nlang: es\n---\n\n[[Otro]]\n",
        None,
    );
    let en = home.wrote(
        "---\ntitle: \"Walks\u{200B} here\"\nlang: en\n---\n\nx\n",
        None,
    );
    home.joined(en, es);
    let other = home.wrote("---\ntitle: \"Otro\u{2060}\"\n---\n\nx\n", None);
    let mut asking = asked(es);
    asking.cited = vec![other];

    let reads = every_reader_opens(&sealed(&home, &asking));
    let anchor = &reads[0].held;
    assert_eq!(anchor["title"], "Paseos exe.fdp");
    assert_eq!(anchor["languages"][0]["title"], "Walks here");
}

#[test]
fn a_person_can_say_a_title_nothing_is_left_of_and_it_still_seals() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("---\ntitle: \"\u{200B}\"\n---\n\nHola\n", None);
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    assert!(read.held["body"].as_str().unwrap().contains("Hola"));
}

#[test]
fn its_properties_make_its_sheet_tags_and_language() {
    let (_tmp, mut home) = home();
    let doc = home.wrote(
        "---\ntitle: Paseos\ntags: [Rina, camión]\npaper: letter\nsheets: on\norientation: landscape\nlang: pt_br\n---\n\n# Paseos\n",
        None,
    );
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    assert_eq!(read.held["lang"], "pt-BR");
    assert_eq!(read.held["sheet"]["paper"], "letter");
    assert_eq!(read.held["sheet"]["on"], true);
    assert_eq!(read.held["sheet"]["orientation"], "landscape");
    assert_eq!(read.held["tags"], serde_json::json!(["rina", "camion"]));
}

#[test]
fn a_picture_is_made_anew_named_by_its_place_and_travels_once() {
    let (_tmp, mut home) = home();
    let picture = a_picture(4000, 3000, 1);
    let leaf = home.attached("Vacaciones en la casa de Rina.png", &picture);
    let doc = home.wrote(&format!("# Paseos\n\n![Rina]({leaf})\n"), None);
    home.wrote(&format!("# Enero\n\n![Otra vez]({leaf})\n"), Some(doc));

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    assert_eq!(read.pictures.len(), 1, "the same picture travels once");
    let said = &read.held["pictures"][0];
    assert_eq!(said["name"], "picture-0.png");
    assert_eq!(said["at"], 0);
    assert_eq!(said["width"], 1600);
    assert_eq!(said["height"], 1200);
    assert!(!read.pictures[0].windows(6).any(|one| one == b"HIDDEN"));
    let html = read.held["html"].as_str().unwrap();
    assert!(html.contains("src=\"tisty:picture/0\""), "{html}");
    assert!(html.contains("width=\"1600\" height=\"1200\""), "{html}");
    assert!(
        read.held["pages"][0]["html"]
            .as_str()
            .unwrap()
            .contains("tisty:picture/0")
    );
    assert!(!all_text(read).to_lowercase().contains("vacaciones"));
    assert_eq!(out.told, []);
}

#[test]
fn two_files_with_the_same_pixels_are_one_picture() {
    let (_tmp, mut home) = home();
    let one = a_picture(8, 8, 1);
    let mut other = one.clone();
    other.extend_from_slice(b"and something else hidden");
    let (one, other) = (home.attached("a.png", &one), home.attached("b.png", &other));
    assert_ne!(one, other);
    let doc = home.wrote(
        &format!(
            "![a]({one}) ![b]({other})
"
        ),
        None,
    );
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    assert_eq!(read.pictures.len(), 1);
    assert_eq!(read.held["pictures"].as_array().unwrap().len(), 1);
    let html = read.held["html"].as_str().unwrap();
    assert_eq!(html.matches("tisty:picture/0").count(), 2, "{html}");
}

#[test]
fn a_heic_is_asked_of_the_machine_and_goes_on_from_those_pixels() {
    let (_tmp, mut home) = home();
    let mut heic = vec![0, 0, 0, 24];
    heic.extend_from_slice(b"ftypheic\0\0\0\0mif1heic");
    heic.extend_from_slice(b"pixels");
    let leaf = home.attached("foto.heic", &heic);
    let doc = home.wrote(&format!("![Rina]({leaf})\n"), None);
    let turned = a_picture(8, 8, 3);
    let gathered = home.gathered_with(&asked(doc), &|_| Some(turned.clone()));
    let answers = by_a_window_with_nothing_to_draw(&gathered);
    let out = home.sealed(gathered, answers);
    let read = &every_reader_opens(&out)[0];
    assert_eq!(read.held["pictures"][0]["kind"], "image/jpeg");
    assert_eq!(read.held["pictures"][0]["name"], "picture-0.jpg");
    assert_eq!(out.told, []);
}

#[test]
fn a_picture_that_cannot_be_made_travels_as_a_file_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let svg = home.attached("dibujo.svg", b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
    let doc = home.wrote(&format!("![Dibujo]({svg})\n"), None);

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    assert_eq!(read.pictures.len(), 0);
    assert_eq!(read.files.len(), 1);
    assert_eq!(read.held["files"][0]["name"], "dibujo.svg");
    assert!(read.held["html"].as_str().unwrap().contains("tisty:file/0"));
    assert!(matches!(&out.told[..], [Told::PictureAsFile { name, .. }] if name == "dibujo.svg"));
}

#[test]
fn what_never_was_a_picture_goes_as_an_attachment_and_nothing_is_told() {
    let (_tmp, mut home) = home();
    let cartilla = home.attached("cartilla.pdf", b"%PDF-1.7 a vet card");
    let bark = home.attached("bark.mp3", b"ID3\x04\x00\x00\x00\x00\x00\x00 a bark");
    let doc = home.wrote(
        &format!("![Cartilla]({cartilla})\n\n![Ladrido]({bark})\n"),
        None,
    );

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    assert_eq!(out.told, []);
    assert_eq!(read.pictures.len(), 0);
    assert_eq!(read.files.len(), 2);
}

#[test]
fn only_a_picture_that_cannot_be_drawn_is_told_and_a_page_with_an_icon_is_not_one() {
    let (_tmp, mut home) = home();
    let walk = home.attached(
        "walk.mp4",
        b"\x00\x00\x00\x18ftypisom\x00\x00\x02\x00isomiso2mp41 a walk",
    );
    let page = home.attached(
        "walks.html",
        b"<!doctype html><html><body><svg viewBox=\"0 0 1 1\"></svg></body></html>",
    );
    let drawn = home.attached(
        "mark.svg",
        b"<?xml version=\"1.0\"?>\n<!-- a mark -->\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );
    let exported = home.attached(
        "logo.svg",
        b"<?xml version=\"1.0\"?>\n<!DOCTYPE svg [\n<!ENTITY ns_svg \"http://www.w3.org/2000/svg\">\n]>\n<svg xmlns=\"&ns_svg;\"/>",
    );
    let broken = home.attached(
        "rina.jpg",
        b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00 not a photo after all",
    );
    let results = home.attached("results.csv", b"P1 team,points\nRina,3\n");
    let doc = home.wrote(
        &format!(
            "![Paseo]({walk})\n\n![Paseos]({page})\n\n![Marca]({drawn})\n\n![Logo]({exported})\n\n![Rina]({broken})\n\n![Tabla]({results})\n"
        ),
        None,
    );

    let out = sealed(&home, &asked(doc));
    let told: Vec<&str> = out
        .told
        .iter()
        .filter_map(|one| match one {
            Told::PictureAsFile { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(told, ["mark.svg", "logo.svg", "rina.jpg"], "{:?}", out.told);
    assert_eq!(out.told.len(), 3, "{:?}", out.told);
}

#[test]
fn a_heic_this_machine_cannot_read_travels_as_a_file_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let mut heic = vec![0, 0, 0, 24];
    heic.extend_from_slice(b"ftypheic\0\0\0\0mif1heic");
    let leaf = home.attached("foto.heic", &heic);
    let doc = home.wrote(&format!("![Rina]({leaf})\n"), None);
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert_eq!(read.files[0], heic);
    assert!(matches!(&out.told[..], [Told::PictureAsFile { .. }]));
}

#[test]
fn a_picture_that_cannot_go_either_way_travels_as_its_alt_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("rina.png", &a_picture(2, 2, 1));
    let doc = home.wrote(&format!("![Rina en el parque]({leaf})\n"), None);
    std::fs::remove_file(home.paths.attachments().join(&leaf)).unwrap();

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    let html = read.held["html"].as_str().unwrap();
    assert!(
        html.contains("Rina en el parque") && !html.contains("<img"),
        "{html}"
    );
    assert!(!all_text(read).contains(&leaf));
    assert!(matches!(&out.told[..], [Told::PictureAsWords { .. }]));
}

#[test]
fn a_picture_on_the_web_stays_a_link_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("![Rina](https://a.example/x.png)\n", None);
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    let html = read.held["html"].as_str().unwrap();
    assert!(
        html.contains("href=\"https://a.example/x.png\"") && !html.contains("<img"),
        "{html}"
    );
    assert!(matches!(&out.told[..], [Told::PictureAsLink { .. }]));
}

#[test]
fn a_file_goes_apart_named_by_itself_and_never_by_its_path() {
    let (_tmp, mut home) = home();
    let pdf = b"%PDF-1.7 cartilla".to_vec();
    let leaf = home.attached("cartilla.pdf", &pdf);
    let doc = home.wrote(
        &format!("[La cartilla]({leaf})\n\n![otra]({leaf} \"file\")\n"),
        None,
    );

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];

    assert_eq!(read.files, vec![pdf.clone()]);
    assert_eq!(read.held["files"][0]["name"], "cartilla.pdf");
    assert_eq!(read.held["files"][0]["size"], pdf.len());
    let html = read.held["html"].as_str().unwrap();
    assert!(
        html.contains("<a class=\"card\" href=\"tisty:file/0\">"),
        "{html}"
    );
    assert!(!all_text(read).contains(&leaf));
    assert_eq!(out.told, []);
}

#[test]
fn a_file_resting_in_the_other_folder_is_found_too() {
    let (tmp, mut home) = home();
    let pdf = b"%PDF-1.7 cartilla";
    let leaf = home.attached("cartilla.pdf", pdf);
    let doc = home.wrote(
        &format!(
            "[c]({leaf})
"
        ),
        None,
    );
    let beside = tmp.path().join("beside");
    std::fs::create_dir_all(beside.join("attachments")).unwrap();
    std::fs::rename(
        home.paths.attachments().join(&leaf),
        beside.join("attachments").join(&leaf),
    )
    .unwrap();
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: Some(&beside),
        turned: &|_| None,
    };
    let gathered = tisty_sharing::gathered(&desk, &asked(doc)).unwrap();
    let answers = by_a_window_with_nothing_to_draw(&gathered);
    let out = home.sealed(gathered, answers);
    assert_eq!(every_reader_opens(&out)[0].files, vec![pdf.to_vec()]);
    assert_eq!(out.told, []);
}

#[test]
fn a_file_that_is_gone_goes_as_its_words_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("cartilla.pdf", b"%PDF");
    let doc = home.wrote(&format!("[La cartilla]({leaf})\n"), None);
    std::fs::remove_file(home.paths.attachments().join(&leaf)).unwrap();
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert!(read.held["html"].as_str().unwrap().contains("La cartilla"));
    assert!(!all_text(read).contains(&leaf));
    assert!(matches!(&out.told[..], [Told::FileAsWords { .. }]));
}

#[test]
fn what_leads_nowhere_goes_as_its_words() {
    let (_tmp, mut home) = home();
    let doc = home.wrote(
        "[nota](C:/Users/mario/notas/secreto.md) y [[Sin llegar]] y [otro](tisty:doc/abc)\n",
        None,
    );
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    let text = all_text(read);
    assert!(
        !text.contains("secreto") && !text.contains("tisty:doc"),
        "{text}"
    );
    assert!(text.contains("Sin llegar"));
}

#[test]
fn pages_go_inside_by_their_place_and_are_reached_by_it() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\n[[Febrero]] y [[Enero]]\n", None);
    let enero = home.wrote("# Enero\n", Some(doc));
    let febrero = home.wrote("# Febrero\n", Some(doc));
    let febrero_file = home.state.docs[&febrero].file.clone();
    let enero_file = home.state.docs[&enero].file.clone();

    let reads = every_reader_opens(&sealed(&home, &asked(doc)));
    assert_eq!(reads.len(), 1, "a page travels in its document's envelope");
    let pages = reads[0].held["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["at"], 1);
    assert_eq!(pages[1]["at"], 2);
    let at_of = |title: &str| {
        pages.iter().find(|one| one["title"] == title).unwrap()["at"]
            .as_u64()
            .unwrap()
    };
    let html = reads[0].held["html"].as_str().unwrap();
    assert!(
        html.contains(&format!("tisty:page/{}", at_of("Enero"))),
        "{html}"
    );
    assert!(
        html.contains(&format!("tisty:page/{}", at_of("Febrero"))),
        "{html}"
    );
    let text = all_text(&reads[0]);
    assert!(!text.contains(&febrero_file) && !text.contains(&enero_file));
}

#[test]
fn only_the_anchor_holds_the_map_of_its_languages_and_all_wear_its_paper() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\npaper: letter\n---\n\n# Paseos\n", None);
    let en = home.wrote("---\nlang: en\n---\n\n# Walks\n", None);
    home.joined(en, es);

    let out = sealed(&home, &asked(en));
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 2, "each language in its own envelope");
    let anchor = reads.iter().find(|one| one.held["lang"] == "es").unwrap();
    let other = reads.iter().find(|one| one.held["lang"] == "en").unwrap();
    let map = &anchor.held["languages"][0];
    assert_eq!(map["lang"], "en");
    assert_eq!(map["document"], other.id.as_str());
    assert_eq!(map["title"], "Walks");
    assert_eq!(map["key"].as_str(), key_of(&out, &other.id).as_deref());
    assert!(other.held.get("languages").is_none());
    assert_eq!(other.held["sheet"]["paper"], "letter");
    assert_ne!(key_of(&out, &anchor.id), key_of(&out, &other.id));
}

#[test]
fn each_language_names_itself_exactly_as_the_map_names_it() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es-CL\n---\n\n# Paseos\n", None);
    let en = home.wrote("---\nlang: en-GB\n---\n\n# Walks\n", None);
    let pt = home.wrote("---\nlang: pt-BR\n---\n\n# Passeios\n", None);
    home.joined(en, es);
    home.joined(pt, es);

    let out = sealed(&home, &asked(es));
    let reads = every_reader_opens(&out);
    let anchor = reads
        .iter()
        .find(|one| one.held["lang"] == "es-CL")
        .unwrap();
    let map = anchor.held["languages"].as_array().unwrap();

    assert_eq!(map.len(), 2);
    for entry in map {
        let named = reads
            .iter()
            .find(|one| one.id == entry["document"].as_str().unwrap())
            .unwrap();
        assert_eq!(named.held["lang"], entry["lang"]);
    }
}

#[test]
fn what_is_cited_goes_only_when_chosen_in_its_own_envelopes_with_its_key() {
    let (_tmp, mut home) = home();
    let other = home.wrote("# Otro\n\nAlgo aparte, de [[Paseos]].\n", None);
    let file = home.state.docs[&other].file.clone();
    let doc = home.wrote(
        &format!("# Paseos\n\n[[Otro]] y ![Otro](tisty:doc/{file})\n"),
        None,
    );

    let alone = every_reader_opens(&sealed(&home, &asked(doc)));
    assert_eq!(alone.len(), 1, "by default it is only words");
    let text = all_text(&alone[0]);
    assert!(text.contains("Otro") && !text.contains(&file) && alone[0].held.get("cited").is_none());

    let mut asking = asked(doc);
    asking.cited = vec![other];
    let out = sealed(&home, &asking);
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 2);
    let cites = &reads[0].held;
    let cited = &cites["cited"];
    assert_eq!(
        cited.as_array().unwrap().len(),
        1,
        "once however many times it is named"
    );
    assert_eq!(cited[0]["document"], reads[1].id.as_str());
    assert_eq!(cited[0]["title"], "Otro");
    assert_eq!(
        cited[0]["key"].as_str(),
        key_of(&out, &reads[1].id).as_deref()
    );
    let html = cites["html"].as_str().unwrap();
    assert!(html.contains("href=\"tisty:cited/0\""), "{html}");
    assert!(
        reads[1].held.get("cited").is_none(),
        "what is cited cites nothing further"
    );
    assert!(!all_text(&reads[1]).contains("tisty:cited"));
}

#[test]
fn what_is_cited_in_two_languages_goes_once_and_its_own_languages_go_with_it() {
    let (_tmp, mut home) = home();
    let otro = home.wrote(
        "---
lang: es
---

# Otro
",
        None,
    );
    let other = home.wrote(
        "---
lang: en
---

# Other
",
        None,
    );
    home.joined(other, otro);
    let es = home.wrote(
        "---
lang: es
---

[[Otro]]
",
        None,
    );
    let en = home.wrote(
        "---
lang: en
---

[[Other]]
",
        None,
    );
    home.joined(en, es);
    let mut asking = asked(es);
    asking.cited = vec![otro];

    let out = sealed(&home, &asking);
    let reads = every_reader_opens(&out);
    assert_eq!(
        reads.len(),
        4,
        "two languages, and the two of what they cite"
    );
    let ids: std::collections::BTreeSet<&String> = reads.iter().map(|one| &one.id).collect();
    assert_eq!(ids.len(), 4);
    for one in reads.iter().filter(|one| one.held.get("cited").is_some()) {
        assert_eq!(one.held["cited"][0]["title"], "Otro");
    }
    assert_eq!(
        reads
            .iter()
            .filter(|one| one.held.get("cited").is_some())
            .count(),
        2
    );
}

#[test]
fn what_is_public_goes_unsealed_with_no_key_anywhere() {
    let (_tmp, mut home) = home();
    let other = home.wrote("# Otro\n", None);
    let es = home.wrote("---\nlang: es\n---\n\n[[Otro]]\n", None);
    let en = home.wrote("---\nlang: en\n---\n\nx\n", None);
    home.joined(en, es);
    let mut asking = asked(es);
    asking.public = true;
    asking.cited = vec![other];

    let out = sealed(&home, &asking);
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 3);
    for one in &out.remembering {
        assert_eq!(one.1.link_key, None);
    }
    let anchor = &reads[0].held;
    assert!(anchor["languages"][0].get("key").is_none());
    assert!(anchor["cited"][0].get("key").is_none());
}

#[test]
fn the_name_it_is_signed_with_is_chosen_and_never_passes_for_a_fingerprint() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("x\n", None);
    let mut asking = asked(doc);
    asking.signed_as = Some("  Rina\u{200B} ".into());
    let read = &every_reader_opens(&sealed(&home, &asking))[0];
    assert_eq!(read.held["signed_as"], "Rina");

    asking.signed_as = Some("a 123 456".into());
    let out = sealed(&home, &asking);
    let read = &every_reader_opens(&out)[0];
    assert!(read.held.get("signed_as").is_none());
    assert_eq!(out.told, [Told::NameLeftOut]);
}

#[test]
fn diagrams_a_window_could_not_draw_are_told() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\n```mermaid\nnot a diagram\n```\n", None);
    let gathered = home.gathered(&asked(doc));
    let answers = gathered
        .unfinished()
        .into_iter()
        .map(|one| Finished {
            name: one.name,
            html: "<figure class=\"code\"><pre><code>not a diagram</code></pre></figure>".into(),
            not_drawn: 1,
        })
        .collect();
    let out = home.sealed(gathered, answers);
    every_reader_opens(&out);
    assert!(
        matches!(&out.told[..], [Told::NotDrawn { document, many: 1 }] if document == "Paseos")
    );
}

#[test]
fn a_window_that_leaves_something_unfinished_stops_it() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\n```rust\nfn a() {}\n```\n", None);
    home.wrote("# Enero\n\n```rust\nfn b() {}\n```\n", Some(doc));
    let mut gathered = home.gathered(&asked(doc));
    let mut answers = by_a_window_with_nothing_to_draw(&gathered);
    assert_eq!(answers.len(), 2);
    answers.pop();
    assert!(matches!(
        gathered.finished(answers),
        Err(Unshared::Unfinished(_))
    ));
}

#[test]
fn a_diagram_never_leaves_undrawn_even_if_a_window_says_it_did() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("```mermaid\nflowchart TD\n```\n", None);
    let mut gathered = home.gathered(&asked(doc));
    let answers = by_a_window_with_nothing_to_draw(&gathered);
    gathered.finished(answers).unwrap();
    let (key, certificate) = home.signer();
    let mut writer = tisty_envelope::sealing::Writer::new(&key, &certificate);
    assert!(gathered.sealed(&mut writer).is_err());
}

fn only_one(remembering: &[(String, Remembered)]) -> &Remembered {
    assert_eq!(remembering.len(), 1);
    &remembering[0].1
}

#[test]
fn a_comma_does_not_send_its_files_again_and_what_was_sent_is_remembered_with_its_key() {
    let (_tmp, mut home) = home();
    let pdf = b"%PDF-1.7 cartilla".to_vec();
    let leaf = home.attached("cartilla.pdf", &pdf);
    let doc = home.wrote(&format!("# Paseos\n\n[La cartilla]({leaf})\n"), None);

    let first = sealed(&home, &asked(doc));
    let was = only_one(&first.remembering).clone();
    assert_eq!(was.version, 1);
    assert!(was.link_key.is_some());
    assert_eq!(was.files_key, was.link_key);
    assert!(was.files_hash.is_some());
    assert!(
        remembered(&home.state).is_empty(),
        "nothing is remembered until it went"
    );
    home.sent(&first.remembering);
    let file = &first.remembering[0].0;
    assert_eq!(remembered(&home.state)[file], was);

    let file_name = home.state.docs[&doc].file.clone();
    tisty_core::docs::write(
        &home.paths.docs(),
        &file_name,
        &format!("# Paseos,\n\n[La cartilla]({leaf})\n"),
    )
    .unwrap();
    let second = sealed(&home, &asked(doc));
    let now = only_one(&second.remembering);
    assert_eq!(now.id, was.id);
    assert_eq!(now.version, 2);
    assert_eq!(now.link_key, was.link_key);
    assert_eq!(now.files_hash, was.files_hash);
    assert_eq!(now.files_key, was.link_key);
    assert!(
        second.going[0].sealed.files.is_none(),
        "its files stay in the envelope already sent"
    );

    let read = &every_reader_opens(&second)[0];
    assert_eq!(read.held["files_hash"].as_str(), was.files_hash.as_deref());
    let key = URL_SAFE_NO_PAD
        .decode(now.link_key.as_ref().unwrap())
        .unwrap();
    let files = first.going[0].sealed.files.as_ref().unwrap();
    let opened =
        opening::files_for(&read.held, files, (now.id.as_str(), "files"), Some(&key)).unwrap();
    assert_eq!(opened.parts, vec![pdf]);
    let manifest: Value = serde_json::from_slice(
        &opening::opened(
            &second.going[0].sealed.main,
            (now.id.as_str(), "main"),
            Some(&key),
        )
        .unwrap()
        .manifest,
    )
    .unwrap();
    assert_eq!(manifest["from"], 1);
    assert_eq!(manifest["version"], 2);
    home.sent(&second.remembering);
    assert_eq!(remembered(&home.state)[file].files_key, was.link_key);
}

#[test]
fn a_file_that_changed_is_sent_again() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("cartilla.pdf", b"%PDF uno");
    let doc = home.wrote(&format!("[c]({leaf})\n"), None);
    let first = sealed(&home, &asked(doc));
    home.sent(&first.remembering);

    let other = home.attached("cartilla.pdf", b"%PDF dos");
    let file = home.state.docs[&doc].file.clone();
    tisty_core::docs::write(&home.paths.docs(), &file, &format!("[c]({other})\n")).unwrap();
    let second = sealed(&home, &asked(doc));
    assert!(second.going[0].sealed.files.is_some());
    assert_ne!(
        only_one(&second.remembering).files_hash,
        only_one(&first.remembering).files_hash
    );
    every_reader_opens(&second);
}

#[test]
fn turning_public_or_private_sends_its_files_again() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("cartilla.pdf", b"%PDF uno");
    let doc = home.wrote(&format!("[c]({leaf})\n"), None);
    let first = sealed(&home, &asked(doc));
    home.sent(&first.remembering);

    let mut public = asked(doc);
    public.public = true;
    let second = sealed(&home, &public);
    assert!(second.going[0].sealed.files.is_some());
    assert_eq!(only_one(&second.remembering).files_key, None);
    every_reader_opens(&second);
    home.sent(&second.remembering);

    let third = sealed(&home, &asked(doc));
    assert!(third.going[0].sealed.files.is_some());
    assert!(only_one(&third.remembering).link_key.is_some());
    every_reader_opens(&third);
}

#[test]
fn no_head_is_used_twice_across_what_it_seals() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("cartilla.pdf", b"%PDF uno");
    let doc = home.wrote(&format!("[c]({leaf})\n"), None);
    let one = sealed(&home, &asked(doc));
    home.sent(&one.remembering);
    let other = sealed(&home, &asked(doc));
    let head_of = |bytes: &[u8]| -> String {
        let size = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let manifest: Value = serde_json::from_slice(&bytes[8..8 + size]).unwrap();
        manifest["head"].as_str().unwrap().to_owned()
    };
    let heads = [
        head_of(&one.going[0].sealed.main),
        head_of(one.going[0].sealed.files.as_ref().unwrap()),
        head_of(&other.going[0].sealed.main),
    ];
    assert_eq!(
        heads
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[test]
fn only_what_is_ones_own_and_open_is_shared() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n", None);
    let page = home.wrote("# Enero\n", Some(doc));
    let refused = |home: &Home, id| {
        let desk = tisty_sharing::Desk {
            paths: &home.paths,
            state: &home.state,
            also: None,
            turned: &|_| None,
        };
        tisty_sharing::gathered(&desk, &asked(id)).err()
    };
    assert!(matches!(refused(&home, page), Some(Unshared::NotADocument)));
    assert!(matches!(
        refused(&home, ulid::Ulid::generate()),
        Some(Unshared::NoSuchDoc)
    ));
    home.commit(Op::DocLock { id: doc });
    assert!(matches!(refused(&home, doc), Some(Unshared::Shut)));
}

#[test]
fn a_copy_of_what_someone_else_shared_is_not_shared_again() {
    let (_tmp, mut home) = home();
    let body = "# De otra persona
";
    let made = tisty_core::docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let id = ulid::Ulid::generate();
    home.commit(Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            file: made.id,
            order: "a0".into(),
            said: Some(tisty_core::event::Said::of(body)),
            guest: true,
            ..Default::default()
        },
    });
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };
    let refused = tisty_sharing::gathered(&desk, &asked(id)).err();
    assert!(matches!(refused, Some(Unshared::NotOwn)), "{refused:?}");
}

#[test]
fn an_anchor_that_is_locked_stays_here_and_the_map_goes_with_who_does() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\n---\n\nsecreto\n", None);
    let en = home.wrote("---\nlang: en\n---\n\nwalks\n", None);
    let pt = home.wrote("---\nlang: pt\n---\n\npasseios\n", None);
    home.joined(en, es);
    home.joined(pt, es);
    home.commit(Op::DocLock { id: es });

    let out = sealed(&home, &asked(en));
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 2);
    assert!(reads.iter().all(|one| !all_text(one).contains("secreto")));
    assert_eq!(
        reads
            .iter()
            .filter(|one| one.held.get("languages").is_some())
            .count(),
        1
    );
    assert_eq!(
        out.told
            .iter()
            .filter(|one| matches!(one, Told::LanguageLeftBehind { .. }))
            .count(),
        1
    );
    assert_eq!(out.remembering.len(), 2);
}

#[test]
fn an_anchor_someone_else_shared_stays_here() {
    let (_tmp, mut home) = home();
    let body = "---\nlang: es\n---\n\nde otra persona\n";
    let made = tisty_core::docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let es = ulid::Ulid::generate();
    home.commit(Op::DocAdd {
        id: es,
        d: tisty_core::event::DocAdd {
            file: made.id,
            order: "a0".into(),
            said: Some(tisty_core::event::Said::of(body)),
            guest: true,
            ..Default::default()
        },
    });
    let en = home.wrote("---\nlang: en\n---\n\nwalks\n", None);
    home.joined(en, es);
    let reads = every_reader_opens(&sealed(&home, &asked(en)));
    assert_eq!(reads.len(), 1);
    assert!(!all_text(&reads[0]).contains("de otra persona"));
}

#[test]
fn a_page_put_away_stays_here_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\n[[Enero]] [[Febrero]]\n", None);
    let enero = home.wrote("# Enero\n\nprivado\n", Some(doc));
    home.wrote("# Febrero\n", Some(doc));
    home.commit(Op::DocArchive { id: enero });

    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert_eq!(read.held["pages"].as_array().unwrap().len(), 1);
    assert_eq!(read.held["pages"][0]["at"], 1);
    assert_eq!(read.held["pages"][0]["title"], "Febrero");
    assert!(!all_text(read).contains("privado"));
    assert!(read.held["html"].as_str().unwrap().contains("Enero"));
    assert!(matches!(&out.told[..], [Told::PageLeftBehind { .. }]));
}

#[test]
fn a_language_put_away_stays_here_and_what_is_put_away_is_not_shared() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\n---\n\nx\n", None);
    let en = home.wrote("---\nlang: en\n---\n\nguardado\n", None);
    home.joined(en, es);
    home.commit(Op::DocArchive { id: en });
    let out = sealed(&home, &asked(es));
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 1);
    assert!(!all_text(&reads[0]).contains("guardado"));
    assert!(matches!(&out.told[..], [Told::LanguageLeftBehind { .. }]));

    let alone = home.wrote("x\n", None);
    home.commit(Op::DocArchive { id: alone });
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };
    let refused = tisty_sharing::gathered(&desk, &asked(alone)).err();
    assert!(matches!(refused, Some(Unshared::Away)), "{refused:?}");
}

#[test]
fn what_is_chosen_and_nobody_names_is_neither_gathered_nor_remembered() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("gone.pdf", b"%PDF");
    let other = home.wrote(&format!("# Otro\n\n[c]({leaf})\n"), None);
    std::fs::remove_file(home.paths.attachments().join(&leaf)).unwrap();
    let doc = home.wrote("# Paseos\n", None);
    let mut asking = asked(doc);
    asking.cited = vec![other];

    let out = sealed(&home, &asking);
    assert_eq!(out.going.len(), 1);
    assert_eq!(out.remembering.len(), 1);
    assert_eq!(out.told, [], "nothing to say of what does not travel");
    assert!(every_reader_opens(&out)[0].held.get("cited").is_none());
}

#[test]
fn words_with_nothing_to_draw_ask_for_no_window() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\nSolo palabras.\n", None);
    home.wrote("# Enero\n\n```mermaid\npie\n```\n", Some(doc));
    home.wrote("# Febrero\n\n::dog orange:: y `code`\n", Some(doc));
    let gathered = home.gathered(&asked(doc));
    let asked_of_it = gathered.unfinished();
    assert_eq!(asked_of_it.len(), 2, "{asked_of_it:?}");
    assert!(
        asked_of_it
            .iter()
            .all(|one| !one.html.contains("Solo palabras"))
    );

    let plain = home.wrote("# Paseos\n", None);
    let gathered = home.gathered(&asked(plain));
    assert!(gathered.unfinished().is_empty());
    let out = home.sealed(gathered, vec![]);
    assert_eq!(every_reader_opens(&out)[0].held["title"], "Paseos");
}

#[test]
fn what_is_chosen_and_cannot_go_stays_as_its_words_and_stops_nothing() {
    let (_tmp, mut home) = home();
    let locked = home.wrote("# Cerrado\n\nprivado\n", None);
    home.commit(Op::DocLock { id: locked });
    let away = home.wrote("# Guardado\n\nprivado\n", None);
    home.commit(Op::DocArchive { id: away });
    let open = home.wrote("# Abierto\n", None);
    let doc = home.wrote("# Paseos\n\n[[Cerrado]] [[Guardado]] [[Abierto]]\n", None);
    let mut asking = asked(doc);
    asking.cited = vec![locked, away, ulid::Ulid::generate(), open];

    let out = sealed(&home, &asking);
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 2);
    assert_eq!(reads[0].held["cited"].as_array().unwrap().len(), 1);
    assert_eq!(reads[0].held["cited"][0]["title"], "Abierto");
    let html = reads[0].held["html"].as_str().unwrap();
    assert!(
        html.contains("Cerrado") && html.contains("Guardado"),
        "{html}"
    );
    assert!(reads.iter().all(|one| !all_text(one).contains("privado")));
    let left: Vec<&Told> = out
        .told
        .iter()
        .filter(|one| matches!(one, Told::CitedLeftBehind { .. }))
        .collect();
    assert_eq!(left.len(), 2, "{:?}", out.told);
}

#[test]
fn a_machine_with_no_identity_makes_no_key_by_asking() {
    let tmp = tempfile::tempdir().unwrap();
    let home = Home::at(tmp.path());
    assert!(signer(&home.paths, &home.state, &home.device).is_none());
    assert!(tisty_core::person::kept_at(&home.paths, &home.device).is_some_and(|at| !at.exists()));
}

#[test]
fn past_the_languages_folio_takes_the_rest_stay_here_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    let mut asked_for = anchor;
    for one in 0..20u8 {
        let code = format!("a{}", char::from(b'a' + one));
        let id = home.wrote(&format!("---\nlang: {code}\n---\n\nwords\n"), None);
        home.joined(id, anchor);
        asked_for = id;
    }

    let out = sealed_within(&home, &asked(asked_for), 20);

    assert_eq!(every_reader_opens(&out).len(), 20);
    assert_eq!(
        out.told,
        [Told::LanguageOverTheTop {
            language: "as".into(),
            top: 20,
        }]
    );
}

fn languages_joined(
    home: &mut Home,
    anchor: tisty_core::model::DocId,
    many: u8,
) -> Vec<tisty_core::model::DocId> {
    (0..many)
        .map(|one| {
            let code = format!("b{}", char::from(b'a' + one));
            let id = home.wrote(&format!("---\nlang: {code}\n---\n\nwords\n"), None);
            home.joined(id, anchor);
            id
        })
        .collect()
}

#[test]
fn a_language_folio_already_holds_keeps_its_place_before_a_new_one() {
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    languages_joined(&mut home, anchor, 19);
    let first = sealed_within(&home, &asked(anchor), 20);
    assert_eq!(first.told, []);
    home.sent(&first.remembering);
    let earlier = home.wrote("---\nlang: aa\n---\n\nwords\n", None);
    home.joined(earlier, anchor);

    let again = sealed_within(&home, &asked(anchor), 20);

    assert_eq!(every_reader_opens(&again).len(), 20);
    assert_eq!(
        again.told,
        [Told::LanguageOverTheTop {
            language: "aa".into(),
            top: 20,
        }]
    );
}

#[test]
fn asking_for_a_new_language_when_folio_holds_as_many_as_it_takes_is_refused_before_anything_goes()
{
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    languages_joined(&mut home, anchor, 19);
    let first = sealed_within(&home, &asked(anchor), 20);
    home.sent(&first.remembering);
    let new = home.wrote("---\nlang: aa\n---\n\nwords\n", None);
    home.joined(new, anchor);

    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };
    let refused = tisty_sharing::gathered_within(&desk, &asked(new), Some(20)).err();

    assert!(
        matches!(refused, Some(Unshared::LanguagesFull)),
        "{refused:?}"
    );
}

#[test]
fn a_language_that_stays_here_still_counts_if_folio_holds_it() {
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    let others = languages_joined(&mut home, anchor, 19);
    let first = sealed_within(&home, &asked(anchor), 20);
    home.sent(&first.remembering);
    home.commit(Op::DocLock { id: others[0] });
    let new = home.wrote("---\nlang: aa\n---\n\nwords\n", None);
    home.joined(new, anchor);

    let again = sealed_within(&home, &asked(anchor), 20);

    assert_eq!(every_reader_opens(&again).len(), 19);
    assert!(
        again
            .told
            .iter()
            .any(|one| matches!(one, Told::LanguageLeftBehind { .. }))
    );
    assert!(
        again
            .told
            .iter()
            .any(|one| matches!(one, Told::LanguageOverTheTop { .. }))
    );
}

#[test]
fn the_top_folio_signs_holds_over_the_one_written_here() {
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    languages_joined(&mut home, anchor, 3);
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };

    let gathered = tisty_sharing::gathered_within(&desk, &asked(anchor), Some(3)).unwrap();

    assert_eq!(
        gathered.told(),
        [Told::LanguageOverTheTop {
            language: "bc".into(),
            top: 3,
        }]
    );
}

#[test]
fn twenty_languages_all_go_and_nobody_is_told() {
    let (_tmp, mut home) = home();
    let anchor = home.wrote("---\nlang: es\n---\n\nhola\n", None);
    for one in 0..19u8 {
        let code = format!("a{}", char::from(b'a' + one));
        let id = home.wrote(&format!("---\nlang: {code}\n---\n\nwords\n"), None);
        home.joined(id, anchor);
    }

    let out = sealed_within(&home, &asked(anchor), 20);

    assert_eq!(every_reader_opens(&out).len(), 20);
    assert_eq!(out.told, []);
}

#[test]
fn a_language_that_stays_is_told_by_its_language() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\n---\n\n# Paseos\n", None);
    let en = home.wrote("---\nlang: en\n---\n\n# Walks\n", None);
    home.joined(en, es);
    home.commit(Op::DocLock { id: en });
    let out = sealed(&home, &asked(es));
    assert_eq!(
        out.told,
        [Told::LanguageLeftBehind {
            language: "en".into()
        }]
    );
}

#[test]
fn a_name_left_blank_signs_with_the_alias_and_no_alias_says_nothing() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("x\n", None);
    let mut asking = asked(doc);
    asking.signed_as = Some("  ".into());
    let out = sealed(&home, &asking);
    assert_eq!(out.told, []);
    assert!(every_reader_opens(&out)[0].held.get("signed_as").is_none());

    home.commit(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("Rina".into()),
            ..Default::default()
        },
    });
    let out = sealed(&home, &asking);
    assert_eq!(out.told, []);
    assert_eq!(every_reader_opens(&out)[0].held["signed_as"], "Rina");
}

#[test]
fn a_picture_never_kept_is_told_by_what_it_said_and_not_as_gone() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("# Paseos\n\n![a](fotos/rina.png)\n", None);
    let out = sealed(&home, &asked(doc));
    every_reader_opens(&out);
    assert_eq!(
        out.told,
        [Told::PictureNotKept {
            document: "Paseos".into(),
            name: "rina.png".into()
        }]
    );

    let doc = home.wrote("# Datos\n\n![a](data:image/png;base64,AAAA)\n", None);
    let out = sealed(&home, &asked(doc));
    assert_eq!(
        out.told,
        [Told::PictureNotKept {
            document: "Datos".into(),
            name: String::new()
        }]
    );
}

#[test]
fn the_person_is_told_of_a_document_by_its_words_and_never_its_markup() {
    let (_tmp, mut home) = home();
    let svg = home.attached("dibujo.svg", b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
    let cartilla = home.attached("cartilla.pdf", b"%PDF");
    let doc = home.wrote(&format!("[La cartilla]({cartilla})\n\n![a]({svg})\n"), None);
    let out = sealed(&home, &asked(doc));
    assert!(
        matches!(&out.told[..], [Told::PictureAsFile { document, .. }] if document == "La cartilla"),
        "{:?}",
        out.told
    );
}

#[test]
fn what_a_language_carries_is_held_to_one_ceiling_in_all() {
    let (_tmp, mut home) = home();
    let one = home.attached("uno.bin", b"uno");
    let other = home.attached("dos.bin", b"dos");
    let doc = home.wrote(&format!("[uno]({one}) [dos]({other})\n"), None);
    for (leaf, size) in [(&one, 40), (&other, 41)] {
        let at = home.paths.attachments().join(leaf);
        let file = std::fs::OpenOptions::new().write(true).open(at).unwrap();
        file.set_len(size * 1024 * 1024).unwrap();
    }
    let gathered = home.gathered(&asked(doc));
    assert!(
        matches!(gathered.told(), [Told::FileAsWords { name, .. }] if name == "dos.bin"),
        "{:?}",
        gathered.told()
    );
}

#[test]
fn a_language_of_its_own_family_is_never_cited() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\n---\n\n# Paseos\n\n[[Walks]]\n", None);
    let en = home.wrote("---\nlang: en\n---\n\n# Walks\n", None);
    home.joined(en, es);
    let mut asking = asked(es);
    asking.cited = vec![en];

    let reads = every_reader_opens(&sealed(&home, &asking));
    assert_eq!(reads.len(), 2);
    assert!(reads.iter().all(|one| one.held.get("cited").is_none()));
}

#[test]
fn the_words_of_a_task_stay_with_the_task() {
    let (_tmp, mut home) = home();
    let body = "# De una tarea\n";
    let made = tisty_core::docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let id = ulid::Ulid::generate();
    let task = ulid::Ulid::generate();
    home.commit(Op::TaskAdd {
        id: task,
        d: tisty_core::event::TaskAdd::new("Una tarea", "a0"),
    });
    home.commit(Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            file: made.id,
            order: "a0".into(),
            said: Some(tisty_core::event::Said::of(body)),
            task_of: Some(task),
            ..Default::default()
        },
    });
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };
    let refused = tisty_sharing::gathered(&desk, &asked(id)).err();
    assert!(
        matches!(refused, Some(Unshared::NotADocument)),
        "{refused:?}"
    );
}

#[test]
fn a_family_chosen_twice_goes_once_under_one_identity() {
    let (_tmp, mut home) = home();
    let otro = home.wrote("---\nlang: es\n---\n\n# Otro\n", None);
    let other = home.wrote("---\nlang: en\n---\n\n# Other\n", None);
    home.joined(other, otro);
    let doc = home.wrote("[[Otro]] y [[Other]]\n", None);
    let mut asking = asked(doc);
    asking.cited = vec![otro, other, otro];

    let out = sealed(&home, &asking);
    let reads = every_reader_opens(&out);
    assert_eq!(reads.len(), 3);
    assert_eq!(out.remembering.len(), 3);
    let files: std::collections::BTreeSet<&String> =
        out.remembering.iter().map(|(file, _)| file).collect();
    assert_eq!(files.len(), 3);
    assert_eq!(
        reads[0].held["cited"].as_array().unwrap().len(),
        1,
        "both names lead to one"
    );
    let html = reads[0].held["html"].as_str().unwrap();
    assert_eq!(html.matches("tisty:cited/0").count(), 2, "{html}");
}

#[test]
fn what_is_drawn_and_also_laid_in_as_a_file_goes_both_ways() {
    let (_tmp, mut home) = home();
    let picture = a_picture(8, 8, 1);
    let leaf = home.attached("foto.png", &picture);
    let doc = home.wrote(&format!("![a]({leaf}) ![b]({leaf} \"file\")\n"), None);
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert_eq!(read.pictures.len(), 1);
    assert_eq!(read.files, vec![picture]);
    let html = read.held["html"].as_str().unwrap();
    assert!(
        html.contains("tisty:picture/0") && html.contains("tisty:file/0"),
        "{html}"
    );
    let body = read.held["body"].as_str().unwrap();
    assert!(
        body.contains("tisty:picture/0") && body.contains("tisty:file/0"),
        "{body}"
    );
    assert_eq!(out.told, []);
}

#[test]
fn a_link_to_a_picture_that_travels_leads_to_it_and_its_bytes_go_once() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("foto.png", &a_picture(8, 8, 1));
    let doc = home.wrote(&format!("![a]({leaf}) [b]({leaf})\n"), None);
    let read = &every_reader_opens(&sealed(&home, &asked(doc)))[0];
    assert_eq!(read.pictures.len(), 1);
    assert!(read.files.is_empty());
    assert_eq!(
        read.held["html"]
            .as_str()
            .unwrap()
            .matches("tisty:picture/0")
            .count(),
        2
    );
}

#[test]
fn a_file_too_large_to_hold_goes_as_its_words_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let leaf = home.attached("pelicula.mp4", b"small for now");
    let doc = home.wrote(&format!("[La pelicula]({leaf})\n"), None);
    let at = home.paths.attachments().join(&leaf);
    let file = std::fs::OpenOptions::new().write(true).open(at).unwrap();
    file.set_len(64 * 1024 * 1024 + 1).unwrap();
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert!(read.files.is_empty());
    assert!(read.held["html"].as_str().unwrap().contains("La pelicula"));
    assert!(matches!(&out.told[..], [Told::FileAsWords { .. }]));
}

#[test]
fn an_address_on_a_picture_goes_as_a_link_and_is_told_as_one() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("![Escribeme](mailto:a@b.example)\n", None);
    let out = sealed(&home, &asked(doc));
    let read = &every_reader_opens(&out)[0];
    assert!(
        read.held["html"]
            .as_str()
            .unwrap()
            .contains("href=\"mailto:a@b.example\"")
    );
    assert!(matches!(&out.told[..], [Told::PictureAsLink { .. }]));
}

#[test]
fn a_language_that_is_locked_stays_here_and_the_person_is_told() {
    let (_tmp, mut home) = home();
    let es = home.wrote("---\nlang: es\n---\n\nx\n", None);
    let en = home.wrote("---\nlang: en\n---\n\ny\n", None);
    home.joined(en, es);
    home.commit(Op::DocLock { id: en });
    let out = sealed(&home, &asked(es));
    assert_eq!(every_reader_opens(&out).len(), 1);
    assert!(matches!(&out.told[..], [Told::LanguageLeftBehind { .. }]));
}

#[test]
fn a_machine_that_answers_to_no_root_has_nothing_to_sign_with() {
    let tmp = tempfile::tempdir().unwrap();
    let home = Home::at(tmp.path());
    assert!(signer(&home.paths, &home.state, &home.device).is_none());
}

#[test]
fn it_signs_as_the_root_this_machine_answers_to_under_its_certificate() {
    let (_tmp, home) = home();
    let (key, certificate) = home.signer();
    assert_eq!(certificate.share, key.shown());
    assert!(certificate.vouches().is_ok());
    assert_eq!(
        certificate.root,
        tisty_share_proto::seed::Seed::from_bytes(&[0x33; 32])
            .root()
            .person()
            .shown()
    );
}

#[test]
fn what_it_remembers_goes_with_the_log_and_keeps_no_file_of_its_own() {
    let (_tmp, mut home) = home();
    let doc = home.wrote("x\n", None);
    let out = sealed(&home, &asked(doc));
    home.sent(&out.remembering);

    assert_eq!(remembered(&home.state).len(), 1);
    assert!(!home.paths.private().join("sharing.json").exists());
}
