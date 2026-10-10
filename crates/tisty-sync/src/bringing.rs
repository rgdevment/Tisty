use super::*;

pub(crate) fn seats(store: &Path) -> std::collections::BTreeSet<tisty_core::event::DeviceId> {
    let Ok(entries) = std::fs::read_dir(store) else {
        return Default::default();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|one| one.path().is_dir())
        .filter_map(|one| one.file_name().to_str().map(str::to_string))
        .map(tisty_core::event::DeviceId)
        .collect()
}

fn anything_signed_in(dir: &Path) -> bool {
    let signed = tisty_core::store::segments_in(dir).is_ok_and(|found| {
        found
            .iter()
            .any(|one| one.with_extension(tisty_core::signing::SIG).is_file())
    });
    signed || tisty_core::store::sealed_in(dir)
}

/// A machine's own word for what it signs with, read from the log only where nobody has answered
/// for it yet. It catches a history altered without that machine's key, which is the hand anybody
/// who reaches the folder can lay on it; what it cannot catch is the key itself being planted,
/// and that is what a person confirming one closes.
fn claimed(
    store: &Path,
    who: &tisty_core::DeviceId,
    knew: &mut Option<tisty_core::store::Ledger>,
) -> Result<Option<String>, ()> {
    let told = match knew {
        Some(told) => told,
        None => match tisty_core::store::ledger(store) {
            Ok(told) => knew.insert(told),
            Err(e) => {
                witness::warn(
                    channel::SYNC,
                    "this machine's own log would not say what the others sign with, so nothing signed was taken in",
                    &[("why", Fact::Why(e.to_string()))],
                );
                return Err(());
            }
        },
    };
    Ok(told.keys.get(who).cloned())
}

fn removed(store: &Path, named: &str, knew: &mut Option<tisty_core::store::Ledger>) -> bool {
    if knew.is_none() {
        *knew = tisty_core::store::ledger(store).ok();
    }
    knew.as_ref()
        .is_some_and(|told| told.was_removed(&tisty_core::DeviceId(named.to_string())))
}

/// An agent is taken on the word of a host this machine trusts, for the very key the host wrote.
fn hosted(
    data: &Path,
    device: &str,
    theirs: &Path,
    who: &tisty_core::DeviceId,
    says: &str,
    knew: &mut Option<tisty_core::store::Ledger>,
) -> bool {
    let Some((host, key)) = knew
        .as_ref()
        .and_then(|told| told.vouched.get(who))
        .cloned()
    else {
        return false;
    };
    // Its join is checked under this very key before anything comes in, so the claim cannot be borrowed.
    if key != says || !tisty_core::store::introduced::introduced_in(theirs, who).agent {
        return false;
    }
    let trusted = host.0.eq_ignore_ascii_case(device)
        || tisty_core::vouched::confirmed(data, &host).is_some();
    trusted && tisty_core::vouched::through(data, who, says, &host)
}

fn owes_a_signature(
    mine: &Path,
    theirs: &Path,
    who: &tisty_core::DeviceId,
    known: bool,
) -> Result<bool, ()> {
    if known
        || tisty_core::store::says_a_key_in(theirs, who)
        || tisty_core::store::says_a_key_in(mine, who)
    {
        return Ok(true);
    }
    tisty_core::store::written_since(theirs, tisty_core::event::SIGNED_FROM).map_err(|e| {
        witness::warn(
            channel::SYNC,
            "the schema a machine's history was written at could not be read, so it was left out",
            &[
                ("at", Fact::Path(theirs.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    })
}

/// Checked before any of it is taken in, and only from where the last round left off.
#[allow(clippy::too_many_arguments)]
fn answers_for_itself(
    data: &Path,
    store: &Path,
    dest: &Path,
    theirs: &Path,
    named: &str,
    device: &str,
    adopting: bool,
    knew: &mut Option<tisty_core::store::Ledger>,
    alike: &mut Alike,
) -> Answered {
    let who = tisty_core::DeviceId(named.to_string());
    let ours = named.eq_ignore_ascii_case(device);
    let mine = store.join(named);
    let from = verified::of(data, dest, named);
    let signed = anything_signed_in(theirs);
    let mut stood = tisty_core::vouched::confirmed(data, &who).map(|one| one.key);
    let mut carried = None;
    if stood.is_none() && !ours && signed {
        let mut said_a_key = false;
        let says = match claimed(store, &who, knew) {
            Ok(Some(claim)) => Some(claim),
            Ok(None) => {
                let said = tisty_core::store::keys_said_in(theirs, &who);
                said_a_key = said.any;
                said.readable
            }
            Err(()) => return Answered::Unreadable,
        };
        match says {
            Some(says) if adopting && tisty_core::vouched::confirm(data, &who, &says) => {
                witness::note(
                    channel::SYNC,
                    "reaching this folder for the first time answered for the key of a machine already writing in it",
                    &[("at", Fact::Id(named.to_string()))],
                );
                stood = Some(says);
            }
            Some(says)
                if mine.is_dir()
                    && tisty_core::store::before::first_key_past(&mine, theirs, &who)
                        .as_deref()
                        == Some(says.as_str()) =>
            {
                carried = Some(says.clone());
                stood = Some(says);
            }
            Some(says) if hosted(data, device, theirs, &who, &says, knew) => {
                witness::note(
                    channel::SYNC,
                    "an agent was answered for by the host it runs on, already confirmed here",
                    &[("at", Fact::Id(named.to_string()))],
                );
                stood = Some(says);
            }
            Some(_) => {
                witness::note(
                    channel::SYNC,
                    "a machine says it signs what it writes and nobody here has answered for its key, so what it writes waits",
                    &[("at", Fact::Id(named.to_string()))],
                );
                return Answered::Unconfirmed;
            }
            None if adopting && !said_a_key => return Answered::Unsaid,
            None => {
                witness::note(
                    channel::SYNC,
                    "a machine signs what it writes and has not yet said with what, so what it writes waits",
                    &[("at", Fact::Id(named.to_string()))],
                );
                return Answered::Unconfirmed;
            }
        }
    }
    if !signed {
        let known = from.signing || stood.is_some();
        let Ok(owed) = owes_a_signature(&mine, theirs, &who, known) else {
            return Answered::Unreadable;
        };
        if !owed {
            return Answered::Yes;
        }
        witness::warn(
            channel::SYNC,
            "a history that owes a signature arrived with none at all, so none of it was taken in",
            &[("at", Fact::Id(named.to_string()))],
        );
        return Answered::Disowned;
    }
    let said = match stood {
        Some(key) => Some(key),
        None => match claimed(store, &who, knew) {
            Ok(said) => said,
            Err(()) => return Answered::Unreadable,
        },
    };
    let by = said.and_then(|said| tisty_core::signing::read(&said));
    let Some(by) = by else {
        witness::warn(
            channel::SYNC,
            "a signed history cannot be checked from here for want of a key that reads, so it was left in the folder",
            &[("at", Fact::Id(named.to_string()))],
        );
        return Answered::Unreadable;
    };
    use tisty_core::answering::Adrift;
    let ours_already = alike.of(named, theirs, &mine).clone();
    let answers = tisty_core::answering::answers(theirs, &who, &by, from, &|segment| {
        ours_already.contains(std::ffi::OsStr::new(segment))
    });
    match answers {
        Ok(held) => {
            verified::keep(data, dest, named, held);
            if let Some(said) = carried
                && tisty_core::vouched::carried(data, &who, &said)
            {
                witness::note(
                    channel::SYNC,
                    "a machine this store held from before it signed was answered for by the history already here",
                    &[("at", Fact::Id(named.to_string()))],
                );
            }
            Answered::Yes
        }
        Err(Adrift::Unreadable(why)) => {
            witness::warn(
                channel::SYNC,
                "a history in the shared folder could not be read through to its signature, so it was left out",
                &[("at", Fact::Id(named.to_string())), ("why", Fact::Why(why))],
            );
            Answered::Unreadable
        }
        Err(Adrift::Disowned(_)) if carried.is_some() => {
            witness::note(
                channel::SYNC,
                "a machine held from before it signed does not answer to the key it says, so a person decides",
                &[("at", Fact::Id(named.to_string()))],
            );
            Answered::Unconfirmed
        }
        Err(Adrift::Disowned(segment)) => {
            // Read again from the first line next round, but never give up knowing it signed:
            // that latch is what tells a signature taken away from a history written before one.
            verified::keep(
                data,
                dest,
                named,
                tisty_core::answering::Reached {
                    signing: true,
                    ..Default::default()
                },
            );
            witness::warn(
                channel::SYNC,
                "a history in the shared folder does not answer to the key kept for that machine, so none of it was taken in",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("segment", Fact::Id(segment)),
                ],
            );
            Answered::Disowned
        }
    }
}

const BRINGING: &str = ".bringing";
const LEFT_FOR: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

pub(crate) struct Aside(std::path::PathBuf);

impl Aside {
    // The window and the command line may both be in a round, so each takes a place of its own.
    pub(crate) fn taken(data: &Path) -> Self {
        let all = data.join(BRINGING);
        swept_aside(&all);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        Self(all.join(format!("{}-{stamp}", std::process::id())))
    }

    pub(crate) fn at(&self) -> &Path {
        &self.0
    }
}

impl Drop for Aside {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
        if let Some(all) = self.0.parent() {
            let _ = std::fs::remove_dir(all);
        }
    }
}

// Only what a round cut short a day ago left: a round still running keeps its place.
fn swept_aside(all: &Path) {
    let Ok(entries) = std::fs::read_dir(all) else {
        return;
    };
    for entry in entries.filter_map(|one| one.ok()) {
        let stale = entry
            .metadata()
            .and_then(|told| told.modified())
            .ok()
            .and_then(|when| when.elapsed().ok())
            .is_some_and(|age| age > LEFT_FOR);
        if stale {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

// Unread or half copied, the history is left out this turn; the caller says how.
fn staging(
    alike: &mut Alike,
    named: &str,
    theirs: &Path,
    mine: &Path,
    aside: &Aside,
) -> Option<std::path::PathBuf> {
    let into = aside.at().join(named);
    let known = alike.of(named, theirs, mine).clone();
    match crate::segments::staged(theirs, mine, &known, &into) {
        Ok(()) => Some(into),
        Err(why) => {
            witness::warn(
                channel::SYNC,
                "a machine's history could not be copied aside to be checked, so it was left out",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("why", Fact::Why(why.to_string())),
                ],
            );
            None
        }
    }
}

enum Answered {
    Yes,
    /// Taken on the first folder's word before it said its key, so adopting waits for that key.
    Unsaid,
    Unreadable,
    Disowned,
    Unconfirmed,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn bring(
    data: &Path,
    store: &Path,
    device: &str,
    dest: &Path,
    adopting: &std::collections::BTreeSet<String>,
    moved: &mut Moved,
    alike: &mut Alike,
    saying: &mut dyn FnMut(Reached),
) -> Result<usize, Trouble> {
    let mut brought = 0;
    let aside = Aside::taken(data);
    let mut knew: Option<tisty_core::store::Ledger> = None;
    let mut away: std::collections::BTreeMap<String, turned::Away> = Default::default();
    let at = dest.join(STORE);
    let entries = match std::fs::read_dir(&at) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "folder unreadable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
            return Ok(0);
        }
    };

    let entries: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();
    let whole = entries.len();
    for (done, entry) in entries.into_iter().enumerate() {
        saying(Reached::Along {
            stage: Stage::Log,
            done,
            whole,
        });
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        // Anybody who reaches the folder can name a directory, and a name is what every memo and
        // every ledger line is keyed by. One that could not be a machine of ours never becomes one.
        if !tisty_core::store::is_device_name(named) {
            witness::warn(
                channel::SYNC,
                "the shared folder holds a directory that no machine of ours could be named, so it was left alone",
                &[("at", Fact::Id(named.to_string()))],
            );
            continue;
        }
        let mine = store.join(named);
        let pending = tisty_core::holes::brought_down(
            tisty_core::holes::still_away(&entry.path()),
            BROUGHT_WITHIN,
        );
        if !pending.is_empty() {
            witness::note(
                channel::SYNC,
                "a machine's history is still on its way down from the cloud, so it waits for the next turn",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("files", Fact::Count(pending.len())),
                ],
            );
            tisty_core::holes::ask_for(pending);
            moved.coming.push(named.to_string());
            continue;
        }
        if named.eq_ignore_ascii_case(device) {
            if !alike.settled(named, &entry.path(), &mine, Toward::Home)
                && ours_went_missing(&mine, &entry.path())
            {
                let Some(theirs) = staging(alike, named, &entry.path(), &mine, &aside) else {
                    continue;
                };
                match tisty_core::store::alone(&mine) {
                    // Our own name is the one worth wearing: what comes back under it is checked
                    // like anybody else's, or the next thing we write would sign it as ours.
                    Some(_held)
                        if !matches!(
                            answers_for_itself(
                                data,
                                store,
                                dest,
                                &theirs,
                                named,
                                device,
                                adopting.contains(named),
                                &mut knew,
                                alike,
                            ),
                            Answered::Yes
                        ) =>
                    {
                        moved.disowned.push(named.to_string());
                    }
                    Some(_held) if ours_went_missing(&mine, &theirs) => {
                        witness::warn(
                            channel::SYNC,
                            "this machine's own history was shorter here than in the shared folder, so it was taken back",
                            &[("at", Fact::Id(named.to_string()))],
                        );
                        plainly(&mine)?;
                        brought += alike.carried(named, &theirs, &mine, Toward::Home, false)?;
                    }
                    Some(_) => {}
                    None => witness::warn(
                        channel::SYNC,
                        "this machine's own history is being written, so it was left as it is",
                        &[("at", Fact::Id(named.to_string()))],
                    ),
                }
            }
            continue;
        }
        plainly(&mine)?;
        let settled = alike.settled(named, &entry.path(), &mine, Toward::Home)
            && !crate::segments::beside_differs(&entry.path(), &mine);
        let theirs = match settled {
            true => entry.path(),
            false => match staging(alike, named, &entry.path(), &mine, &aside) {
                Some(theirs) => theirs,
                None => {
                    if !removed(store, named, &mut knew) {
                        moved.unreadable.push(named.to_string());
                    }
                    continue;
                }
            },
        };
        let answered = answers_for_itself(
            data,
            store,
            dest,
            &theirs,
            named,
            device,
            adopting.contains(named),
            &mut knew,
            alike,
        );
        // Removed by the person, so nothing of it is theirs to hear about any more.
        if !matches!(answered, Answered::Yes | Answered::Unsaid) && removed(store, named, &mut knew)
        {
            continue;
        }
        match answered {
            Answered::Yes => {}
            Answered::Unsaid => moved.unsaid.push(named.to_string()),
            Answered::Unreadable => {
                away.insert(named.to_string(), turned::Away::Unreadable);
                moved.unreadable.push(named.to_string());
                continue;
            }
            Answered::Disowned => {
                away.insert(named.to_string(), turned::Away::Disowned);
                moved.disowned.push(named.to_string());
                continue;
            }
            Answered::Unconfirmed => {
                away.insert(named.to_string(), turned::Away::Unconfirmed);
                moved.unconfirmed.push(named.to_string());
                continue;
            }
        }
        if !settled {
            let coming = match tisty_core::store::check_device(&theirs)
                .and_then(|_| tisty_core::store::distinct_in(&theirs))
            {
                Ok(coming) => coming,
                Err(tisty_core::Error::UnsupportedVersion { .. }) => {
                    witness::warn(
                        channel::SYNC,
                        "another machine writes a newer schema, so nothing was carried either way",
                        &[("at", Fact::Id(named.to_string()))],
                    );
                    return Err(Trouble::Newer(named.to_string()));
                }
                Err(why) => {
                    witness::warn(
                        channel::SYNC,
                        "a machine's history in the shared folder could not be read, so it was left out",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("why", Fact::Why(why.to_string())),
                        ],
                    );
                    moved.unreadable.push(named.to_string());
                    continue;
                }
            };
            let held = match tisty_core::store::distinct_in(&mine) {
                Ok(held) => held,
                Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => 0,
                Err(why) => {
                    witness::warn(
                        channel::SYNC,
                        "what we hold for a machine cannot be counted, so nothing replaces it",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("why", Fact::Why(why.to_string())),
                        ],
                    );
                    continue;
                }
            };
            if coming < held {
                if !matches!(one_grew_from_the_other(&mine, &theirs), Grew::Yes) {
                    witness::warn(
                        channel::SYNC,
                        "a shorter history for a machine was left where it was",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("held", Fact::Count(held)),
                            ("coming", Fact::Count(coming)),
                        ],
                    );
                }
                continue;
            }
        }
        // Held the same in every byte, so nothing is copied from a folder that could change under us.
        if !settled {
            brought += alike.carried(named, &theirs, &mine, Toward::Home, false)?;
        }
    }
    saying(Reached::Along {
        stage: Stage::Log,
        done: whole,
        whole,
    });

    turned::keep(data, &away);
    if brought > 0 {
        tisty_core::store::read_all(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    Ok(brought)
}
