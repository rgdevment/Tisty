use std::path::{Path, PathBuf};

use crate::event::{DeviceId, Event, Op};
use crate::signing::{self, SigningKey};

pub(crate) struct Sixteen {
    pub dir: PathBuf,
    pub who: DeviceId,
    key: SigningKey,
    seq: u64,
}

const ACTIVE: &str = "active.tisty";

impl Sixteen {
    pub fn at(dir: &Path, who: &DeviceId, key: SigningKey) -> Self {
        std::fs::create_dir_all(dir).unwrap();
        Self {
            dir: dir.to_path_buf(),
            who: who.clone(),
            key,
            seq: 0,
        }
    }

    pub fn signed(&mut self, ops: impl IntoIterator<Item = Op>) -> &mut Self {
        self.lines(16, ops);
        let covers = self.covers(&self.dir.join(ACTIVE));
        self.sign(ACTIVE, &covers);
        self
    }

    pub fn unsigned(&mut self, ops: impl IntoIterator<Item = Op>) -> &mut Self {
        self.lines(15, ops);
        self
    }

    pub fn closed(&mut self) -> String {
        let number = crate::store::segments_in(&self.dir)
            .unwrap()
            .iter()
            .filter_map(|one| one.file_stem()?.to_str()?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        let named = format!("{number:06}.tisty");
        let at = self.dir.join(&named);
        std::fs::rename(self.dir.join(ACTIVE), &at).unwrap();
        let had = self.dir.join(ACTIVE).with_extension(signing::SIG).exists();
        let _ = std::fs::remove_file(self.dir.join(ACTIVE).with_extension(signing::SIG));
        if had {
            let covers = self.covers(&at);
            self.sign(&named, &covers);
        }
        let lines = std::fs::read_to_string(&at).unwrap().lines().count();
        std::fs::write(at.with_extension("count"), lines.to_string()).unwrap();
        named
    }

    fn lines(&mut self, v: u32, ops: impl IntoIterator<Item = Op>) {
        use std::io::Write;

        let mut said = String::new();
        for op in ops {
            self.seq += 1;
            let at = jiff::Timestamp::from_second(1_700_000_000 + self.seq as i64).unwrap();
            let mut event = Event::new(self.who.clone(), at, op);
            event.version = v;
            said.push_str(&serde_json::to_string(&event).unwrap());
            said.push('\n');
        }
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(ACTIVE))
            .unwrap()
            .write_all(said.as_bytes())
            .unwrap();
    }

    fn covers(&self, upto: &Path) -> signing::Covers {
        let mut tip = signing::NOTHING_BEFORE;
        for one in crate::store::segments_in(&self.dir).unwrap() {
            tip = signing::tip_of(tip, &std::fs::read(&one).unwrap());
            if one == upto {
                break;
            }
        }
        signing::Covers {
            tip,
            at: std::fs::metadata(upto).unwrap().len(),
        }
    }

    fn sign(&self, named: &str, covers: &signing::Covers) {
        let about = signing::About {
            device: &self.who.0,
            segment: named,
        };
        std::fs::write(
            self.dir.join(named).with_extension(signing::SIG),
            signing::signed(&self.key, &about, covers),
        )
        .unwrap();
    }
}
