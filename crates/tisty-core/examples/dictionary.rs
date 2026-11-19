use std::collections::{BTreeMap, BTreeSet, HashMap};

use sha2::{Digest, Sha256};
use tisty_core::text::folded;

const SPANISH_SHA256: &str = "9822d50707d3e3322e040a1f551996b0957d1d9c98e0a686dc04219c17656652";
const ENGLISH_SHA256: &str = "3ab4c3e4324a6d1676cf23ea3eafdd8f858f92969bf85108d049db6b7e4539bd";

struct Sense {
    synset: String,
    count: u64,
}

struct Entry {
    lemma: String,
    senses: Vec<Sense>,
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [spanish, english, out] = args.as_slice() else {
        return Err("give omw-es.xml, omw-en.xml and the table to write".into());
    };
    let spanish = pinned(spanish, SPANISH_SHA256)?;
    let english = pinned(english, ENGLISH_SHA256)?;
    let table = table(&entries(&spanish)?, &entries(&english)?);
    std::fs::write(out, &table).map_err(|e| e.to_string())?;
    println!(
        "{} words, {} bytes, sha256 {}",
        table.lines().count(),
        table.len(),
        hex(&Sha256::digest(table.as_bytes()))
    );
    Ok(())
}

fn pinned(at: &str, wanted: &str) -> Result<String, String> {
    let said = std::fs::read_to_string(at).map_err(|e| format!("{at}: {e}"))?;
    let found = hex(&Sha256::digest(said.as_bytes()));
    if found != wanted {
        return Err(format!(
            "{at} is not OMW 2.0: sha256 {found}, wanted {wanted}"
        ));
    }
    Ok(said)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|one| format!("{one:02x}")).collect()
}

fn entries(xml: &str) -> Result<Vec<Entry>, String> {
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let document =
        roxmltree::Document::parse_with_options(xml, options).map_err(|e| e.to_string())?;
    let mut all = Vec::new();
    for entry in document
        .descendants()
        .filter(|one| one.has_tag_name("LexicalEntry"))
    {
        let Some(lemma) = entry
            .children()
            .find(|one| one.has_tag_name("Lemma"))
            .and_then(|one| one.attribute("writtenForm"))
        else {
            continue;
        };
        let senses = entry
            .children()
            .filter(|one| one.has_tag_name("Sense"))
            .filter_map(|sense| {
                let synset = sense
                    .attribute("synset")?
                    .splitn(3, '-')
                    .nth(2)?
                    .to_string();
                let count = sense
                    .children()
                    .find(|one| one.has_tag_name("Count"))
                    .and_then(|one| one.text())
                    .and_then(|said| said.trim().parse().ok())
                    .unwrap_or(0);
                Some(Sense { synset, count })
            })
            .collect();
        all.push(Entry {
            lemma: lemma.to_string(),
            senses,
        });
    }
    Ok(all)
}

fn single(word: &str) -> bool {
    !word.contains([' ', '_', '-'])
}

// The pass splits text where a letter or digit ends, so a word with anything else in it is never looked up.
fn looked_up(word: &str) -> bool {
    word.chars().all(char::is_alphanumeric)
}

fn table(spanish: &[Entry], english: &[Entry]) -> String {
    let mut used: HashMap<&str, u64> = HashMap::new();
    let mut members: HashMap<&str, Vec<String>> = HashMap::new();
    for entry in english {
        for sense in &entry.senses {
            *used.entry(sense.synset.as_str()).or_default() += sense.count;
        }
        if single(&entry.lemma) {
            for sense in &entry.senses {
                members
                    .entry(sense.synset.as_str())
                    .or_default()
                    .push(entry.lemma.to_lowercase());
            }
        }
    }
    let mut synsets: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for entry in spanish.iter().filter(|one| single(&one.lemma)) {
        let held = synsets.entry(entry.lemma.to_lowercase()).or_default();
        for sense in &entry.senses {
            if !held.contains(&sense.synset.as_str()) {
                held.push(sense.synset.as_str());
            }
        }
    }
    let mut kept: BTreeMap<String, String> = BTreeMap::new();
    for (word, held) in &synsets {
        let mut score: BTreeMap<&str, u64> = BTreeMap::new();
        for synset in held {
            for member in members.get(synset).into_iter().flatten() {
                *score.entry(member.as_str()).or_default() +=
                    used.get(synset).copied().unwrap_or(0) + 1;
            }
        }
        let Some(best) = score.values().max().copied() else {
            continue;
        };
        let pivot = score
            .iter()
            .find(|(_, said)| **said == best)
            .map(|(member, _)| folded(member))
            .unwrap_or_default();
        kept.entry(folded(word)).or_insert(pivot);
    }
    let unchanged: BTreeSet<&String> = kept
        .iter()
        .filter(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect();
    kept.iter()
        .filter(|(word, _)| looked_up(word) && !unchanged.contains(word))
        .map(|(word, pivot)| format!("{word}\t{pivot}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENGLISH: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE LexicalResource SYSTEM "https://globalwordnet.github.io/schemas/WN-LMF-1.4.dtd">
<LexicalResource><Lexicon id="omw-en">
<LexicalEntry id="a"><Lemma writtenForm="pay" partOfSpeech="v"/>
<Sense id="a1" synset="omw-en-02251743-v" n="1"><Count>40</Count></Sense></LexicalEntry>
<LexicalEntry id="b"><Lemma writtenForm="give" partOfSpeech="v"/>
<Sense id="b1" synset="omw-en-02251743-v" n="2"><Count>2</Count></Sense>
<Sense id="b2" synset="omw-en-02200686-v" n="1"/></LexicalEntry>
<LexicalEntry id="c"><Lemma writtenForm="water" partOfSpeech="n"/>
<Sense id="c1" synset="omw-en-14845743-n" n="1"><Count>9</Count></Sense></LexicalEntry>
<LexicalEntry id="d"><Lemma writtenForm="chile" partOfSpeech="n"/>
<Sense id="d1" synset="omw-en-07820145-n" n="1"/></LexicalEntry>
<LexicalEntry id="e"><Lemma writtenForm="pay up" partOfSpeech="v"/>
<Sense id="e1" synset="omw-en-02200686-v" n="1"><Count>100</Count></Sense></LexicalEntry>
</Lexicon></LexicalResource>"#;

    const SPANISH: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource><Lexicon id="omw-es">
<LexicalEntry id="x"><Lemma writtenForm="pagar" partOfSpeech="v"/>
<Sense id="x1" synset="omw-es-02251743-v"/><Sense id="x2" synset="omw-es-02200686-v"/></LexicalEntry>
<LexicalEntry id="y"><Lemma writtenForm="Agua" partOfSpeech="n"/>
<Sense id="y1" synset="omw-es-14845743-n"/></LexicalEntry>
<LexicalEntry id="z"><Lemma writtenForm="chile" partOfSpeech="n"/>
<Sense id="z1" synset="omw-es-07820145-n"/></LexicalEntry>
<LexicalEntry id="w"><Lemma writtenForm="dar a luz" partOfSpeech="v"/>
<Sense id="w1" synset="omw-es-02251743-v"/></LexicalEntry>
<LexicalEntry id="v"><Lemma writtenForm="sin" partOfSpeech="n"/>
<Sense id="v1" synset="omw-es-00000000-n"/></LexicalEntry>
</Lexicon></LexicalResource>"#;

    fn made() -> String {
        table(&entries(SPANISH).unwrap(), &entries(ENGLISH).unwrap())
    }

    #[test]
    fn a_spanish_word_stands_for_the_english_word_of_its_most_used_synsets() {
        assert_eq!(made(), "agua\twater\npagar\tgive\n");
    }

    #[test]
    fn the_same_sources_make_the_same_bytes() {
        assert_eq!(made(), made());
    }

    #[test]
    fn a_wrong_file_is_refused_before_anything_is_written() {
        let at = std::env::temp_dir().join("tisty-dictionary-not-omw.xml");
        std::fs::write(&at, "<x/>").unwrap();
        assert!(pinned(at.to_str().unwrap(), SPANISH_SHA256).is_err());
    }
}
