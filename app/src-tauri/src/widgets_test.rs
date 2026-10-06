use super::*;

fn asked(path: &str) -> Uri {
    format!("http://widget.localhost{path}").parse().unwrap()
}

fn said(answer: &Response<Vec<u8>>) -> String {
    String::from_utf8(answer.body().clone()).unwrap()
}

#[test]
fn a_lent_widget_is_served_inside_its_own_fence() {
    let lent = Lent::default();
    let id = lent
        .lend("<div class=\"stat\"><b>4</b></div>".into())
        .unwrap();

    let answer = lent.shown(&asked(&format!("/{id}")));

    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(
        answer.headers()[header::CONTENT_SECURITY_POLICY],
        FENCED,
        "a widget served without its own policy inherits whatever the frame allows"
    );
    let page = said(&answer);
    assert!(page.contains("<div class=\"stat\"><b>4</b></div>"));
    assert!(
        page.contains(".stat{"),
        "the kit that makes plain HTML look like Tisty is missing"
    );
    assert!(!page.contains("class=\"dark\""));
}

#[test]
fn the_fence_lets_no_request_out() {
    for wanted in [
        "default-src 'none'",
        "connect-src 'none'",
        "frame-src 'none'",
        "form-action 'none'",
        "base-uri 'none'",
    ] {
        assert!(FENCED.contains(wanted), "the policy lost {wanted}");
    }
    assert!(
        !FENCED.contains("http"),
        "the policy names a place a widget could reach"
    );
}

#[test]
fn a_dark_window_asks_for_the_dark_page() {
    let lent = Lent::default();
    let id = lent.lend("<p>x</p>".into()).unwrap();

    let page = said(&lent.shown(&asked(&format!("/{id}?dark=1"))));

    assert!(page.contains("<html class=\"dark\">"));
}

#[test]
fn nothing_is_served_for_a_name_never_lent_or_already_taken_back() {
    let lent = Lent::default();
    let unknown = lent.shown(&asked("/01jzzzzzzzzzzzzzzzzzzzzzzz"));
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(unknown.headers()[header::CONTENT_SECURITY_POLICY], FENCED);
    assert!(said(&unknown).is_empty());

    let id = lent.lend("<p>gone</p>".into()).unwrap();
    lent.take_back(&id);
    assert_eq!(
        lent.shown(&asked(&format!("/{id}"))).status(),
        StatusCode::NOT_FOUND,
        "a closed document's widget was still served"
    );
}

#[test]
fn a_widget_past_the_ceiling_is_refused() {
    let lent = Lent::default();
    let refused = lent.lend("x".repeat(LENT_AT_MOST + 1)).unwrap_err();
    assert_eq!(refused.code, "widgetTooBig");
    assert!(lent.lend("x".repeat(LENT_AT_MOST)).is_ok());
}

#[test]
fn no_more_than_so_many_are_held_at_once_and_the_oldest_goes_first() {
    let lent = Lent::default();
    let first = lent.lend("<p>first</p>".into()).unwrap();
    for _ in 0..OUT_AT_ONCE {
        lent.lend("<p>more</p>".into()).unwrap();
    }
    assert_eq!(lent.0.lock().unwrap().len(), OUT_AT_ONCE);
    assert_eq!(
        lent.shown(&asked(&format!("/{first}"))).status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn the_page_takes_the_window_theme_down_to_its_scrollbars() {
    let page = shell("<p>x</p>", true);
    assert!(page.contains(":root.dark{color-scheme:dark}"));
    assert!(
        page.contains("scrollbar-width:thin"),
        "a widget's scroll areas would draw the system's bars instead of Tisty's"
    );
}

#[test]
fn only_an_attached_page_is_lent_from_the_store() {
    assert!(a_page("attachments/ab/informe-12345678.html"));
    assert!(a_page("attachments/ab/INFORME-12345678.HTM"));
    for one in [
        "attachments/ab/foto-12345678.png",
        "attachments/ab/notas-12345678.html.txt",
        "docs/informe.html",
        "../attachments/ab/x.html",
        "C:/Users/x/informe.html",
        "tisty:doc/abc-0001",
    ] {
        assert!(!a_page(one), "{one} was taken for an attached page");
    }
}

#[test]
fn an_attached_page_has_its_own_ceiling_and_survives_bytes_that_are_not_text() {
    let lent = Lent::default();
    assert_eq!(
        lent.lend_kept(vec![b'x'; KEPT_AT_MOST + 1])
            .unwrap_err()
            .code,
        "pageTooBig"
    );
    let id = lent
        .lend_kept(vec![b'<', b'p', b'>', 0xff, b'<', b'/', b'p', b'>'])
        .unwrap()
        .id;
    assert_eq!(
        lent.shown(&asked(&format!("/{id}"))).status(),
        StatusCode::OK
    );
}

#[test]
fn the_frame_is_measured_by_what_the_widget_holds_not_by_the_frame_itself() {
    let page = shell("<p>x</p>", false);
    assert!(
        page.contains("document.querySelector(\"main.w\")"),
        "measuring the whole page never reads smaller than the frame, so it could only grow"
    );
    assert!(!page.contains("document.documentElement.scrollHeight"));
}

#[test]
fn an_attached_whole_page_is_served_as_itself_under_the_page_policy() {
    let lent = Lent::default();
    let borrowed = lent
        .lend_kept(
            b"<!DOCTYPE html><html><head><title>t</title></head><body><p>hola</p></body></html>"
                .to_vec(),
        )
        .unwrap();
    assert!(
        borrowed.whole,
        "the window would theme a whole page through its address"
    );
    let id = borrowed.id;

    let answer = lent.shown(&asked(&format!("/{id}?dark=1")));

    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(answer.headers()[header::CONTENT_SECURITY_POLICY], PAGED);
    let page = said(&answer);
    assert!(
        !page.contains("<main class=\"w\">") && page.contains("</script><html><head><title>t"),
        "a whole page wrapped inside the widget kit nests one document in another"
    );
    assert!(
        page.starts_with("<!DOCTYPE html><script>") && page.ends_with("<p>hola</p></body></html>"),
        "the measurer goes right after the declaration, before anything the page holds"
    );
}

#[test]
fn an_attached_fragment_keeps_the_widget_fence() {
    let lent = Lent::default();
    let borrowed = lent
        .lend_kept(b"<div class=\"card\">x</div>".to_vec())
        .unwrap();
    assert!(!borrowed.whole);
    let id = borrowed.id;

    let answer = lent.shown(&asked(&format!("/{id}")));

    assert_eq!(answer.headers()[header::CONTENT_SECURITY_POLICY], FENCED);
    assert!(said(&answer).contains("<main class=\"w\"><div class=\"card\">x</div></main>"));
}

#[test]
fn a_whole_page_is_told_by_how_it_opens() {
    for one in [
        "<!doctype html><p>x</p>",
        "  \n<!DOCTYPE HTML>",
        "\u{feff}<html lang=\"es\">",
        "<HTML>",
        "<!-- generated by a tool --><!DOCTYPE html><p>x</p>",
        "<!-- one --> <!-- two -->\n<html>",
        "<head><title>t</title></head><p>x</p>",
    ] {
        assert!(whole(one), "{one:?} was not taken for a whole page");
    }
    for one in [
        "<div>x</div>",
        "<p>&lt;html&gt;</p>",
        "",
        "texto",
        "<!-- a note --><div>x</div>",
        "<!-- never closed <html>",
    ] {
        assert!(!whole(one), "{one:?} was taken for a whole page");
    }
}

#[test]
fn a_page_without_a_body_still_gets_its_measurer() {
    let page = paged("<p>sin cuerpo</p>");
    assert!(page.starts_with("<script>") && page.ends_with("</script><p>sin cuerpo</p>"));
}

#[test]
fn the_measurer_cannot_land_inside_a_script_of_the_page() {
    let page = paged("<!doctype html><script>const t = \"</body>\";</script><p>x</p>");
    assert!(
        page.starts_with("<!doctype html><script>(()=>"),
        "the measurer went after the page's own markup, where a string could swallow it"
    );
    assert!(page.ends_with("<script>const t = \"</body>\";</script><p>x</p>"));
}

#[test]
fn the_measurer_keeps_the_declaration_first_after_comments() {
    let page = paged("<!-- hecho a mano -->\n<!DOCTYPE html><html><body></body></html>");
    assert!(
        page.starts_with("<!-- hecho a mano -->\n<!DOCTYPE html><script>"),
        "a script before the doctype would drop the page into quirks mode"
    );
}

#[test]
fn a_page_is_measured_by_its_loose_text_and_its_closing_margins() {
    let page = paged("<html><body></body></html>");
    for wanted in [
        "b.childNodes",
        "nodeType===3",
        "px(s.marginBottom)",
        "px(s.paddingBottom)",
        "px(s.borderBottomWidth)",
    ] {
        assert!(page.contains(wanted), "the measurer lost {wanted}");
    }
}

#[test]
fn the_page_policy_lets_it_unpack_itself_but_reach_no_one() {
    for wanted in [
        "default-src 'none'",
        "script-src 'unsafe-inline' 'unsafe-eval' blob:",
        "connect-src data: blob:",
        "frame-src blob: data:",
        "form-action 'none'",
        "base-uri 'none'",
    ] {
        assert!(PAGED.contains(wanted), "the page policy lost {wanted}");
    }
    for never in ["http", "'self'", "*", "ws:", "wss:"] {
        assert!(
            !PAGED.contains(never),
            "the page policy lets a page reach out through {never}"
        );
    }
}

#[test]
fn a_page_is_measured_by_what_flows_in_its_body_even_after_it_replaces_itself() {
    let page = paged("<html><body></body></html>");
    assert!(
        page.contains("position===\"fixed\""),
        "a fixed overlay as tall as the frame would keep the frame from ever shrinking"
    );
    assert!(
        page.contains("document.documentElement!==root"),
        "a page that swaps its document would be measured through the one it threw away"
    );
}

#[test]
fn a_page_too_large_is_refused_by_its_size_before_a_byte_is_read() {
    assert!(small_enough(KEPT_AT_MOST as u64).is_ok());
    assert_eq!(
        small_enough(750 * 1024 * 1024).unwrap_err().code,
        "pageTooBig",
        "a page a document may hold, far past the ceiling, was let through to be read whole"
    );
}

#[test]
fn the_theme_reaches_the_page_through_its_address_and_nothing_is_posted_into_it() {
    let page = shell("<p>x</p>", false);
    assert!(
        page.contains("hashchange"),
        "a theme change would need a message posted into the frame"
    );
    assert!(
        !page.contains("addEventListener(\"message\""),
        "the page listens for messages, and any window could send one"
    );
}

#[test]
fn a_page_that_loads_again_reads_its_theme_from_its_address() {
    let page = shell("<p>x</p>", true);
    assert!(
        page.contains("themed();"),
        "a reloaded frame would keep the theme it was first served with"
    );
}
