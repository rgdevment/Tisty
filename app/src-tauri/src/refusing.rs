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
    "restoredApart",
    "wouldReset",
    "syncLater",
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

pub struct Behind<'a> {
    pub spanish: bool,
    pub itself: bool,
    pub written_by: &'a str,
    pub from_the_store: bool,
}

pub fn behind_words(behind: Behind) -> (String, &'static str, &'static str) {
    let (said, by, store, how, yes, no) = if behind.spanish {
        (
            "Una versión más nueva de Tisty actualizó tus datos.

Actualiza este Tisty para que los dos vuelvan a entenderse: abrirlos con esta versión perdería trabajo.",
            "

La escribió la máquina «{name}».",
            " Este Tisty es el de Microsoft Store: si en este equipo hay otro Tisty instalado de otra forma, fue ese. Actualiza este desde la Store, o sigue con el otro.",
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

The machine «{name}» wrote it.",
            " This Tisty came from the Microsoft Store: if another Tisty installed some other way runs on this computer, that one did. Update this one from the Store, or keep using the other.",
            "

This copy is updated by whoever installed it, not by Tisty.",
            "Update",
            "Close",
        )
    };
    let mut told = said.to_string();
    if !behind.written_by.is_empty() {
        told.push_str(&by.replace("{name}", behind.written_by));
        if behind.from_the_store {
            // The sync guard turns a newer history from another computer away before it lands here.
            told.push_str(store);
        }
    }
    if !behind.itself {
        told.push_str(how);
    }
    (told, yes, no)
}

pub fn takes_itself_there() -> bool {
    update::self_installs(update::route().route) && !update::from_a_mount()
}

pub fn not_moved_words(spanish: bool, why: &str) -> String {
    let said = if spanish {
        "Tisty no pudo mover tus datos fuera de AppData, así que no abrió para no arriesgar la copia que ya tenías. Nada se perdió.

Cierra cualquier otro Tisty que esté abierto y vuelve a abrirlo. Si sigue pasando, esto es lo que respondió el equipo: "
    } else {
        "Tisty could not move your data out of AppData, so it stayed closed rather than risk the copy you already had. Nothing was lost.

Close any other Tisty that is open and start it again. If it keeps happening, this is what the computer answered: "
    };
    format!("{said}{why}")
}

pub fn behind_here(written_by: &str) -> (String, &'static str, &'static str) {
    behind_words(Behind {
        spanish: speaks_spanish(),
        itself: takes_itself_there(),
        written_by,
        from_the_store: tisty_core::paths::from_the_store(),
    })
}

pub fn blamed(channel: &'static str, said: &'static str, error: tisty_core::Error) -> Refusal {
    witness::error(channel, said, &error.told());
    match error {
        tisty_core::Error::UnsupportedVersion { .. } => Refusal::of("storeNewer"),
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
    "syncLater",
    "syncLaterToLeave",
    "noMeetingPlace",
    "syncUnreadable",
    "syncRefused",
    "syncBroke",
    "wouldMerge",
    "remoteInsideStore",
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
