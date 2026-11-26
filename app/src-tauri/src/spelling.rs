//! The page keeps the right-click for its own menu, so the suggestions the browser would offer are asked of the system here.

use serde::Serialize;

const GUESSES_SHOWN: usize = 5;
const LONGEST_WORD: usize = 64;

#[derive(Serialize, Default, Debug, PartialEq, Eq)]
pub struct Spelled {
    pub wrong: bool,
    pub guesses: Vec<String>,
}

fn spellable(word: &str) -> Option<String> {
    let word = word.trim();
    let letters = word.chars().count();
    (letters > 0 && letters <= LONGEST_WORD && !word.chars().any(char::is_whitespace))
        .then(|| word.to_owned())
}

fn primary(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

pub(crate) fn tongue_chosen(
    supported: &[String],
    asked: Option<&str>,
    own: Option<&str>,
) -> Option<String> {
    let same = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    let kin = |a: &str, b: &str| same(primary(a), primary(b));
    let exact = |tag: &str| supported.iter().find(|one| same(one, tag)).cloned();
    let family = |tag: &str| supported.iter().find(|one| kin(one, tag)).cloned();
    let asked = asked.filter(|tag| !tag.is_empty());
    let own = own.filter(|tag| !tag.is_empty());
    asked
        .and_then(exact)
        .or_else(|| {
            own.filter(|o| asked.is_some_and(|a| kin(a, o)))
                .and_then(exact)
        })
        .or_else(|| asked.and_then(family))
        .or_else(|| own.and_then(exact))
        .or_else(|| own.and_then(family))
        .or_else(|| supported.first().cloned())
}

#[tauri::command(async)]
pub async fn word_spelled(app: tauri::AppHandle, word: String, tongue: Option<String>) -> Spelled {
    let Some(word) = spellable(&word) else {
        return Spelled::default();
    };
    let mut said = asked_of_system(app, move || system::spelled(&word, tongue.as_deref()))
        .await
        .unwrap_or_default();
    said.guesses.truncate(GUESSES_SHOWN);
    said
}

#[tauri::command(async)]
pub async fn word_learned(app: tauri::AppHandle, word: String, tongue: Option<String>) -> bool {
    let Some(word) = spellable(&word) else {
        return false;
    };
    asked_of_system(app, move || system::learned(&word, tongue.as_deref()))
        .await
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
async fn asked_of_system<T: Send + 'static>(
    app: tauri::AppHandle,
    ask: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(ask());
    })
    .ok()?;
    tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(SYSTEM_ANSWERS_WITHIN).ok())
        .await
        .ok()
        .flatten()
}

#[cfg(target_os = "macos")]
const SYSTEM_ANSWERS_WITHIN: std::time::Duration = std::time::Duration::from_secs(2);

#[cfg(not(target_os = "macos"))]
async fn asked_of_system<T: Send + 'static>(
    _app: tauri::AppHandle,
    ask: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    tauri::async_runtime::spawn_blocking(ask).await.ok()
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod system {
    use super::{Spelled, tongue_chosen};
    use windows::Win32::Globalization::{
        GetUserDefaultLocaleName, ISpellChecker, ISpellCheckerFactory, SpellCheckerFactory,
    };
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        CoTaskMemFree, IEnumString,
    };
    use windows::core::{HSTRING, PWSTR};

    const LOCALE_NAME_MAX_LENGTH: usize = 85;

    fn read(listed: &IEnumString) -> Vec<String> {
        let mut found = Vec::new();
        loop {
            let mut one = [PWSTR::null()];
            let mut fetched = 0u32;
            let more = unsafe { listed.Next(&mut one, Some(&raw mut fetched)) };
            if more.is_err() || fetched == 0 {
                return found;
            }
            unsafe {
                if let Ok(word) = one[0].to_string() {
                    found.push(word);
                }
                CoTaskMemFree(Some(one[0].0 as *const _));
            }
        }
    }

    fn own_locale() -> Option<String> {
        let mut name = [0u16; LOCALE_NAME_MAX_LENGTH];
        let len = unsafe { GetUserDefaultLocaleName(&mut name) };
        let len = usize::try_from(len).ok()?.checked_sub(1)?;
        String::from_utf16(&name[..len]).ok()
    }

    fn checker(asked: Option<&str>) -> Option<ISpellChecker> {
        unsafe {
            // A thread already in an apartment answers S_FALSE or a changed mode, and the speller works in either.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let factory: ISpellCheckerFactory =
                CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER).ok()?;
            let supported = read(&factory.SupportedLanguages().ok()?);
            let tongue = tongue_chosen(&supported, asked, own_locale().as_deref())?;
            factory.CreateSpellChecker(&HSTRING::from(tongue)).ok()
        }
    }

    pub(super) fn spelled(word: &str, asked: Option<&str>) -> Spelled {
        let Some(checker) = checker(asked) else {
            return Spelled::default();
        };
        let said = HSTRING::from(word);
        unsafe {
            let Ok(errors) = checker.Check(&said) else {
                return Spelled::default();
            };
            let mut first = None;
            if errors.Next(&raw mut first).is_err() || first.is_none() {
                return Spelled::default();
            }
            let guesses = checker
                .Suggest(&said)
                .map(|listed| read(&listed))
                .unwrap_or_default();
            Spelled {
                wrong: true,
                guesses,
            }
        }
    }

    pub(super) fn learned(word: &str, asked: Option<&str>) -> bool {
        checker(asked).is_some_and(|checker| unsafe { checker.Add(&HSTRING::from(word)).is_ok() })
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod system {
    use super::Spelled;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSArray, NSRange, NSString};

    fn checker() -> Option<Retained<AnyObject>> {
        unsafe { msg_send![class!(NSSpellChecker), sharedSpellChecker] }
    }

    pub(super) fn spelled(word: &str, _asked: Option<&str>) -> Spelled {
        let Some(checker) = checker() else {
            return Spelled::default();
        };
        let said = NSString::from_str(word);
        unsafe {
            let missed: NSRange =
                msg_send![&checker, checkSpellingOfString: &*said, startingAt: 0isize];
            if missed.length == 0 {
                return Spelled::default();
            }
            let none: Option<&NSString> = None;
            let guessed: Option<Retained<NSArray<NSString>>> = msg_send![
                &checker,
                guessesForWordRange: missed,
                inString: &*said,
                language: none,
                inSpellDocumentWithTag: 0isize
            ];
            let guesses = guessed
                .map(|all| all.iter().map(|one| one.to_string()).collect())
                .unwrap_or_default();
            Spelled {
                wrong: true,
                guesses,
            }
        }
    }

    pub(super) fn learned(word: &str, _asked: Option<&str>) -> bool {
        let Some(checker) = checker() else {
            return false;
        };
        let said = NSString::from_str(word);
        unsafe {
            let _: () = msg_send![&checker, learnWord: &*said];
        }
        true
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod system {
    use super::Spelled;

    pub(super) fn spelled(_word: &str, _asked: Option<&str>) -> Spelled {
        Spelled::default()
    }

    pub(super) fn learned(_word: &str, _asked: Option<&str>) -> bool {
        false
    }
}

#[cfg(test)]
#[path = "spelling_test.rs"]
mod tests;
