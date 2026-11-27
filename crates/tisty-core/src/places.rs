use std::collections::BTreeSet;
use std::path::Path;

use crate::State;
use crate::model::{DocId, FolderId, Kept};

/// What a document or a folder is to the rules of what it takes: Ideas, the diary and AGENTS each hold only their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    Plain,
    Ideas,
    Diary,
    Agents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Making {
    Doc,
    Diagram,
    Folder,
    Import,
    Parcel,
}

pub const MAKINGS: [Making; 5] = [
    Making::Doc,
    Making::Diagram,
    Making::Folder,
    Making::Import,
    Making::Parcel,
];

pub const PLACES: [Place; 4] = [Place::Plain, Place::Ideas, Place::Diary, Place::Agents];

/// A refusal's code, the same one the window shows and the assistant's door words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ruled(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Rules {
    pub makes: Vec<Making>,
    pub pages: bool,
    pub moves: bool,
    pub copies: bool,
    pub goes: bool,
}

impl Place {
    /// What can be made inside: only a folder makes anything, and AGENTS makes only its rules.
    pub fn makes(self, what: Making) -> Result<(), Ruled> {
        match (self, what) {
            (Place::Plain, _) | (Place::Agents, Making::Doc) => Ok(()),
            (Place::Agents, _) => Err(Ruled("agentsHoldOnlyRules")),
            (Place::Ideas, _) => Err(Ruled("ideasHoldOnlyIdeas")),
            (Place::Diary, _) => Err(Ruled("diaryHoldsOnlyDays")),
        }
    }

    /// Whether a document of this place takes a page hung from elsewhere.
    pub fn takes_pages(self) -> Result<(), Ruled> {
        match self {
            Place::Ideas => Err(Ruled("ideasHoldNoPages")),
            Place::Diary => Err(Ruled("diaryHoldsOnlyDays")),
            Place::Plain | Place::Agents => Ok(()),
        }
    }

    /// Whether a document of this place leaves where it stands: into a folder, or as somebody's page.
    pub fn moves(self) -> Result<(), Ruled> {
        match self {
            Place::Ideas => Err(Ruled("ideasStayPut")),
            Place::Diary => Err(Ruled("diaryStaysPut")),
            Place::Plain | Place::Agents => Ok(()),
        }
    }

    /// Whether a document of this place is copied: there is one Ideas sheet, and a copied diary would be a second one.
    pub fn copies(self) -> Result<(), Ruled> {
        match self {
            Place::Ideas => Err(Ruled("ideasAreOne")),
            Place::Diary => Err(Ruled("diaryIsOne")),
            Place::Plain | Place::Agents => Ok(()),
        }
    }

    /// Whether a document of this place is archived or deleted: without Ideas, Tisty would start an empty sheet and leave every idea behind.
    pub fn goes(self) -> Result<(), Ruled> {
        match self {
            Place::Ideas => Err(Ruled("ideasStay")),
            Place::Plain | Place::Diary | Place::Agents => Ok(()),
        }
    }

    pub fn rules(self) -> Rules {
        Rules {
            makes: MAKINGS
                .into_iter()
                .filter(|one| self.makes(*one).is_ok())
                .collect(),
            pages: self.takes_pages().is_ok(),
            moves: self.moves().is_ok(),
            copies: self.copies().is_ok(),
            goes: self.goes().is_ok(),
        }
    }
}

/// Which books are the diary is read from their words, so it is read once and asked many times.
pub struct Places {
    diaries: BTreeSet<DocId>,
    agents: BTreeSet<FolderId>,
}

impl Places {
    pub fn read(docs: &Path, state: &State) -> Self {
        Self {
            diaries: crate::diary::books(docs, state),
            agents: crate::agents::trees(state),
        }
    }

    /// A page stands where the document it belongs to stands.
    pub fn of_doc(&self, state: &State, kept: &Kept) -> Place {
        let book = kept
            .page_of
            .and_then(|up| state.docs.get(&up))
            .unwrap_or(kept);
        if book.brainstorm {
            return Place::Ideas;
        }
        if self.diaries.contains(&book.id) {
            return Place::Diary;
        }
        self.of_folder(book.folder)
    }

    pub fn of_folder(&self, at: Option<FolderId>) -> Place {
        match at.is_some_and(|at| self.agents.contains(&at)) {
            true => Place::Agents,
            false => Place::Plain,
        }
    }

    /// Hanging `page` under `book` moves the page and fills the book, so both have their say.
    pub fn may_hang(&self, state: &State, page: &Kept, book: &Kept) -> Result<(), Ruled> {
        self.of_doc(state, page).moves()?;
        self.of_doc(state, book).takes_pages()
    }

    pub fn may_file(&self, state: &State, kept: &Kept) -> Result<(), Ruled> {
        self.of_doc(state, kept).moves()
    }

    pub fn may_make(&self, into: Option<FolderId>, what: Making) -> Result<(), Ruled> {
        self.of_folder(into).makes(what)
    }

    pub fn may_copy(&self, state: &State, kept: &Kept) -> Result<(), Ruled> {
        self.of_doc(state, kept).copies()
    }

    /// AGENTS takes in from outside only an agent's own file, one with its `agent:` card.
    pub fn may_lodge(
        &self,
        state: &State,
        kept: &Kept,
        into: Place,
        agents_own: bool,
    ) -> Result<(), Ruled> {
        let from = self.of_doc(state, kept);
        match into == Place::Agents && from != Place::Agents && !agents_own {
            true => Err(Ruled("agentsHoldOnlyRules")),
            false => Ok(()),
        }
    }

    /// A folder of AGENTS stays inside it, and only one already its own moves in; AGENTS itself moves only among the top folders.
    pub fn may_shelve(
        &self,
        state: &State,
        folder: FolderId,
        into: Option<FolderId>,
    ) -> Result<(), Ruled> {
        let root = crate::agents::roots(state).contains(&folder);
        match (self.of_folder(Some(folder)), self.of_folder(into)) {
            (Place::Agents, Place::Agents) => Ok(()),
            (Place::Agents, _) if root && into.is_none() => Ok(()),
            (Place::Agents, _) => Err(Ruled("agentsKeepTheirFolders")),
            _ => self.may_make(into, Making::Folder),
        }
    }
}

#[cfg(test)]
#[path = "places_test.rs"]
mod tests;
