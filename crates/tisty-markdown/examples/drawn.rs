use std::io::{Read, Write};

// The guide's build hands its pieces in as a JSON list of texts and takes each one back drawn.
fn main() {
    let mut said = String::new();
    std::io::stdin()
        .read_to_string(&mut said)
        .expect("the texts to draw");
    let texts: Vec<String> = serde_json::from_str(&said).expect("a JSON list of texts");
    let drawn: Vec<String> = texts
        .iter()
        .map(|text| tisty_markdown::html(&tisty_markdown::read(text)))
        .collect();
    std::io::stdout()
        .write_all(
            serde_json::to_string(&drawn)
                .expect("drawn pages")
                .as_bytes(),
        )
        .expect("the drawn pages written out");
}
