use std::io::Write;
use std::path::Path;
use std::sync::mpsc;

use crate::{Changes, Expect, Hitch, Limits, Remote, Seen, Watch};

const CHUNKS_WAITING: usize = 4;

fn on_a_runtime() -> bool {
    tokio::runtime::Handle::try_current().is_ok()
}

// Blocking reqwest refuses to run on an async runtime's thread, and a window command may be one.
pub fn away<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    if !on_a_runtime() {
        return work();
    }
    std::thread::scope(|scope| match scope.spawn(work).join() {
        Ok(done) => done,
        Err(panicked) => std::panic::resume_unwind(panicked),
    })
}

/// A provider whose every request runs off the async runtime, wherever it is asked from.
pub struct OffRuntime<R>(pub R);

struct Piped(mpsc::SyncSender<Vec<u8>>);

impl Write for Piped {
    fn write(&mut self, said: &[u8]) -> std::io::Result<usize> {
        self.0
            .send(said.to_vec())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
        Ok(said.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<R: Remote> Remote for OffRuntime<R> {
    fn limits(&self) -> Limits {
        self.0.limits()
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        away(|| self.0.list(under))
    }

    // The bytes come over a short channel, so a file of gigabytes never sits whole in memory.
    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        if !on_a_runtime() {
            return self.0.fetch(name, from, into);
        }
        let (sent, came) = mpsc::sync_channel(CHUNKS_WAITING);
        std::thread::scope(|scope| {
            let fetching = scope.spawn(move || self.0.fetch(name, from, &mut Piped(sent)));
            let mut refused = None;
            for chunk in came {
                if let Err(e) = into.write_all(&chunk) {
                    refused = Some(Hitch::Broke(e.to_string()));
                    break;
                }
            }
            let fetched = match fetching.join() {
                Ok(done) => done,
                Err(panicked) => std::panic::resume_unwind(panicked),
            };
            match refused {
                Some(hitch) => Err(hitch),
                None => fetched,
            }
        })
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        away(|| self.0.put(name, from, expect))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        away(|| self.0.delete(name, expect))
    }

    fn renamed(&self, from: &str, to: &str) -> Result<(), Hitch> {
        away(|| self.0.renamed(from, to))
    }

    fn moved_on(&self) {
        self.0.moved_on();
    }

    fn root(&self) -> &str {
        self.0.root()
    }

    fn mark_up(&self) -> Result<(), Hitch> {
        away(|| self.0.mark_up())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.0.hash_of(local)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        away(|| self.0.about(name))
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        away(|| self.0.changes(since))
    }

    fn hears(&self, since: &str) -> Option<Box<dyn Watch>> {
        self.0.hears(since)
    }

    fn lends(&self, name: &str) -> Result<Option<String>, Hitch> {
        away(|| self.0.lends(name))
    }

    fn spent_beyond(&self) -> u64 {
        self.0.spent_beyond()
    }

    fn room(&self) -> Result<Option<u64>, Hitch> {
        away(|| self.0.room())
    }
}

#[cfg(test)]
#[path = "off_test.rs"]
mod tests;
