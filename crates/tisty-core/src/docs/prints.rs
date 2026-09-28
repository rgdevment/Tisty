use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Told {
    weighs: u64,
    at: i64,
    print: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prints(BTreeMap<String, Told>);

fn ledger(aside: &Path) -> PathBuf {
    aside.join("prints.json")
}

fn stamped(at: &Path) -> Option<(u64, i64)> {
    let told = std::fs::metadata(at).ok()?;
    let when = told
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some((told.len(), when.as_nanos() as i64))
}

impl Prints {
    pub fn read(aside: &Path) -> Self {
        std::fs::read_to_string(ledger(aside))
            .ok()
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, aside: &Path) {
        if std::fs::create_dir_all(aside).is_err() {
            return;
        }
        if let Ok(said) = serde_json::to_string(self) {
            let _ = crate::store::write_atomic(&ledger(aside), said.as_bytes());
        }
    }

    pub fn of(&mut self, at: &Path) -> std::io::Result<Option<String>> {
        let named = at.to_string_lossy().into_owned();
        let Some((weighs, when)) = stamped(at) else {
            self.0.remove(&named);
            return super::print_of(at);
        };
        if let Some(told) = self.0.get(&named)
            && told.weighs == weighs
            && told.at == when
        {
            return Ok(Some(told.print.clone()));
        }
        let print = super::print_of(at)?;
        match &print {
            Some(print) => self.0.insert(
                named,
                Told {
                    weighs,
                    at: when,
                    print: print.clone(),
                },
            ),
            None => self.0.remove(&named),
        };
        Ok(print)
    }
}
