use std::path::Path;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    Error, Result,
    event::DeviceId,
    paths::Paths,
    store,
    witness::{self, Fact, channel},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "how", content = "at")]
pub enum Sync {
    Local,
    Folder(std::path::PathBuf),
}

impl Config {
    pub fn muted(&self) -> &[String] {
        self.quiet.as_deref().unwrap_or_default()
    }

    pub fn copies_up_to(&self) -> u64 {
        self.attach_up_to
            .unwrap_or(crate::attach::COPIED_UP_TO)
            .clamp(crate::attach::COPIED_LEAST, crate::attach::COPIED_MOST)
    }

    pub fn copies_in_a_doc(&self) -> u64 {
        crate::attach::COPIED_IN_DOC
    }

    pub fn backs_up(&self) -> bool {
        !matches!(self.sync, Some(Sync::Folder(_)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Closing {
    Hide,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub device_id: DeviceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_close: Option<Closing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backed_up_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<Sync>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_up_to: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide: Option<String>,
}

impl Config {
    pub fn load_or_init(paths: &Paths) -> Result<Self> {
        if let Some(existing) = Self::load(&paths.config_file())? {
            return Ok(existing);
        }

        let config = Self {
            device_id: DeviceId(new_device_id()),
            locale: None,
            editor: None,
            quiet: None,
            checked_at: None,
            attach_up_to: Some(crate::attach::COPIED_AT_FIRST),
            opened_by: None,
            on_close: None,
            backed_up_at: None,
            sync: None,
            synced_at: None,
            heard_at: None,
            guide: None,
        };
        config.save(paths)?;
        Ok(config)
    }

    pub fn load(file: &Path) -> Result<Option<Self>> {
        match std::fs::read_to_string(file) {
            Ok(text) => Ok(Some(toml::from_str(&text)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        std::fs::create_dir_all(paths.config())?;
        if let Err(e) = crate::paths::ours_alone(paths.config()) {
            witness::warn(
                channel::CONFIG,
                "config folder not made private",
                &[
                    ("at", Fact::Path(paths.config().to_path_buf())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
        store::write_atomic(
            &paths.config_file(),
            toml::to_string_pretty(self)?.as_bytes(),
        )
    }
}

const TREES: [&str; 64] = [
    "abeto",
    "acacia",
    "alamo",
    "alerce",
    "algarrobo",
    "aliso",
    "almendro",
    "arce",
    "arrayan",
    "avellano",
    "azahar",
    "boj",
    "brezo",
    "cactus",
    "canelo",
    "carrasco",
    "castano",
    "cedro",
    "cerezo",
    "cipres",
    "ciruelo",
    "coihue",
    "drago",
    "encina",
    "enebro",
    "espino",
    "eucalipto",
    "fresno",
    "ginkgo",
    "granado",
    "haya",
    "helecho",
    "hiedra",
    "higuera",
    "jacaranda",
    "jazmin",
    "laurel",
    "lavanda",
    "lentisco",
    "lila",
    "madrono",
    "magnolio",
    "manzano",
    "membrillo",
    "menta",
    "mirto",
    "moral",
    "musgo",
    "nogal",
    "olivo",
    "olmo",
    "orquidea",
    "palmera",
    "peral",
    "pino",
    "quillay",
    "roble",
    "romero",
    "salvia",
    "sauce",
    "tejo",
    "tomillo",
    "trebol",
    "yuca",
];

pub fn nicknamed(device: &str) -> String {
    use sha2::{Digest, Sha256};
    let said = Sha256::digest(device.as_bytes());
    let word = TREES[(said[0] as usize) % TREES.len()];
    let number = (u16::from(said[1]) * 100 / 256) + 1;
    format!("{word} {number}")
}

pub fn new_device_id() -> String {
    let ulid = Ulid::generate().to_string().to_lowercase();
    format!("dev_{}", &ulid[ulid.len() - 8..])
}

#[cfg(test)]
mod tests {

    #[test]
    fn the_same_machine_is_called_the_same_thing_on_every_machine_that_asks() {
        for (id, called) in [
            ("dev_a657da33", "carrasco 75"),
            ("dev_jtntzhbx", "acacia 1"),
            ("dev_ej8mf31b", "roble 60"),
        ] {
            assert_eq!(
                nicknamed(id),
                called,
                "dos maquinas dejarian de llamar igual a la misma"
            );
        }
    }

    #[test]
    fn the_whole_dictionary_is_used_and_not_a_corner_of_it() {
        let said: std::collections::BTreeSet<String> = (0..60_000)
            .map(|n| nicknamed(&format!("dev_{n:08x}")))
            .collect();

        assert!(
            said.len() > 6_000,
            "solo {} nombres de los 6400 posibles",
            said.len()
        );
    }

    #[test]
    fn a_handful_of_machines_can_be_told_apart_by_name_alone() {
        let mut twice = 0;
        for round in 0..500 {
            let names: std::collections::BTreeSet<String> = (0..5)
                .map(|n| nicknamed(&format!("dev_{round:04x}{n:04x}")))
                .collect();
            if names.len() < 5 {
                twice += 1;
            }
        }

        assert!(twice < 15, "{twice} de 500 flotas con dos nombres iguales");
    }

    #[test]
    fn a_nickname_is_a_word_and_a_number_that_can_be_said_out_loud() {
        for n in 0..500 {
            let said = nicknamed(&format!("dev_{n:08x}"));
            let (word, number) = said.split_once(' ').expect("una palabra y un numero");
            assert!(TREES.contains(&word), "{word} no esta en el diccionario");
            let number: u16 = number.parse().expect("un numero");
            assert!((1..=100).contains(&number), "{number} fuera de rango");
            assert!(word.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn the_nickname_never_carries_anything_of_the_identifier_it_came_from() {
        let said = nicknamed("dev_a657da33");

        assert!(!said.contains("a657"));
        assert!(!said.contains("da33"));
        assert!(!said.contains("dev"));
    }
    use super::*;

    fn paths(tmp: &tempfile::TempDir) -> Paths {
        Paths::new(tmp.path().join("data"), tmp.path().join("config"))
    }

    #[test]
    fn the_device_id_is_generated_once_and_reused() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);

        let first = Config::load_or_init(&p).unwrap();
        let second = Config::load_or_init(&p).unwrap();

        assert_eq!(first.device_id, second.device_id);
    }

    #[test]
    fn the_guide_it_wrote_is_remembered_so_a_second_one_is_never_written() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);

        let mut first = Config::load_or_init(&p).unwrap();
        assert_eq!(first.guide, None, "no hay guia antes de escribirla");
        first.guide = Some("mac0-0001".into());
        first.save(&p).unwrap();

        let again = Config::load_or_init(&p).unwrap();

        assert_eq!(again.guide.as_deref(), Some("mac0-0001"));
    }

    #[test]
    fn a_settings_file_written_before_the_guide_existed_still_reads() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);
        std::fs::create_dir_all(p.config()).unwrap();
        std::fs::write(p.config_file(), "device_id = \"dev_a3f10000\"\n").unwrap();

        let kept = Config::load(&p.config_file()).unwrap().unwrap();

        assert_eq!(kept.guide, None);
    }

    #[test]
    fn two_installs_never_share_a_device_id() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();

        assert_ne!(
            Config::load_or_init(&paths(&a)).unwrap().device_id,
            Config::load_or_init(&paths(&b)).unwrap().device_id
        );
    }

    #[test]
    fn the_config_file_is_written_outside_the_synced_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);
        Config::load_or_init(&p).unwrap();

        assert!(p.config_file().exists());
        assert!(!p.config_file().starts_with(p.data()));
    }

    #[test]
    fn round_trips_through_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);

        let mut config = Config::load_or_init(&p).unwrap();
        config.locale = Some("es".into());
        config.editor = Some("hx".into());
        config.save(&p).unwrap();

        assert_eq!(Config::load(&p.config_file()).unwrap().unwrap(), config);
    }

    #[test]
    fn a_missing_config_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(
            Config::load(&tmp.path().join("absent.toml"))
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn a_table_valued_field_does_not_swallow_what_follows_it() {
        let config = Config {
            device_id: DeviceId("dev_a".into()),
            locale: Some("es".into()),
            editor: None,
            opened_by: Some("0.1.0".into()),
            on_close: Some(Closing::Hide),
            backed_up_at: None,
            sync: Some(Sync::Folder("G:/Mi unidad/Tisty".into())),
            synced_at: None,
            heard_at: None,
            quiet: None,
            checked_at: None,
            attach_up_to: None,
            guide: Some("mac0-0001".into()),
        };

        let written = toml::to_string_pretty(&config).unwrap();
        let read: Config = toml::from_str(&written).unwrap();
        assert_eq!(
            read, config,
            "round trip lost something:
{written}"
        );
    }

    mod ceilings {
        use super::*;

        fn bare() -> Config {
            Config {
                device_id: DeviceId(new_device_id()),
                locale: None,
                editor: None,
                quiet: None,
                checked_at: None,
                attach_up_to: None,
                opened_by: None,
                on_close: None,
                backed_up_at: None,
                sync: None,
                synced_at: None,
                heard_at: None,
                guide: None,
            }
        }

        #[test]
        fn a_task_never_takes_a_file_past_what_the_product_promises() {
            let mut config = bare();
            config.attach_up_to = Some(200 * 1024 * 1024);

            assert_eq!(
                config.copies_up_to(),
                crate::attach::COPIED_UP_TO,
                "a setting talked the ceiling above what a task is meant to hold"
            );
        }

        #[test]
        fn a_store_that_never_chose_keeps_the_ceiling_it_always_had() {
            assert_eq!(bare().copies_up_to(), crate::attach::COPIED_UP_TO);
        }

        #[test]
        fn a_machine_starting_today_asks_for_less_until_it_says_otherwise() {
            let room = tempfile::tempdir().unwrap();
            let paths = Paths::new(room.path().join("data"), room.path().join("config"));

            let made = Config::load_or_init(&paths).unwrap();

            assert_eq!(made.copies_up_to(), crate::attach::COPIED_AT_FIRST);
            assert!(made.copies_up_to() < crate::attach::COPIED_UP_TO);
        }

        #[test]
        fn a_document_still_holds_five_hundred_megabytes_whatever_a_task_is_set_to() {
            let mut config = bare();
            config.attach_up_to = Some(crate::attach::COPIED_AT_FIRST);

            assert_eq!(config.copies_in_a_doc(), 500 * 1024 * 1024);
            assert!(config.copies_in_a_doc() > config.copies_up_to());
        }
    }
}
