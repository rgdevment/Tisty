use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{
    Moved, PAPERS, Trouble, Undecided, copy_onto, docs_lock, io, joined, landed, plainly, straight,
    write,
};

pub(crate) fn settled_body(data: &Path, id: &str, mine: &Path, theirs: &Path) {
    let said = std::fs::read_to_string(mine)
        .or_else(|_| std::fs::read_to_string(theirs))
        .ok();
    let Some(said) = said else {
        tisty_core::docs::forget_carried(data, id);
        return;
    };
    if tisty_core::docs::keep_carried(data, id, &said).is_err() {
        witness::warn(
            channel::SYNC,
            "the settled body could not be kept, so the next round has no base to lean on",
            &[("at", Fact::Id(id.to_string()))],
        );
        tisty_core::docs::forget_carried(data, id);
    }
}

pub fn carry_papers(data: &Path, dest: &Path, alive: &[String]) -> Result<Moved, Trouble> {
    carry_papers_leaning_on(data, dest, alive, &[], false)
}

pub fn carry_papers_holding(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
) -> Result<Moved, Trouble> {
    carry_papers_leaning_on(data, dest, alive, shut, false)
}

pub(crate) fn carry_papers_leaning_on(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
    again: bool,
) -> Result<Moved, Trouble> {
    use tisty_core::docs::{Carried, Move, Prints, moved, print_of};

    let here = data.join(PAPERS);
    let there = dest.join(PAPERS);
    straight(&there, dest)?;
    let was = Carried::read(data);
    let mut said = was.clone();
    let mut prints = Prints::read(data);
    let asked = prints.clone();
    let mut done = Moved::default();

    let outcome = (|| -> Result<(), Trouble> {
        for id in alive {
            let (Ok(mine), Ok(theirs)) = (
                tisty_core::docs::resolve(&here, id),
                tisty_core::docs::resolve(&there, id),
            ) else {
                witness::warn(
                    channel::SYNC,
                    "a document was named in a way no document can be named",
                    &[("at", Fact::Id(id.clone()))],
                );
                continue;
            };
            if plainly(&theirs).is_err() || plainly(&mine).is_err() {
                done.astray.push(id.clone());
                continue;
            }
            let (ours, yours) = match (prints.of(&mine), prints.of(&theirs)) {
                (Ok(ours), Ok(yours)) => (ours, yours),
                (here, there) => {
                    let why = here.err().or(there.err());
                    witness::warn(
                        channel::SYNC,
                        "a document could not be read, so this turn leaves it alone",
                        &[
                            ("at", Fact::Id(id.clone())),
                            (
                                "why",
                                Fact::Why(why.map(|e| e.to_string()).unwrap_or_else(|| "?".into())),
                            ),
                        ],
                    );
                    done.astray.push(id.clone());
                    continue;
                }
            };

            match moved(said.of(id), ours.as_deref(), yours.as_deref()) {
                Move::Nothing => {
                    if again && mine.is_file() {
                        std::fs::create_dir_all(&there).map_err(io)?;
                        copy_onto(&mine, &theirs)?;
                    }
                    if let Some(print) = ours.or(yours) {
                        let kept = tisty_core::docs::carried_at(data, id)
                            .and_then(|at| prints.of(&at).ok().flatten());
                        let steady = said.of(id) == Some(print.as_str())
                            && kept.as_deref() == Some(print.as_str());
                        if !steady {
                            settled_body(data, id, &mine, &theirs);
                        }
                        said.keep(id, &print);
                    }
                }
                Move::Send => {
                    std::fs::create_dir_all(&there).map_err(io)?;
                    copy_onto(&mine, &theirs)?;
                    done.sent += 1;
                    if let Some(print) = ours {
                        settled_body(data, id, &mine, &theirs);
                        said.keep(id, &print);
                    }
                }
                Move::Bring if shut.contains(id) => {
                    witness::warn(
                        channel::SYNC,
                        "a locked document arrived changed, so it waits for the person",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    done.undecided.push(Undecided {
                        id: id.clone(),
                        theirs: yours.unwrap_or_default(),
                    });
                }
                Move::TheyDecide if shut.contains(id) => {
                    witness::warn(
                        channel::SYNC,
                        "a locked document was written on both sides, and no join writes over it",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    done.undecided.push(Undecided {
                        id: id.clone(),
                        theirs: yours.unwrap_or_default(),
                    });
                }
                Move::Bring => {
                    std::fs::create_dir_all(&here).map_err(io)?;
                    let _held = docs_lock(&here, id);
                    copy_onto(&theirs, &mine)?;
                    done.brought += 1;
                    done.arrived.push(id.clone());
                    if let Some(print) = yours {
                        settled_body(data, id, &mine, &theirs);
                        said.keep(id, &print);
                    }
                }
                Move::TheyDecide => {
                    let _held = docs_lock(&here, id);
                    match joined(data, dest, id, &mine, &theirs) {
                        Some(whole) => {
                            write(&mine, whole.as_bytes())?;
                            copy_onto(&mine, &theirs)?;
                            done.sent += 1;
                            done.brought += 1;
                            done.joined.push(id.clone());
                            done.arrived.push(id.clone());
                            if landed(&mine, &theirs) {
                                if let Ok(Some(print)) = print_of(&mine) {
                                    settled_body(data, id, &mine, &theirs);
                                    said.keep(id, &print);
                                }
                            } else {
                                witness::warn(
                                    channel::SYNC,
                                    "another machine wrote while this one joined, so the base stays put",
                                    &[("at", Fact::Id(id.clone()))],
                                );
                            }
                        }
                        None => done.undecided.push(Undecided {
                            id: id.clone(),
                            theirs: yours.unwrap_or_default(),
                        }),
                    }
                }
            }
        }
        Ok(())
    })();

    if said != was {
        said.save(data)
            .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    if prints != asked {
        prints.save(data);
    }
    outcome?;
    Ok(done)
}
