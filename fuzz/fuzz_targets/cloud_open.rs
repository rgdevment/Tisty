#![no_main]

use std::io::{ErrorKind, Read};

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::{self, BLOCK, CloudKey, HEAD, LEAD, TAG};

const CLOUD: &str = "Tisty";
const NAME: &str = "store/device/events.jsonl";

#[derive(Arbitrary, Debug)]
enum Handed {
    Sealed { plain: Vec<u8>, blocks: u8 },
    Anything(Vec<u8>),
}

#[derive(Arbitrary, Debug)]
struct Input {
    handed: Handed,
    step: u16,
    stalls: bool,
}

struct Trickle<'a> {
    left: &'a [u8],
    step: usize,
    stalls: bool,
    stalled: bool,
}

impl Read for Trickle<'_> {
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        if self.stalls && !self.stalled {
            self.stalled = true;
            return Err(ErrorKind::Interrupted.into());
        }
        self.stalled = false;
        let given = self.step.min(into.len()).min(self.left.len());
        into[..given].copy_from_slice(&self.left[..given]);
        self.left = &self.left[given..];
        Ok(given)
    }
}

fuzz_target!(|input: Input| {
    let key = CloudKey::from_bytes(&[7; 32]);
    let (sealed, plain) = match input.handed {
        Handed::Sealed { plain, blocks } => {
            let wide = plain.len() + usize::from(blocks % 3) * BLOCK;
            let plain: Vec<u8> = plain
                .into_iter()
                .chain(std::iter::repeat(0))
                .take(wide)
                .collect();
            let sealed = cloud::sealed(&key, CLOUD, NAME, &plain).expect("sealed");
            (sealed, Some(plain))
        }
        Handed::Anything(bytes) => (bytes, None),
    };

    let opened = cloud::opened(&key, CLOUD, NAME, &sealed);
    let mut trickled = Trickle {
        left: &sealed,
        step: usize::from(input.step.max(1)),
        stalls: input.stalls,
        stalled: false,
    };
    let mut streamed = Vec::new();
    let open = cloud::open(&key, CLOUD, NAME, &mut trickled, &mut streamed);
    assert_eq!(opened.as_ref().err(), open.as_ref().err());
    if let Some(plain) = plain {
        assert_eq!(opened.as_deref(), Ok(&plain));
    }
    if let Ok(plain) = opened {
        assert!(cloud::is_sealed(&sealed));
        let blocks = plain.len().div_ceil(BLOCK).max(1);
        assert_eq!(sealed.len(), LEAD + HEAD + plain.len() + blocks * TAG);
        assert_eq!(*plain, streamed);
    }
});
