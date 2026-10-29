use std::cell::RefCell;
use std::collections::HashMap;

use crate::store::{Doc, Fault, Kept, Saved, Store};

/// The store for tests and for running the server on one machine; nothing survives it.
#[derive(Debug, Default)]
pub struct Memory {
    docs: RefCell<HashMap<String, Doc>>,
    envelopes: RefCell<HashMap<(String, Kept), Vec<u8>>>,
}

impl Memory {
    pub fn envelopes(&self) -> usize {
        self.envelopes.borrow().len()
    }
}

impl Store for Memory {
    async fn doc(&self, id: &str) -> Result<Option<Doc>, Fault> {
        Ok(self.docs.borrow().get(id).cloned())
    }

    async fn save(&self, doc: &Doc, revision: Option<u64>) -> Result<Saved, Fault> {
        let mut docs = self.docs.borrow_mut();
        if docs.get(&doc.id).map(|held| held.revision) != revision {
            return Ok(Saved::Raced);
        }
        docs.insert(doc.id.clone(), doc.clone());
        Ok(Saved::Done)
    }

    async fn forget(&self, id: &str) -> Result<(), Fault> {
        self.docs.borrow_mut().remove(id);
        self.envelopes.borrow_mut().retain(|(of, _), _| of != id);
        Ok(())
    }

    async fn envelope(&self, id: &str, kept: &Kept) -> Result<Option<Vec<u8>>, Fault> {
        Ok(self
            .envelopes
            .borrow()
            .get(&(id.to_owned(), *kept))
            .cloned())
    }

    async fn keep(&self, id: &str, kept: &Kept, bytes: &[u8]) -> Result<(), Fault> {
        self.envelopes
            .borrow_mut()
            .insert((id.to_owned(), *kept), bytes.to_vec());
        Ok(())
    }

    async fn drop_envelope(&self, id: &str, kept: &Kept) -> Result<(), Fault> {
        self.envelopes.borrow_mut().remove(&(id.to_owned(), *kept));
        Ok(())
    }

    async fn ending(&self, now: i64) -> Result<Vec<String>, Fault> {
        let mut ids: Vec<String> = self
            .docs
            .borrow()
            .values()
            .filter(|doc| doc.ends < now)
            .map(|doc| doc.id.clone())
            .collect();
        ids.sort();
        Ok(ids)
    }
}
