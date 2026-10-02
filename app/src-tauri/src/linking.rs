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

pub fn as_opened(url: &str, a_handler: bool) -> String {
    match a_handler && on_the_web(url) {
        true => through(url),
        false => url.to_string(),
    }
}

#[cfg(windows)]
pub fn a_handler_is_there() -> bool {
    winreg::RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT)
        .open_subkey(SCHEME)
        .is_ok_and(|key| {
            key.get_raw_value("URL Protocol").is_ok()
                || key.open_subkey(r"shell\open\command").is_ok()
        })
}

#[cfg(not(windows))]
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
