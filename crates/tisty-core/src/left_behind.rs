use std::path::Path;

use crate::moved::Word;

/// A note for whoever opens a folder this machine stopped syncing.
pub const NOTICE: &str = "README - Tisty.txt";
const HEAD: &str = "Tisty · README";

/// Never written over a file of the same name that Tisty did not write.
pub fn left_notice(place: &Path, said: &str) {
    let Some(word) = crate::moved::read(said) else {
        return;
    };
    let at = place.join(NOTICE);
    if at.exists() && !ours(&at) {
        return;
    }
    if let Err(why) = crate::store::write_atomic(&at, noticed(&word).as_bytes()) {
        crate::witness::note(
            crate::witness::channel::SYNC,
            "the note for the folder left behind could not be written",
            &[("why", crate::witness::Fact::Why(why.to_string()))],
        );
    }
}

/// Only Tisty's own note goes, once a machine syncs in the place again.
pub fn notice_gone(place: &Path) {
    let at = place.join(NOTICE);
    if ours(&at) {
        let _ = std::fs::remove_file(at);
    }
}

fn ours(at: &Path) -> bool {
    std::fs::read_to_string(at).is_ok_and(|there| there.starts_with(HEAD))
}

pub fn noticed(word: &Word) -> String {
    noticed_in(word, &crate::model::spoken(None))
}

/// The language this computer speaks goes first; the other follows for whoever opens it elsewhere.
pub fn noticed_in(word: &Word, spoken: &str) -> String {
    let name = crate::config::nicknamed(&word.by);
    let date = i64::try_from(word.at)
        .ok()
        .and_then(|at| jiff::Timestamp::from_second(at).ok())
        .map(|at| at.to_zoned(jiff::tz::TimeZone::system()).date().to_string())
        .unwrap_or_default();
    let (es, en) = match crate::config::bound(&word.to) {
        crate::config::Bound::Cloud { provider, email } => {
            let shown = crate::config::shown_of(provider);
            let whose = match email.is_empty() {
                true => String::new(),
                false => format!(" ({email})"),
            };
            (
                format!("en el espacio de Tisty en {shown}{whose}"),
                format!("in Tisty's space in {shown}{whose}"),
            )
        }
        crate::config::Bound::Folder(folder) => (
            format!("en otra carpeta ({folder})"),
            format!("in another folder ({folder})"),
        ),
        crate::config::Bound::Alone => (
            "solo en ese equipo".to_string(),
            "only on that computer".to_string(),
        ),
    };
    let spanish = format!(
        "El equipo «{name}» dejó de sincronizar Tisty en esta carpeta el {date}.\n\
         Sus tareas y documentos ahora están {es}.\n\
         Lo que quedó aquí ya no se actualiza desde ese equipo.\n\
         Para volver: abre Tisty en ese equipo, ve a Configuración › Tus datos y elige esta carpeta.\n"
    );
    let english = format!(
        "The computer \"{name}\" stopped syncing Tisty in this folder on {date}.\n\
         Its tasks and documents are now {en}.\n\
         What stayed here is no longer kept up to date from that computer.\n\
         To come back: open Tisty on that computer, go to Settings › Your data and choose this folder.\n"
    );
    match spoken.to_lowercase().starts_with("es") {
        true => format!("{HEAD}\n\n{spanish}\n{english}"),
        false => format!("{HEAD}\n\n{english}\n{spanish}"),
    }
}

#[cfg(test)]
#[path = "left_behind_test.rs"]
mod tests;
