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
        .unwrap();
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
