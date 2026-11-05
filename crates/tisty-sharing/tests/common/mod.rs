//! A real store, with a real identity, and a document written into it the way the window writes one.

use std::path::{Path, PathBuf};

use jiff::Timestamp;
use tisty_core::event::{DocAdd, Said, Tongue};
use tisty_core::model::DocId;
use tisty_core::{DeviceId, DeviceKind, Op, Paths, State, Store, attach, docs, person};
use tisty_envelope::sealing::Writer;
use tisty_share_proto::identity::{Certificate, ShareKey};
use tisty_share_proto::seed::Seed;
use tisty_sharing::{Asked, Desk, Finished, Gathered, Sealed, gathered, signer};

pub struct Home {
    pub paths: Paths,
    pub device: DeviceId,
    pub state: State,
    store: Store,
    orders: u32,
}

impl Home {
    pub fn at(folder: &Path) -> Self {
        let paths = Paths::new(folder.join("data"), folder.join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        std::fs::create_dir_all(paths.attachments()).unwrap();
        let device = tisty_core::config::Config::load_or_init(&paths)
            .unwrap()
            .device_id;
        let store = Store::open(paths.store(), device.clone()).unwrap();
        let state = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
        Self {
            paths,
            device,
            state,
            store,
            orders: 0,
        }
    }

    pub fn commit(&mut self, op: Op) {
        let event = self.store.append(op).unwrap();
        self.state.apply(&event);
    }

    pub fn with_a_name(&mut self) {
        self.commit(Op::DeviceJoin {
            d: self.device.clone(),
            k: Some(DeviceKind::Machine),
            p: None,
        });
        let seed = Seed::from_bytes(&[0x33; 32]);
        let now = "2026-11-03T12:00:00Z".parse::<Timestamp>().unwrap();
        let ops =
            person::begun(&self.paths, &self.device, &seed, &self.state.persons, now).unwrap();
        for op in ops {
            self.commit(op);
        }
    }

    pub fn wrote(&mut self, body: &str, page_of: Option<DocId>) -> DocId {
        let made = docs::create(&self.paths.docs(), &self.device, body).unwrap();
        let id = ulid::Ulid::generate();
        self.orders += 1;
        self.commit(Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id,
                order: format!("a{:04}", self.orders),
                said: Some(Said::of(body)),
                page_of,
                ..Default::default()
            },
        });
        id
    }

    pub fn joined(&mut self, id: DocId, anchor: DocId) {
        self.commit(Op::DocTongue {
            id,
            d: Tongue { of: Some(anchor) },
        });
    }

    /// Kept as the window keeps a file dropped on a document, under its name and a stamp of its bytes. It answers what its words then name.
    pub fn attached(&mut self, named: &str, bytes: &[u8]) -> String {
        let beside: PathBuf = self.paths.config().join(named);
        std::fs::create_dir_all(self.paths.config()).unwrap();
        std::fs::write(&beside, bytes).unwrap();
        let kept = attach::keep(&beside, self.paths.data(), attach::COPIED_IN_DOC).unwrap();
        self.commit(kept.told(attach::Via::Window));
        attach::leaf_of(&kept.at).unwrap().to_owned()
    }

    pub fn signer(&self) -> (ShareKey, Certificate) {
        signer(&self.paths, &self.state, &self.device).expect("this machine answers to a root")
    }

    pub fn gathered_with(
        &self,
        asked: &Asked,
        turned: &dyn Fn(&Path) -> Option<Vec<u8>>,
    ) -> Gathered {
        let desk = Desk {
            paths: &self.paths,
            state: &self.state,
            also: None,
            turned,
        };
        gathered(&desk, asked).unwrap()
    }

    pub fn gathered(&self, asked: &Asked) -> Gathered {
        self.gathered_with(asked, &|_| None)
    }

    pub fn sealed(&self, mut gathered: Gathered, answers: Vec<Finished>) -> Sealed {
        gathered.finished(answers).unwrap();
        let (key, certificate) = self.signer();
        let mut writer = Writer::new(&key, &certificate);
        gathered.sealed(&mut writer).unwrap()
    }
}

pub fn asked(doc: DocId) -> Asked {
    Asked {
        doc,
        public: false,
        cited: vec![],
        signed_as: None,
    }
}

// A real picture, as anyone would drop one: a PNG with something hidden after its end.
pub fn a_picture(wide: u32, tall: u32, seed: u8) -> Vec<u8> {
    let mut pixels = image::RgbImage::new(wide, tall);
    for (at, pixel) in pixels.pixels_mut().enumerate() {
        *pixel = image::Rgb([seed, at as u8, 7]);
    }
    let mut out = std::io::Cursor::new(Vec::new());
    pixels.write_to(&mut out, image::ImageFormat::Png).unwrap();
    let mut out = out.into_inner();
    out.extend_from_slice(b"HIDDEN-AFTER-ITS-END");
    out
}
