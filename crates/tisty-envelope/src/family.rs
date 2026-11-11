use std::collections::BTreeSet;
use std::sync::Arc;

use tisty_contracts::{Cited, Document, Sealing, Translation};

use crate::reviews::LeftOut;
use crate::sealing::{Refused, Sealed, Writer};

#[derive(Debug, Clone, PartialEq)]
pub struct Tongue {
    pub document: Document,
    pub pictures: Vec<Vec<u8>>,
    pub files: Vec<Vec<u8>>,
    pub sealing: Sealing,
    // Shared, since two languages may cite one document and its bytes need not be held twice.
    pub cited: Vec<Arc<Family>>,
}

// Only the anchor holds the map, so a reader reaches each language from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Family {
    pub anchor: Tongue,
    pub others: Vec<Tongue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Going {
    pub id: String,
    pub sealed: Sealed,
}

const OTHER_LANGUAGES_AT_MOST: usize = 19;

fn translation_of(
    tongue: &Tongue,
    anchor_token: Option<&str>,
    pending: bool,
) -> Result<Translation, Refused> {
    let lang = tongue.document.lang.clone();
    let edit = match (anchor_token, tongue.sealing.token.as_deref()) {
        (Some(anchor), Some(token)) => Some(crate::tokens::sealed_for(
            anchor,
            &tongue.sealing.id,
            token,
        )?),
        _ => None,
    };
    Ok(Translation {
        lang: lang.ok_or(Refused::DoesNotAdd(
            "a language of a family does not say which it is",
        ))?,
        document: tongue.sealing.id.clone(),
        title: tongue.document.title.clone(),
        key: tongue.sealing.link_key.clone(),
        edit,
        pending: pending.then_some(true),
    })
}

fn languages_add(anchor: &Tongue, others: &[Tongue]) -> Result<(), Refused> {
    if anchor.document.lang.is_none() {
        return Err(Refused::DoesNotAdd(
            "the anchor of a family does not say its language",
        ));
    }
    let mut seen = BTreeSet::from([anchor.document.lang.clone()]);
    let mut ids = BTreeSet::from([&anchor.sealing.id]);
    for tongue in others {
        if !ids.insert(&tongue.sealing.id) {
            return Err(Refused::DoesNotAdd(
                "two languages of a family go under one id",
            ));
        }
        if !seen.insert(tongue.document.lang.clone()) {
            return Err(Refused::DoesNotAdd("a family says one language twice"));
        }
        // Everything public can be read by anyone, so a key in it would open what it was meant to keep.
        if anchor.sealing.link_key.is_none() && tongue.sealing.link_key.is_some() {
            return Err(Refused::DoesNotAdd(
                "what is public cannot hold the key of a private language",
            ));
        }
    }
    Ok(())
}

fn cited_by(tongue: &Tongue) -> Result<Vec<Cited>, Refused> {
    let mut seen = BTreeSet::new();
    let mut said = Vec::with_capacity(tongue.cited.len());
    for one in &tongue.cited {
        let id = &one.anchor.sealing.id;
        if !seen.insert(id) {
            return Err(Refused::DoesNotAdd("a document cites another one twice"));
        }
        if std::iter::once(&one.anchor)
            .chain(&one.others)
            .any(|tongue| !tongue.cited.is_empty())
        {
            return Err(Refused::DoesNotAdd(
                "what is cited is held by the document itself, and cites nothing further",
            ));
        }
        let key = one.anchor.sealing.link_key.clone();
        if tongue.sealing.link_key.is_none() && key.is_some() {
            return Err(Refused::DoesNotAdd(
                "what is public cannot hold the key of a private document",
            ));
        }
        said.push(Cited {
            document: id.clone(),
            key,
            title: one.anchor.document.title.clone(),
        });
    }
    Ok(said)
}

pub struct Mine<'a> {
    pub id: Option<&'a str>,
    pub lang: Option<&'a str>,
}

/// What stays of the map as it arrived. Only the owner's own language of one lang takes the place of an entry of that lang: someone who edits never takes another's.
pub fn untouched(
    held: Vec<Translation>,
    anchor_lang: Option<&str>,
    mine: &[Mine],
    owner_writes: bool,
) -> Vec<Translation> {
    let replaces = |tongue: &Mine, one: &Translation| {
        tongue.id == Some(one.document.as_str())
            || (owner_writes && one.pending != Some(true) && tongue.lang == Some(one.lang.as_str()))
    };
    held.into_iter()
        .filter(|one| anchor_lang != Some(one.lang.as_str()))
        .filter(|one| !mine.iter().any(|tongue| replaces(tongue, one)))
        .collect()
}

fn languages_sealed(
    writer: &mut Writer,
    family: Family,
    going: &mut Vec<Going>,
) -> Result<Vec<Arc<Family>>, Refused> {
    let Family {
        mut anchor,
        mut others,
    } = family;
    if !others.is_empty() {
        languages_add(&anchor, &others)?;
        let owner = anchor
            .sealing
            .owner
            .as_ref()
            .map_or_else(|| writer.root().to_owned(), |one| one.root.clone());
        let anothers = owner != writer.root();
        let mine: Vec<Mine> = others
            .iter()
            .map(|tongue| Mine {
                id: Some(&tongue.sealing.id),
                lang: tongue.document.lang.as_deref(),
            })
            .collect();
        let held = anchor.document.languages.take().unwrap_or_default();
        let untouched = untouched(held, anchor.document.lang.as_deref(), &mine, !anothers);
        let mut map = others
            .iter()
            .map(|one| {
                let pending = anothers && one.sealing.owner.is_none();
                translation_of(one, anchor.sealing.token.as_deref(), pending)
            })
            .collect::<Result<Vec<_>, _>>()?;
        map.extend(untouched);
        let reviews = anchor.document.reviews.as_deref().unwrap_or_default();
        let anchor_key = anchor.sealing.link_key.as_deref();
        let gone = anchor_key.and_then(|key| LeftOut::by(key, &anchor.sealing.id, &owner, reviews));
        if let Some(gone) = gone {
            map.retain(|one| !gone.names(&one.document));
        }
        if map.len() > OTHER_LANGUAGES_AT_MOST {
            return Err(Refused::DoesNotAdd(
                "a map holds at most 19 languages besides its anchor, and none is taken out to make room",
            ));
        }
        anchor.document.languages = Some(map);
        // Paper, sheets and orientation are the group's, and each language says which anchor it belongs to.
        for tongue in &mut others {
            tongue.document.sheet.clone_from(&anchor.document.sheet);
            tongue.document.anchor = Some(anchor.sealing.id.clone());
        }
    }
    let mut cited = Vec::new();
    for mut tongue in std::iter::once(anchor).chain(others) {
        let said = cited_by(&tongue)?;
        if !said.is_empty() {
            tongue.document.cited = Some(said);
        }
        let id = tongue.sealing.id.clone();
        let sealed = writer.sealed(
            tongue.document,
            tongue.pictures,
            tongue.files,
            &tongue.sealing,
        )?;
        going.push(Going { id, sealed });
        cited.extend(tongue.cited);
    }
    Ok(cited)
}

fn tongues(family: &Family) -> impl Iterator<Item = &Tongue> {
    std::iter::once(&family.anchor).chain(&family.others)
}

// Each document goes once under its id, so two that claim one would leave one of them unsaid.
fn each_goes_once(family: &Family) -> Result<(), Refused> {
    let mut going = vec![family];
    for one in tongues(family).flat_map(|tongue| &tongue.cited) {
        let one: &Family = one;
        if going.contains(&one) {
            continue;
        }
        let same_id = |tongue: &Tongue| tongues(one).any(|its| its.sealing.id == tongue.sealing.id);
        if going.iter().any(|kept| tongues(kept).any(same_id)) {
            return Err(Refused::DoesNotAdd(
                "two different documents go under one id",
            ));
        }
        going.push(one);
    }
    Ok(())
}

pub fn sealed_family(writer: &mut Writer, family: Family) -> Result<Vec<Going>, Refused> {
    each_goes_once(&family)?;
    let mut going = Vec::new();
    let cited = languages_sealed(writer, family, &mut going)?;
    let mut seen = BTreeSet::new();
    let once_each: Vec<Arc<Family>> = cited
        .into_iter()
        .filter(|one| seen.insert(one.anchor.sealing.id.clone()))
        .collect();
    for one in once_each {
        languages_sealed(writer, Arc::unwrap_or_clone(one), &mut going)?;
    }
    Ok(going)
}
