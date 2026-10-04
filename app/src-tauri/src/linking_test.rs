use super::*;

fn given_back(said: &str) -> String {
    let asked = said
        .strip_prefix("linkunbound://open?url=")
        .expect("the shape it is handed in");
    percent_encoding::percent_decode_str(asked)
        .decode_utf8()
        .expect("what went in was text")
        .into_owned()
}

#[test]
fn a_link_with_everything_a_url_can_carry_comes_back_whole() {
    for one in [
        "https://example.com/a?b=1&c=2#part+two",
        "https://example.com/caña?q=qué+tal#final",
        "https://example.com/already%20encoded?x=%26",
        "http://example.com/",
        "https://example.com/a/b/c?d=:/?&=#+",
    ] {
        assert_eq!(given_back(&through(one)), one, "it did not survive: {one}");
    }
}

#[test]
fn nothing_a_reader_could_cut_the_link_at_is_left_as_it_was() {
    let said = through("https://example.com/a?b=1&c=2#d+e");

    let asked = said.strip_prefix("linkunbound://open?url=").unwrap();
    for one in [':', '/', '?', '&', '=', '#', '+'] {
        assert!(
            !asked.contains(one),
            "{one} travelled as itself, so the link arrives cut"
        );
    }
}

#[test]
fn only_a_link_of_the_web_is_handed_over() {
    assert!(on_the_web("https://example.com"));
    assert!(on_the_web("HTTP://example.com"));
    assert!(!on_the_web("mailto:someone@example.com"));
    assert!(!on_the_web("file:///C:/one.txt"));
    assert!(!on_the_web("ms-windows-store://review/?ProductId=9"));
    assert!(!on_the_web("tisty://whatever"));
    assert!(!on_the_web("https:"));
    assert!(!on_the_web("nothing at all"));
}

#[test]
fn what_is_opened_is_the_real_link_until_something_answers_for_the_scheme() {
    let one = "https://example.com/a?b=1";

    assert_eq!(as_opened(one, false), one, "it was wrapped for nobody");
    assert_eq!(as_opened(one, true), through(one));
    assert_eq!(
        as_opened("mailto:someone@example.com", true),
        "mailto:someone@example.com",
        "a link that is not of the web was handed to the wrapper"
    );
}

#[test]
fn a_link_carrying_a_control_character_goes_to_the_browser_as_it_is() {
    for one in [
        "https://example.com/a\tb",
        "https://example.com/a\nb",
        "https://example.com/\u{7f}",
        "https://example.com/\u{85}",
    ] {
        assert_eq!(
            as_opened(one, true),
            one,
            "it was handed to a reader that refuses it"
        );
    }
    assert_eq!(
        as_opened("https://example.com/a b", true),
        through("https://example.com/a b"),
        "a space inside travels encoded and is not a reason to skip LinkUnbound"
    );
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn the_system_answers_for_a_scheme_only_when_something_registered_it() {
    assert!(answered("https"), "a machine with no browser at all");
    assert!(
        !answered("tisty-nobody-registers-this"),
        "a scheme nothing registered was said to have a handler"
    );
}

#[cfg(windows)]
#[test]
fn a_scheme_a_packaged_app_declares_has_a_handler() {
    assert!(
        answered("ms-settings"),
        "a scheme only a packaged app answers for, as a Store LinkUnbound does, was said to have none"
    );
}

#[cfg(windows)]
#[test]
fn the_open_with_picker_is_not_a_handler() {
    use crate::desktop::answered_by;
    assert!(answered_by(
        Some(r"C:\Program Files\LinkUnbound\linkunbound-shell.exe"),
        None
    ));
    assert!(answered_by(
        None,
        Some("rgdevment.LinkUnbound-BrowserPicker_kdjgfdc2rb3gc!LinkUnbound")
    ));
    assert!(
        !answered_by(Some(r"C:\WINDOWS\system32\OpenWith.exe"), None),
        "the picker Windows offers when nothing answers was taken for an app"
    );
    assert!(!answered_by(
        Some(r"c:\windows\SYSTEM32\openwith.EXE"),
        None
    ));
    assert!(!answered_by(None, None));
}
