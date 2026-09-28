use tisty_core::witness::{self, Fact, channel};

use crate::update;

/// A refusal the person could not have caused by asking for something ordinary: it says the store
/// and the window disagree, and that is worth a line in a log somebody will send us.
const TELLS_OF_TROUBLE: &[&str] = &[
    "noSuchDoc",
    "noSuchTask",
    "deleteRefused",
    "internal",
    "internalNamed",
    "storeNewer",
    "otherStore",
    "wouldReset",
];

pub const RELEASES: &str = "https://github.com/rgdevment/Tisty/releases/latest";

/// The releases page hands a copy kept by the Store an installer that would settle beside the
/// package instead of replacing it, and leave the person with two Tistys.
const IN_THE_STORE: &str = "https://apps.microsoft.com/detail/9PGVWXD8X93N";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Refusal {
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub fn where_it_comes_from() -> &'static str {
    match update::route().route {
        update::Route::Store => IN_THE_STORE,
        _ => RELEASES,
    }
}

pub fn speaks_spanish() -> bool {
    let chosen = tisty_core::Paths::resolve()
        .ok()
        .and_then(|paths| tisty_core::config::Config::load_or_init(&paths).ok())
        .and_then(|config| config.locale);
    tisty_core::model::spoken(chosen.as_deref())
        .to_lowercase()
        .starts_with("es")
}

pub fn behind_words(spanish: bool, itself: bool) -> (String, &'static str, &'static str) {
    let (said, how, yes, no) = if spanish {
        (
            "Una versión más nueva de Tisty actualizó tus datos.

Actualiza este Tisty para que los dos vuelvan a entenderse: abrirlos con esta versión perdería trabajo.",
            "

Esta copia la actualiza quien la instaló, no Tisty.",
            "Actualizar",
            "Cerrar",
        )
    } else {
        (
            "A newer Tisty updated your data.

Update this one so the two agree again: opening it with this version would lose work.",
            "

This copy is updated by whoever installed it, not by Tisty.",
            "Update",
            "Close",
        )
    };
    match itself {
        true => (said.to_string(), yes, no),
        false => (format!("{said}{how}"), yes, no),
    }
}

pub fn takes_itself_there() -> bool {
    update::self_installs(update::route().route) && !update::from_a_mount()
}

pub fn behind_said() -> String {
    behind_words(speaks_spanish(), takes_itself_there()).0
}

pub fn behind_buttons() -> (&'static str, &'static str) {
    let (_, yes, no) = behind_words(speaks_spanish(), takes_itself_there());
    (yes, no)
}

pub fn blamed(channel: &'static str, said: &'static str, error: tisty_core::Error) -> Refusal {
    witness::error(channel, said, &error.told());
    match error {
        tisty_core::Error::UnsupportedVersion(_) => Refusal::of("storeNewer"),
        other => Refusal::about("internalNamed", other.to_string()),
    }
}

impl Refusal {
    pub fn of(code: &'static str) -> Self {
        Self::told(code, None)
    }

    pub fn about(code: &'static str, name: impl Into<String>) -> Self {
        Self::told(code, Some(name.into()))
    }

    /// The name is left out on purpose: it carries what the person wrote, and this file is meant
    /// to be shared.
    pub fn told(code: &'static str, name: Option<String>) -> Self {
        let facts = [("code", Fact::Code(code))];
        if TELLS_OF_TROUBLE.contains(&code) {
            witness::warn(channel::WINDOW, "the window was refused", &facts);
        } else {
            witness::trace(channel::WINDOW, "the window was refused", &facts);
        }
        Self { code, name }
    }
}

const REFUSALS: &[&str] = &[
    "untitled",
    "noSuchList",
    "ambiguousList",
    "badTag",
    "notATaskId",
    "notAListId",
    "notAStepId",
    "notAnEntry",
    "notADate",
    "notAPriority",
    "notACadence",
    "emptyStep",
    "emptyEntry",
    "pastDeadline",
    "pastReminder",
    "cannotRead",
    "cannotOpen",
    "cannotWrite",
    "attachmentTooBig",
    "noRemote",
    "noMeetingPlace",
    "syncUnreadable",
    "syncRefused",
    "syncBroke",
    "wouldMerge",
    "remoteInsideStore",
    "sharedIsTheBackup",
    "otherStore",
    "restoreFailed",
    "stillCarrying",
    "sandboxCannotMerge",
    "noSuchDoc",
    "folderAway",
    "folderAwayHolds",
    "folderIsAway",
    "notAParcel",
    "parcelNewer",
    "parcelLocked",
    "wrongNumber",
    "parcelTorn",
    "noRoom",
    "nothingToCarry",
    "stillPacking",
    "aliasTooLong",
    "tooBig",
    "noSuchIcon",
    "noSuchColour",
    "noSuchFolder",
    "manyLists",
    "internal",
    "internalNamed",
];

pub(crate) fn refusal_code(said: &str) -> Option<&'static str> {
    REFUSALS.iter().copied().find(|one| *one == said)
}
