use tisty_core::agents::{Unfit, proposable};
use tisty_core::event::{Op, Proposal as Settled};

use super::agents_sync::{Line, compared};
use crate::{Answer, Refusal, Session};

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    print: String,
    via: Option<String>,
    at: String,
    stale: bool,
    unfit: Option<&'static str>,
    lines: Vec<Line>,
}

fn id_of(session: &Session, file: &str) -> Option<tisty_core::model::DocId> {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .map(|one| one.id)
}

pub(crate) fn proposal_of(session: &Session, file: &str, approved: &str) -> Option<Proposal> {
    let waiting = session.state.proposals.get(&id_of(session, file)?)?;
    let print = tisty_core::docs::print_of_text(approved);
    Some(Proposal {
        print: waiting.print.clone(),
        via: waiting.via.clone(),
        at: waiting.at.to_string(),
        stale: waiting.base.as_ref().is_some_and(|base| *base != print),
        unfit: fits(approved, &waiting.body).err(),
        lines: compared(approved.as_bytes(), waiting.body.as_bytes()),
    })
}

// What the person approves is the proposal the card showed, over the text it showed it against.
pub(crate) fn approve(session: &mut Session, file: &str, print: &str, over: &str) -> Answer<()> {
    let id = id_of(session, file).ok_or_else(|| Refusal::of("agentNoCard"))?;
    let body = waiting(session, id, print)?.body.clone();
    let now = tisty_core::docs::read(&session.paths.docs(), file)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    if tisty_core::docs::print_of_text(&now) != over {
        return Err(Refusal::of("agentDocChanged"));
    }
    // Lines the comparison shows as approved must be so, or approving would take them in unread.
    if session
        .state
        .docs
        .get(&id)
        .and_then(|one| one.agreed.as_deref())
        != Some(over)
    {
        return Err(Refusal::of("agentProposalOverUnagreed"));
    }
    fits(&now, &body).map_err(Refusal::of)?;
    super::papers::written_by_hand(
        session,
        file.to_string(),
        body.clone(),
        None,
        Some(over.to_string()),
    )?;
    let hand = crate::signing(&session.state);
    session.retell_written(file, &body, hand);
    settle(session, id, print)
}

// One reading for the card's warning and the refusal, or an old proposal takes back an old `agent:` line.
fn fits(now: &str, proposal: &str) -> Result<(), &'static str> {
    match proposable(now, proposal) {
        Ok(()) => Ok(()),
        Err(Unfit::NoCard | Unfit::Retargeted) => Err("agentProposalRetargeted"),
        Err(_) => Err("agentProposalUnfit"),
    }
}

pub(crate) fn discard(session: &mut Session, file: &str, print: &str) -> Answer<()> {
    let id = id_of(session, file).ok_or_else(|| Refusal::of("agentNoCard"))?;
    waiting(session, id, print)?;
    settle(session, id, print)
}

fn waiting<'a>(
    session: &'a Session,
    id: tisty_core::model::DocId,
    print: &str,
) -> Answer<&'a tisty_core::model::Proposed> {
    session
        .state
        .proposals
        .get(&id)
        .filter(|one| one.print == print)
        .ok_or_else(|| Refusal::of("agentProposalChanged"))
}

fn settle(session: &mut Session, id: tisty_core::model::DocId, print: &str) -> Answer<()> {
    session
        .commit(Op::DocProposal {
            id,
            d: Settled {
                body: None,
                base: None,
                settles: Some(print.to_string()),
            },
        })
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))
}
