#![no_main]

use libfuzzer_sys::fuzz_target;
use tisty_share_proto::envelope::{self, Envelope, LinkKey};

// Whatever a stranger uploads reaches this before anything is stored, so it may refuse but never fall over.
fuzz_target!(|bytes: &[u8]| {
    let Ok(read) = Envelope::read(bytes) else {
        return;
    };
    assert!(read.manifest().holds().is_ok());
    let again = Envelope::read(&read.to_bytes()).expect("what was read reads again");
    assert_eq!(again.manifest(), read.manifest());
    assert_eq!(again.digest(), read.digest());
    let _ = read.sender(&[]);
    let _ = envelope::open(&LinkKey::from_bytes(&[7; 32]), &read, &[]);
});
