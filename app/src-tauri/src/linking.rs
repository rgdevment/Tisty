use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::{Answer, Refusal};

const SCHEME: &str = "linkunbound";

pub fn on_the_web(url: &str) -> bool {
    matches!(url.split_once(':'), Some((scheme, rest))
        if !rest.is_empty()
            && (scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")))
}

pub fn through(url: &str) -> String {
    format!(
        "{SCHEME}://open?url={}",
        utf8_percent_encode(url, NON_ALPHANUMERIC)
    )
}

// LinkUnbound refuses a link carrying control characters and says nothing, so the browser takes it.
pub fn as_opened(url: &str, a_handler: bool) -> String {
    match a_handler && on_the_web(url) && !url.chars().any(char::is_control) {
        true => through(url),
        false => url.to_string(),
    }
}

#[cfg(any(windows, target_os = "macos"))]
pub fn a_handler_is_there() -> bool {
    answered(SCHEME)
}

// A packaged app declares its scheme in its manifest and never writes HKCR, so the shell is asked instead.
#[cfg(windows)]
fn answered(scheme: &str) -> bool {
    crate::desktop::answers_for(scheme)
}

#[cfg(target_os = "macos")]
fn answered(scheme: &str) -> bool {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    NSURL::URLWithString(&NSString::from_str(&format!("{scheme}://open"))).is_some_and(|probe| {
        NSWorkspace::sharedWorkspace()
            .URLForApplicationToOpenURL(&probe)
            .is_some()
    })
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn a_handler_is_there() -> bool {
    false
}

#[tauri::command]
pub fn open_link(url: String) -> Answer<()> {
    let said = url.trim();
    let asked = as_opened(said, a_handler_is_there());
    if asked != said && tauri_plugin_opener::open_url(&asked, None::<&str>).is_ok() {
        return Ok(());
    }
    tauri_plugin_opener::open_url(said, None::<&str>)
        .map_err(|_| Refusal::about("cannotOpen", said.to_string()))
}

#[cfg(test)]
#[path = "linking_test.rs"]
mod tests;
