use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use super::call::{Call, Leaving};
use super::command::written;
use super::marks::{Mark, Session};
use super::outside::{Here, depends_on_where_it_runs};

const FOLDERS_A_SESSION: usize = 3;
const CARDS: &str = "outside-cards";

pub fn asked(
    session: &Session,
    here: &Here,
    call: &Call,
    tool: Option<&str>,
    at: i128,
) -> Option<&'static str> {
    // Claimed before it speaks, so a mark that cannot be kept never asks twice.
    let (first, said) = match call.leaving(tool) {
        Leaving::Nothing => return None,
        Leaving::Card => (session.claim(CARDS), crate::mcp::CARD_LEFT_OUTSIDE),
        Leaving::File(path) => {
            let folder = here.outside(&path, call.cwd())?;
            (first_there(session, &folder), crate::mcp::FILE_LEFT_OUTSIDE)
        }
        Leaving::Command(command) => {
            let folder = by_command(here, &command, call.cwd())?;
            let said = crate::mcp::COMMAND_LEAVES_OUTSIDE;
            (first_there(session, &folder), said)
        }
    };
    if !first {
        return None;
    }
    session.mark(Mark::Asked, at);
    Some(said)
}

fn by_command(here: &Here, command: &str, cwd: &str) -> Option<PathBuf> {
    written(command)
        .into_iter()
        .filter(|one| !(one.moved && depends_on_where_it_runs(&one.path)))
        .find_map(|one| here.outside_in_shell(&one.path, cwd))
}

fn first_there(session: &Session, folder: &Path) -> bool {
    let asked_above = folder
        .ancestors()
        .skip(1)
        .any(|one| session.claimed(&print(one)));
    !asked_above
        && session.claim(&print(folder))
        && (0..FOLDERS_A_SESSION).any(|one| session.claim(&format!("outside-{one}")))
}

fn print(folder: &Path) -> String {
    let said = folder.to_string_lossy();
    let mut hasher = DefaultHasher::new();
    match cfg!(windows) {
        true => said.to_lowercase().hash(&mut hasher),
        false => said.hash(&mut hasher),
    }
    format!("outside-{:016x}", hasher.finish())
}

#[cfg(test)]
#[path = "asking_test.rs"]
mod tests;
