#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::handing::SigningKey;
use tisty_share_proto::notice::Unsealed;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    json: &'a [u8],
    cloud: String,
    named: String,
    by: String,
    at: i64,
    field: u8,
    text: String,
    number: i64,
}

fn again(notice: &Unsealed) -> Unsealed {
    let written = serde_json::to_vec(notice).expect("written");
    serde_json::from_slice(&written).expect("read back")
}

fuzz_target!(|input: Input| {
    if let Ok(read) = serde_json::from_slice::<Unsealed>(input.json) {
        let _ = read.key();
        assert_eq!(again(&read).holds(), read.holds());
    }

    let signer = SigningKey::from_bytes(&[1; 32]);
    let notice = Unsealed::signed(&input.cloud, &input.named, &input.by, &signer, input.at);
    assert!(notice.holds());
    assert!(again(&notice).holds());
    let mut changed = notice.clone();
    match input.field % 7 {
        0 => changed.v = input.number as u32,
        1 => changed.cloud = input.text,
        2 => changed.named = input.text,
        3 => changed.by = input.text,
        4 => changed.key = input.text,
        5 => changed.at = input.number,
        _ => changed.sig = input.text,
    }
    if changed != notice {
        assert!(!changed.holds());
        assert!(!again(&changed).holds());
    }
});
