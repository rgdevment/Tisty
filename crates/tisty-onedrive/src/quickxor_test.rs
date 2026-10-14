use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{QuickXor, of_file};

// Taken from rclone's backend/onedrive/quickxorhash, itself checked against OneDrive: (size, input, hash), both in base64.
const VECTORS: &[(usize, &str, &str)] = &[
    (0, "", "AAAAAAAAAAAAAAAAAAAAAAAAAAA="),
    (1, "Sg==", "SgAAAAAAAAAAAAAAAQAAAAAAAAA="),
    (2, "tbQ=", "taAFAAAAAAAAAAAAAgAAAAAAAAA="),
    (3, "0pZP", "0rDEEwAAAAAAAAAAAwAAAAAAAAA="),
    (7, "qaApEj66lw==", "qQBFCiTgA11cAgAABwAAAAAAAAA="),
    (8, "/aNzzCFPS/A=", "/RjFHJgRgicsAR4ACAAAAAAAAAA="),
    (
        19,
        "g8VtAh+2Kf4k0kY5tzji2i2zmA==",
        "wDNrgwHWAVukwB8kg4YRcnALHIg=",
    ),
    (
        20,
        "T6LYJIfDh81JrAK309H2JMJTXis=",
        "zBTHrspn3mEcohlJdIUAbjGNaNg=",
    ),
    (
        21,
        "DWAAX5/CIfrmErgZa8ot6ZraeSbu",
        "LR2Z0PjuRYGKQB/mhQAuMrAGZbQ=",
    ),
    (
        40,
        "YnF4smoy9hox2jBlJ3VUa4qyCRhOZbWcmFGIiszTT4zAdYHsqJazyg==",
        "arkIn+ELddmE8N34J9ydyFKW+9w=",
    ),
    (
        111,
        "jyibxJSzO6ZiZ0O1qe3tG/bvIAYssvukh9suIT5wEy1JBINVgPiqdsTW0cOpP0aUfP7mgqLfADkz
I/m/GgCuVhr8oFLrOCoTx1/psBOWwhltCbhUx51Icm9aH8tY4Z3ccU+6BKpYQkLCy0B/A9Zc",
        "hZfLIilSITC6N3e3tQ/iSgEzkto=",
    ),
    (
        128,
        "ikwCorI7PKWz17EI50jZCGbV9JU2E8bXVfxNMg5zdmqSZ2NlsQPp0kqYIPjzwTg1MBtfWPg53k0h
0P2naJNEVgrqpoHTfV2b3pJ4m0zYPTJmUX4Bg/lOxcnCxAYKU29Y5F0U8Quz7ZXFBEweftXxJ7RS
4r6N7BzJrPsLhY7hgck=",
        "imAoFvCWlDn4yVw3/oq1PDbbm6U=",
    ),
    (
        222,
        "PfxMcUd0vIW6VbHG/uj/Y0W6qEoKmyBD0nYebEKazKaKG+UaDqBEcmQjbfQeVnVLuodMoPp7P7TR
1htX5n2VnkHh22xDyoJ8C/ZQKiSNqQfXvh83judf4RVr9exJCud8Uvgip6aVZTaPrJHVjQhMCp/d
EnGvqg0oN5OVkM2qqAXvA0teKUDhgNM71sDBVBCGXxNOR2bpbD1iM4dnuT0ey4L+loXEHTL0fqMe
UcEi2asgImnlNakwenDzz0x57aBwyq3AspCFGB1ncX4yYCr/OaCcS5OKi/00WH+wNQU3",
        "QX/YEpG0gDsmhEpCdWhsxDzsfVE=",
    ),
    (
        256,
        "qwGf2ESubE5jOUHHyc94ORczFYYbc2OmEzo+hBIyzJiNwAzC8PvJqtTzwkWkSslgHFGWQZR2BV5+
uYTrYT7HVwRM40vqfj0dBgeDENyTenIOL1LHkjtDKoXEnQ0mXAHoJ8PjbNC93zi5TovVRXTNzfGE
s5dpWVqxUzb5lc7dwkyvOluBw482mQ4xrzYyIY1t+//OrNi1ObGXuUw2jBQOFfJVj2Y6BOyYmfB1
y36eBxi3zxeG5d5NYjm2GSh6e08QMAwu3zrINcqIzLOuNIiGXBtl7DjKt7b5wqi4oFiRpZsCyx2s
mhSrdrtK/CkdU6nDN+34vSR/M8rZpWQdBE7a8g==",
        "WYT9JY3JIo/pEBp+tIM6Gt2nyTM=",
    ),
    (
        333,
        "w0LGhqU1WXFbdavqDE4kAjEzWLGGzmTNikzqnsiXHx2KRReKVTxkv27u3UcEz9+lbMvYl4xFf2Z4
aE1xRBBNd1Ke5C0zToSaYw5o4B/7X99nKK2/XaUX1byLow2aju2XJl2OpKpJg+tSJ2fmjIJTkfuY
Uz574dFX6/VXxSxwGH/xQEAKS5TCsBK3CwnuG1p5SAsQq3gGVozDWyjEBcWDMdy8/AIFrj/y03Lf
c/RNRCQTAfZbnf2QwV7sluw4fH3XJr07UoD0YqN+7XZzidtrwqMY26fpLZnyZjnBEt1FAZWO7RnK
G5asg8xRk9YaDdedXdQSJAOy6bWEWlABj+tVAigBxavaluUH8LOj+yfCFldJjNLdi90fVHkUD/m4
Mr5OtmupNMXPwuG3EQlqWUVpQoYpUYKLsk7a5Mvg6UFkiH596y5IbJEVCI1Kb3D1",
        "e3+wo77iKcILiZegnzyUNcjCdoQ=",
    ),
];

// The longer inputs are wrapped across lines as rclone keeps them.
fn decoded(input: &str) -> Vec<u8> {
    let joined: String = input.split_whitespace().collect();
    STANDARD.decode(joined).unwrap()
}

#[test]
fn the_hash_matches_what_onedrive_computes_for_every_known_input() {
    for (size, input, expected) in VECTORS {
        let input = decoded(input);
        assert_eq!(input.len(), *size);
        let mut hash = QuickXor::default();
        hash.update(&input);
        assert_eq!(hash.printed(), *expected, "{size} bytes");
    }
}

#[test]
fn bytes_fed_in_pieces_hash_as_when_fed_at_once() {
    let (_, input, expected) = VECTORS.last().unwrap();
    let input = decoded(input);
    for cut in [1, 7, 20, 64, 160, 161] {
        let mut hash = QuickXor::default();
        for piece in input.chunks(cut) {
            hash.update(piece);
        }
        assert_eq!(hash.printed(), *expected, "fed {cut} at a time");
    }
}

#[test]
fn a_file_is_hashed_as_its_bytes() {
    let (_, input, expected) = VECTORS.last().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join("one.bin");
    std::fs::write(&at, decoded(input)).unwrap();
    assert_eq!(of_file(&at).unwrap(), *expected);
}
