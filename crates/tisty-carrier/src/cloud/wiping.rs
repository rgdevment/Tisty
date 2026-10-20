use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;

use crate::{Hitch, Remote, Seen};

// One at a time, each file's several round trips made emptying a cloud take half an hour.
const AT_ONCE: usize = 4;

/// Deletes every one, telling each name as it goes, and stops at the first that would not go.
pub(super) fn deleted_all(
    remote: &dyn Remote,
    all: &[&Seen],
    gone: &mut dyn FnMut(&str),
    once: impl Fn(&dyn Remote, &Seen) -> Result<(), Hitch> + Sync,
) -> Result<(), Hitch> {
    let Some((first, rest)) = all.split_first() else {
        return Ok(());
    };
    // The first goes alone, so whatever it makes on the way, a bin, is made once and not by each worker.
    once(remote, first)?;
    gone(&first.name);
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let (tell, told) = mpsc::channel::<Result<&str, Hitch>>();
    std::thread::scope(|scope| {
        for _ in 0..AT_ONCE.min(rest.len()) {
            let tell = tell.clone();
            let (next, failed, once) = (&next, &failed, &once);
            scope.spawn(move || {
                while !failed.load(Ordering::Relaxed) {
                    let Some(one) = rest.get(next.fetch_add(1, Ordering::Relaxed)) else {
                        break;
                    };
                    let done = once(remote, one).map(|()| one.name.as_str());
                    failed.fetch_or(done.is_err(), Ordering::Relaxed);
                    if tell.send(done).is_err() {
                        break;
                    }
                }
            });
        }
        drop(tell);
        let mut ended = Ok(());
        for done in told {
            match done {
                Ok(name) => gone(name),
                Err(hitch) if ended.is_ok() => ended = Err(hitch),
                Err(_) => {}
            }
        }
        ended
    })
}
