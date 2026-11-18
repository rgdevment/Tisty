use std::io::{Read, Write};

use tisty_markdown::cards::{DocCard, doc_card, words};

#[derive(serde::Deserialize)]
struct Asked {
    spoken: String,
    cards: Vec<Named>,
}

#[derive(serde::Deserialize)]
struct Named {
    id: String,
    words: String,
}

// The guide's build hands in the document cards its examples cite and takes each one back drawn.
fn main() {
    let mut said = String::new();
    std::io::stdin()
        .read_to_string(&mut said)
        .expect("the cards to draw");
    let asked: Asked = serde_json::from_str(&said).expect("a language and a JSON list of cards");
    let words = words(&asked.spoken);
    let drawn: Vec<String> = asked
        .cards
        .iter()
        .map(|one| doc_card(&DocCard::example(&one.id, &one.words), words))
        .collect();
    std::io::stdout()
        .write_all(
            serde_json::to_string(&drawn)
                .expect("drawn cards")
                .as_bytes(),
        )
        .expect("the drawn cards written out");
}
