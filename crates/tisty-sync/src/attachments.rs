use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tisty_core::config::Holds;

use crate::{HELD, Reached, Trouble, copy_held, left_behind, let_go_of};

pub struct Giving<'a> {
    pub data: &'a Path,
    pub buried: &'a BTreeSet<String>,
    pub avowed: &'a BTreeMap<String, (String, u64)>,
    pub again: bool,
    pub holds: Holds,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Given {
    pub sent: usize,
    pub freed: u64,
    pub let_go: Vec<String>,
}

pub struct Taking<'a> {
    pub data: &'a Path,
    pub buried: &'a BTreeSet<String>,
    pub avowed: &'a BTreeMap<String, (String, u64)>,
    pub reachable: Option<&'a BTreeSet<String>>,
    pub holds: Holds,
    pub saying: &'a mut dyn FnMut(Reached),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Taken {
    pub brought: usize,
    pub took_in: Vec<(String, String, u64)>,
}

pub trait Attachments {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble>;
    fn take(&mut self, round: Taking) -> Result<Taken, Trouble>;
}

pub struct Beside<'a> {
    pub dest: &'a Path,
}

impl Attachments for Beside<'_> {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble> {
        let mut carried = Vec::new();
        let sent = copy_held(
            &round.data.join(HELD),
            &self.dest.join(HELD),
            round.buried,
            round.again,
            None,
            None,
            None,
            Some(&mut carried),
            round.avowed,
            &mut |_| {},
        )?;
        let (freed, let_go) = match round.holds {
            Holds::Shared => let_go_of(
                round.data,
                self.dest,
                &carried,
                tisty_core::attach::COPIED_UP_TO,
            ),
            _ => (0, Vec::new()),
        };
        Ok(Given {
            sent,
            freed,
            let_go,
        })
    }

    fn take(&mut self, round: Taking) -> Result<Taken, Trouble> {
        let mut took_in = Vec::new();
        let brought = copy_held(
            &self.dest.join(HELD),
            &round.data.join(HELD),
            round.buried,
            false,
            Some(round.data),
            left_behind(round.holds),
            round.reachable,
            Some(&mut took_in),
            round.avowed,
            round.saying,
        )?;
        Ok(Taken { brought, took_in })
    }
}
