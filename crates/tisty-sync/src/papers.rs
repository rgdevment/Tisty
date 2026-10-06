use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{
    Holding, Moved, PAPERS, Reached, STORE, Stage, Trouble, Undecided, copy_onto, docs_lock, io,
    joined, pointed_away, straight, write,
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
    carry_papers_leaning_on(
        data,
        dest,
        alive,
        &[],
        None,
        None,
        false,
        false,
        true,
        &mut |_| {},
    )
}

pub fn carry_papers_holding(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
) -> Result<Moved, Trouble> {
    carry_papers_leaning_on(
        data,
        dest,
        alive,
        shut,
        None,
        None,
        false,
        false,
        true,
        &mut |_| {},
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn carry_papers_leaning_on(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
    empty: Option<&[String]>,
    printed: Option<&std::collections::BTreeMap<String, Answers>>,
    again: bool,
    been_here: bool,
    taking: bool,
    saying: &mut dyn FnMut(Reached),
) -> Result<Moved, Trouble> {
    use tisty_core::docs::{Carried, Move, Prints, Seen, moved, print_of};

    let here = data.join(PAPERS);
    let there = dest.join(PAPERS);
    straight(&there, dest)?;
    let was = Carried::read(data);
    let mut said = was.clone();
    if said.facing(dest, been_here) {
        tisty_core::docs::forget_what_was_carried(data);
    }
    let mut prints = Prints::read(data);
    let asked = prints.clone();
    let mut done = Moved::default();
    let mut asked_for = Vec::new();

    let whole = alive.len();
    let outcome = (|| -> Result<(), Trouble> {
        for (done_so_far, id) in alive.iter().enumerate() {
            saying(Reached::Along {
                stage: Stage::Papers,
                done: done_so_far,
                whole,
            });
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
            if tisty_core::holes::a_hole(&theirs) {
                asked_for.push(theirs);
                done.coming.push(id.clone());
                continue;
            }
            let told_empty = empty.is_none_or(|told| told.contains(id));
            let (ours, yours, mine_holds, theirs_holds) =
                match (prints.seen(&mine), prints.seen(&theirs)) {
                    (_, Ok(Seen::Linked)) => {
                        pointed_away(&theirs);
                        done.astray.push(id.clone());
                        continue;
                    }
                    (Ok(Seen::Linked), _) => {
                        pointed_away(&mine);
                        done.astray.push(id.clone());
                        continue;
                    }
                    (
                        Ok(Seen::Held {
                            print: ours,
                            weighs: mine_weighs,
                        }),
                        Ok(Seen::Held {
                            print: yours,
                            weighs: theirs_weighs,
                        }),
                    ) => (ours, yours, mine_weighs > 0, theirs_weighs > 0),
                    (here, there) => {
                        let why = here.err().or(there.err());
                        witness::warn(
                            channel::SYNC,
                            "a document could not be read, so this turn leaves it alone",
                            &[
                                ("at", Fact::Id(id.clone())),
                                (
                                    "why",
                                    Fact::Why(
                                        why.map(|e| e.to_string()).unwrap_or_else(|| "?".into()),
                                    ),
                                ),
                            ],
                        );
                        done.astray.push(id.clone());
                        continue;
                    }
                };

            let yours = a_body(yours, told_empty || !mine_holds, theirs_holds, id);

            let how = moved(said.of(id), ours.as_deref(), yours.as_deref());
            let answer = match how {
                Move::Bring | Move::TheyDecide if taking => {
                    answered_for(yours.as_ref(), printed.and_then(|told| told.get(id)), id)
                }
                _ => Answer::Yes,
            };
            let how = match how {
                Move::Bring | Move::TheyDecide if !taking => continue,
                // Only a body nobody here touched waits: one edited on both sides is put to the person as ever.
                Move::Bring if answer == Answer::Waits => {
                    witness::note(
                        channel::SYNC,
                        "a body a machine still waiting to be confirmed answers for waits with it",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    done.waiting.push(id.clone());
                    continue;
                }
                Move::Bring | Move::TheyDecide if matches!(answer, Answer::No | Answer::Waits) => {
                    done.undecided.push(Undecided {
                        id: id.clone(),
                        theirs: yours.unwrap_or_default(),
                    });
                    continue;
                }
                one => one,
            };

            match how {
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
                Move::Send
                    if printed.is_some_and(|told| {
                        told.get(id)
                            .is_some_and(|says| !says.answers_ours(ours.as_deref()))
                    }) =>
                {
                    done.unanswered.push(id.clone());
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
                    if answer == Answer::Doubtful
                        && let Ok(left) = std::fs::read_to_string(&theirs)
                    {
                        set_aside(data, id, &mine, &left);
                    }
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
                    match joined(data, dest, id, &mine, &theirs, said.of(id)) {
                        Some(whole) => {
                            if answer == Answer::Doubtful {
                                set_aside(data, id, &mine, &whole);
                            }
                            write(&mine, whole.as_bytes())?;
                            done.brought += 1;
                            done.joined.push(id.clone());
                            done.arrived.push(id.clone());
                            match (print_of(&theirs), yours) {
                                (Ok(Some(now)), Some(print)) if now == print => {
                                    settled_body(data, id, &theirs, &mine);
                                    said.keep(id, &print);
                                }
                                _ => witness::warn(
                                    channel::SYNC,
                                    "another machine wrote while this one joined, so the base stays put",
                                    &[("at", Fact::Id(id.clone()))],
                                ),
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
        saying(Reached::Along {
            stage: Stage::Papers,
            done: whole,
            whole,
        });
        Ok(())
    })();
    tisty_core::holes::ask_for(asked_for);

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

pub fn unclaimed(dest: &Path) -> Holding {
    match tisty_core::store::read_all(dest.join(STORE)) {
        Ok(events) => unclaimed_leaning_on(dest, &tisty_core::State::replay(&events)),
        Err(_) => Holding::Unreadable,
    }
}

pub(crate) fn unclaimed_leaning_on(dest: &Path, told: &tisty_core::State) -> Holding {
    let here = tisty_core::docs::names(&dest.join(PAPERS));
    let named: std::collections::BTreeSet<String> = told
        .docs
        .values()
        .map(|one| one.file.clone())
        .chain(told.shed.iter().cloned())
        .collect();
    match here.difference(&named).count() {
        0 => Holding::Whole,
        adrift => Holding::Strays(adrift),
    }
}

fn a_body(print: Option<String>, allowed: bool, holds: bool, id: &str) -> Option<String> {
    let print = print?;
    if allowed || holds {
        return Some(print);
    }
    witness::warn(
        channel::SYNC,
        "the folder holds nothing where this machine holds a body, so this turn leaves it there",
        &[("at", Fact::Id(id.to_string()))],
    );
    None
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Answers {
    pub newest: Option<String>,
    pub own: Option<String>,
    pub others: std::collections::BTreeSet<String>,
    pub waiting: std::collections::BTreeSet<String>,
}

impl Answers {
    fn answers_ours(&self, ours: Option<&str>) -> bool {
        let silent = self.newest.is_none() && self.own.is_none() && self.others.is_empty();
        silent || ours.is_none() || ours == self.own.as_deref() || ours == self.newest.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
    Yes,
    Doubtful,
    Waits,
    No,
}

fn set_aside(data: &Path, id: &str, mine: &Path, left: &str) {
    let Ok(was) = std::fs::read_to_string(mine) else {
        return;
    };
    if tisty_core::docs::kept_before(data, id, &was, left).is_err() {
        witness::warn(
            channel::SYNC,
            "what a document held could not be set aside before another machine's version replaced it",
            &[("at", Fact::Id(id.to_string()))],
        );
    }
}

fn answered_for(print: Option<&String>, says: Option<&Answers>, id: &str) -> Answer {
    let (Some(print), Some(says)) = (print, says) else {
        return Answer::Yes;
    };
    if says.newest.as_ref() == Some(print) || (says.newest.is_none() && says.others.is_empty()) {
        return Answer::Yes;
    }
    if says.others.contains(print) {
        return Answer::Doubtful;
    }
    if says.waiting.contains(print) {
        return Answer::Waits;
    }
    witness::warn(
        channel::SYNC,
        "the folder holds a body the log does not answer for, so the person decides it",
        &[("at", Fact::Id(id.to_string()))],
    );
    Answer::No
}

#[cfg(test)]
#[path = "papers_test.rs"]
mod tests;
