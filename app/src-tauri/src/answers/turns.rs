use crate::{Answer, Refusal, Session};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Turn {
    pub hand: String,
    pub n: u64,
}

struct Standing {
    hand: String,
    n: u64,
    landed: Option<u64>,
}

#[derive(Default)]
pub struct Turns(std::collections::HashMap<String, Standing>);

impl Turns {
    fn presented(&mut self, file: &str, turn: &Turn) -> Result<(), Refusal> {
        match self.0.get_mut(file) {
            Some(was) if was.hand != turn.hand || turn.n < was.n => {
                Err(Refusal::about("saveLate", file))
            }
            Some(was) => {
                was.n = turn.n;
                Ok(())
            }
            None => {
                self.0.insert(file.to_string(), Standing::of(turn));
                Ok(())
            }
        }
    }

    pub fn opened(&mut self, file: &str, turn: &Turn) {
        match self.0.get_mut(file) {
            Some(was) if was.hand == turn.hand => was.n = was.n.max(turn.n),
            _ => {
                self.0.insert(file.to_string(), Standing::of(turn));
            }
        }
    }

    fn landed(&mut self, file: &str, turn: &Turn) {
        if let Some(was) = self.0.get_mut(file)
            && was.hand == turn.hand
        {
            was.landed = Some(turn.n);
        }
    }

    fn last_landed(&self, file: &str, turn: &Turn) -> Option<u64> {
        self.0
            .get(file)
            .filter(|was| was.hand == turn.hand)
            .and_then(|was| was.landed)
    }
}

impl Standing {
    fn of(turn: &Turn) -> Self {
        Self {
            hand: turn.hand.clone(),
            n: turn.n,
            landed: None,
        }
    }
}

pub(crate) fn in_turn<T>(
    session: &mut Session,
    file: &str,
    turn: Option<Turn>,
    write: impl FnOnce(&mut Session) -> Answer<T>,
    wrote: impl FnOnce(&T) -> bool,
) -> Answer<T> {
    let Some(turn) = turn else {
        return write(session);
    };
    session.turns.presented(file, &turn)?;
    match write(session) {
        Ok(done) => {
            if wrote(&done) {
                session.turns.landed(file, &turn);
            }
            Ok(done)
        }
        Err(refused) if refused.code == "documentMoved" => {
            Err(refused.with_landed(session.turns.last_landed(file, &turn)))
        }
        Err(refused) => Err(refused),
    }
}

#[cfg(test)]
#[path = "turns_test.rs"]
mod tests;
