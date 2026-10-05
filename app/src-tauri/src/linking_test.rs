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

#[cfg(windows)]
#[test]
fn a_scheme_left_undecided_is_not_taken_for_one_that_has_an_app() {
    use crate::desktop::answered_by;
    assert!(
        !answered_by(Some(r"C:\WINDOWS\system32\OpenWith.exe"), Some("Undecided")),
        "mailto and tel on a machine with no default answer exactly this, and wrapping a link for \
         them shows the «look for an app» dialog the probe exists to avoid"
    );
}

#[cfg(windows)]
#[test]
fn only_a_package_counts_where_no_executable_answers() {
    use crate::desktop::answered_by;
    assert!(answered_by(
        None,
        Some("AppX1h1kv4gmb0dpfenf5p98f1a1d3btwnwj")
    ));
    assert!(answered_by(
        None,
        Some("appxfvdy2xs18pcp2dv99rrcxe3kqmx56dq6")
    ));
    assert!(
        !answered_by(None, Some("linkunbound")),
        "a protocol key left with no command answers with its own name, and that is no app"
    );
    assert!(!answered_by(None, Some("Undecided")));
    assert!(!answered_by(None, Some("App")));
}

#[cfg(windows)]
#[test]
fn the_machine_never_takes_the_picker_for_a_handler() {
    let executable =
        crate::desktop::associated("mailto", windows::Win32::UI::Shell::ASSOCSTR_EXECUTABLE);
    let picked = executable
        .as_deref()
        .and_then(|one| std::path::Path::new(one).file_name())
        .and_then(|leaf| leaf.to_str())
        .is_some_and(|leaf| leaf.eq_ignore_ascii_case("OpenWith.exe"));
    if picked {
        assert!(
            !answered("mailto"),
            "the shell sends mailto to the picker here, so nothing answers it"
        );
    }
}
