use tisty_contracts::{Cited, Document, Sealing, Translation};

use crate::sealing::{Refused, Sealed, Writer};

/// One language of a document, ready to seal.
#[derive(Debug, Clone)]
pub struct Tongue {
    pub document: Document,
    pub pictures: Vec<Vec<u8>>,
    pub files: Vec<Vec<u8>>,
    pub sealing: Sealing,
}

/// A document and its other languages. Only the anchor holds the map, so a reader reaches each language from it.
#[derive(Debug, Clone)]
pub struct Family {
    pub anchor: Tongue,
    pub others: Vec<Tongue>,
}

/// Each envelope that has to go, under the id it asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Going {
    pub id: String,
    pub sealed: Sealed,
}

fn translation_of(tongue: &Tongue) -> Result<Translation, Refused> {
    let lang = tongue.document.lang.clone();
    Ok(Translation {
        lang: lang.ok_or(Refused::DoesNotAdd(
            "a language of a family does not say which it is",
        ))?,
        document: tongue.sealing.id.clone(),
        title: tongue.document.title.clone(),
        key: tongue.sealing.link_key.clone(),
    })
}

/// Seals a family. Each language goes in its own envelopes, signed here, and the anchor carries the map to them.
pub fn sealed_family(writer: &mut Writer, family: Family) -> Result<Vec<Going>, Refused> {
    let Family { mut anchor, others } = family;
    if !others.is_empty() {
        if anchor.document.lang.is_none() {
            return Err(Refused::DoesNotAdd(
                "the anchor of a family does not say its language",
            ));
        }
        anchor.document.languages = Some(
            others
                .iter()
                .map(translation_of)
                .collect::<Result<_, _>>()?,
        );
    }
    let mut going = Vec::with_capacity(others.len() + 1);
    for tongue in std::iter::once(anchor).chain(others) {
        let id = tongue.sealing.id.clone();
        let sealed = writer.sealed(
            tongue.document,
            tongue.pictures,
            tongue.files,
            &tongue.sealing,
        )?;
        going.push(Going { id, sealed });
    }
    Ok(going)
}

/// Seals a document with what it cites. Each cited family keeps its own envelopes and its key goes in `cited`, in the order the HTML counts them.
pub fn sealed_with_cited(
    writer: &mut Writer,
    mut family: Family,
    cited: Vec<Family>,
) -> Result<Vec<Going>, Refused> {
    let mut going = Vec::new();
    let mut said = Vec::with_capacity(cited.len());
    for one in cited {
        let key = one
            .anchor
            .sealing
            .link_key
            .clone()
            .ok_or(Refused::DoesNotAdd(
                "what is cited has to say the key that opens it",
            ))?;
        said.push(Cited {
            document: one.anchor.sealing.id.clone(),
            key,
            title: one.anchor.document.title.clone(),
        });
        going.extend(sealed_family(writer, one)?);
    }
    if !said.is_empty() {
        family.anchor.document.cited = Some(said);
    }
    going.splice(0..0, sealed_family(writer, family)?);
    Ok(going)
}
