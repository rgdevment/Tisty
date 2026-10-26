#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_sync::{
    Sealing, Turn, claim_in, claimed, claimed_by, closed, dropped, needs_later, phase, sealed_in,
    sealing_in, stirred, stirred_in, switched,
};

const DEVICE: &str = "01JB8Z5W3K9QH2V6X4T7N0RMCD";
const CLOUD: &str = "Tisty";

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    text: &'a str,
    on: bool,
    nonce: u64,
    sealed: bool,
    at: u64,
}

fuzz_target!(|input: Input| {
    let text = input.text;
    if claim_in(text).is_some() {
        assert!(matches!(
            sealing_in(text),
            Sealing::Copying(_) | Sealing::Closed
        ));
    }
    let _ = (sealed_in(text), stirred_in(text), needs_later(text));

    let turn = if input.on { Turn::On } else { Turn::Off };
    let nonce = format!("{:016x}", input.nonce);
    let claim = format!("{DEVICE}:{nonce}");
    let Some(taken) = claimed(text, &phase(turn, DEVICE, &nonce)) else {
        return;
    };
    assert_eq!(sealing_in(&taken), Sealing::Copying(turn));
    assert_eq!(claim_in(&taken).as_deref(), Some(claim.as_str()));
    assert!(claimed_by(&taken, &claim));
    assert_eq!(stirred_in(&taken), None);
    if let Some(stirring) = stirred(&taken, &claim, input.at) {
        assert_eq!(stirred_in(&stirring), Some(input.at));
    }

    let left = dropped(&taken, &claim).expect("dropped");
    assert_eq!(claim_in(&left), None);
    let shut = closed(&taken, &claim).expect("closed");
    assert_eq!(sealing_in(&shut), Sealing::Closed);
    let sealed = input.sealed.then_some(CLOUD);
    let done = switched(&shut, &claim, sealed).expect("switched");
    assert_eq!(claim_in(&done), None);
    assert_eq!(sealed_in(&done).as_deref(), sealed);
});
