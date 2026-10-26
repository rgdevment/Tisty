#![no_main]

use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::{self, CloudKey, HEAD};

const CLOUD: &str = "Tisty";
const NAME: &str = "store/device/events.jsonl";

fuzz_target!(|sealed: &[u8]| {
    let key = CloudKey::from_bytes(&[7; 32]);
    let opened = cloud::opened(&key, CLOUD, NAME, sealed);
    let mut streamed = Vec::new();
    let open = cloud::open(&key, CLOUD, NAME, &mut &sealed[..], &mut streamed);
    assert_eq!(opened.as_ref().err(), open.as_ref().err());
    if let Ok(plain) = opened {
        assert!(cloud::is_sealed(sealed));
        assert!(plain.len() + 5 + HEAD + 16 <= sealed.len());
        assert_eq!(*plain, streamed);
    }
});
