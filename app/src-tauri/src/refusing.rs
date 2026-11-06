use tisty_core::witness::{self, Fact, channel};

use crate::update;

/// A refusal the person could not have caused by asking for something ordinary: it says the store
/// and the window disagree, and that is worth a line in a log somebody will send us.
const TELLS_OF_TROUBLE: &[&str] = &[
    "noSuchDoc",
    "noSuchTask",
    "deleteRefused",
    "storeNewer",
    "otherStore",
    "restoredApart",
    "wouldReset",
    "syncLater",
    "settingsUnreadable",
];

pub const RELEASES: &str = "https://github.com/rgdevment/Tisty/releases/latest";

/// The releases page hands a copy kept by the Store an installer that would settle beside the
/// package instead of replacing it, and leave the person with two Tistys.
const IN_THE_STORE: &str = "https://apps.microsoft.com/detail/9PGVWXD8X93N";

#[derive(Debug, Clone, serde::Serialize)]
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

pub fn page(spanish: bool) -> &'static str {
    match spanish {
        true => "index.html?locale=es",
        false => "index.html?locale=en",
    }
}

pub fn light_page(spanish: bool) -> &'static str {
    match spanish {
        true => "viewer/index.html?locale=es",
        false => "viewer/index.html?locale=en",
    }
}

pub fn speaks_spanish() -> bool {
    let chosen = tisty_core::Paths::resolve()
        .ok()
        .and_then(|paths| tisty_core::config::Config::load_or_init(&paths).ok())
        .and_then(|config| config.locale);
    spanish(chosen.as_deref())
}

pub fn spanish(locale: Option<&str>) -> bool {
    tisty_core::model::spoken(locale)
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

pub fn edit_look_words(spanish: bool) -> (&'static str, &'static str, &'static str) {
    if spanish {
        (
            "Llegó a Tisty un link a un documento compartido. Para mostrarlo, Tisty tiene que pedírselo a Folio, que verá la dirección de red de este equipo. Si no lo esperabas, elige «Ahora no».",
            "Buscarlo",
            "Ahora no",
        )
    } else {
        (
            "A link to a shared document reached Tisty. To show it, Tisty has to ask Folio for it, and Folio will see this computer's network address. If you were not expecting it, choose Not now.",
            "Look for it",
            "Not now",
        )
    }
}

pub fn edit_asked_words(
    spanish: bool,
    title: &str,
    name: &str,
    print: &str,
    doubtful: bool,
) -> (String, &'static str, &'static str) {
    if spanish {
        let doubt = if doubtful {
            " Ojo: esa llave fue retirada o no se puede confirmar, así que no lo des por suyo sin preguntar."
        } else {
            ""
        };
        (
            format!(
                "¿Traer «{title}» a tu Tisty? Lo firmó {name}, con la huella {print}. Compárala con quien te lo compartió.{doubt}"
            ),
            "Traer",
            "Ahora no",
        )
    } else {
        let doubt = if doubtful {
            " Careful: that key was taken back or cannot be confirmed, so do not take it as theirs without asking."
        } else {
            ""
        };
        (
            format!(
                "Bring «{title}» into your Tisty? It was signed by {name}, with the fingerprint {print}. Compare it with whoever shared it.{doubt}"
            ),
            "Bring it",
            "Not now",
        )
    }
}

pub fn edit_failed_words(spanish: bool, code: &str) -> &'static str {
    match (code, spanish) {
        ("folioGone", true) => "Ese link ya no existe: caducó o se dejó de compartir.",
        ("folioGone", false) => "That link no longer exists: it ran out or it was stopped.",
        ("folioBlocked", true) => "Folio dejó de servir ese documento por denuncias.",
        ("folioBlocked", false) => "Folio stopped serving that document after reports.",
        ("folioPaused", true) => "Quien lo compartió lo dejó en pausa, así que por ahora no abre.",
        ("folioPaused", false) => "Whoever shared it paused it, so it does not open for now.",
        ("folioOutdated", true) => {
            "Este Tisty es demasiado antiguo para ese documento. Actualízalo."
        }
        ("folioOutdated", false) => "This Tisty is too old for that document. Update it.",
        ("folioUnopened", true) => {
            "No se pudo abrir ese documento: su firma o su contenido no calzan."
        }
        ("folioUnopened", false) => {
            "That document could not be opened: its signature or its contents do not hold."
        }
        (_, true) => "No se pudo traer ese documento. Revisa tu conexión, o léelo en el navegador.",
        (_, false) => {
            "That document could not be brought in. Check your connection, or read it in your browser."
        }
    }
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
    "notADate",
    "notAPriority",
    "notACadence",
    "pastDeadline",
    "pastReminder",
    "cannotRead",
    "cannotOpen",
    "cannotWrite",
    "attachmentTooBig",
    "noRemote",
    "syncLater",
    "syncLaterToLeave",
    "settingsUnreadable",
    "noMeetingPlace",
    "syncUnreadable",
    "syncRefused",
    "syncBroke",
    "syncFailed",
    "syncStuckDoc",
    "syncStuckHistory",
    "cloudEmptied",
    "cloudFull",
    "cloudLost",
    "cloudElsewhere",
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
    "nameTooLong",
    "nameUnfit",
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
