#![cfg_attr(not(windows), allow(dead_code))]

/// A shop that answers it has nothing is not a shop that never answered: the first means this copy
/// is current, the second that nothing is known and the old wording still stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    Waiting(String),
    /// Already on this machine, staged beside the copy in use, and it takes over when this one
    /// closes. Nothing left to download, so nothing to offer but the closing.
    Landed(String),
    /// The Store has it in its own queue — getting it, paused, or waiting its turn. It says
    /// nothing about updates then, because from where it stands there is nothing left to find.
    Queued,
    Current,
    Silent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    Gone,
    Stopped,
    Failed(String),
}

/// The Store counts the download as the first four fifths of the errand and the install as the
/// rest, so what it hands over is stretched back into a download of its own.
const DOWNLOADED: f64 = 0.8;

pub fn step(far: f64) -> (&'static str, u64) {
    if far >= DOWNLOADED {
        return ("installing", 100);
    }
    ("getting", (far.max(0.0) / DOWNLOADED * 100.0) as u64)
}

pub fn asked(window: isize, forced: bool) -> Shelf {
    there::asked(window, forced)
}

pub fn take(
    window: isize,
    told: impl FnMut(&'static str, u64) + Send + 'static,
) -> Result<(), Trouble> {
    there::take(window, told)
}

#[cfg(windows)]
mod there {
    use super::{Shelf, Trouble};
    use tisty_core::witness::{self, Fact, channel};
    use windows::ApplicationModel::{Package, PackageSignatureKind, PackageVersion};
    use windows::Services::Store::{
        StoreContext, StorePackageUpdateState, StorePackageUpdateStatus, StoreQueueItemState,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::CoIncrementMTAUsage;
    use windows::Win32::UI::Shell::IInitializeWithWindow;
    use windows::core::{Interface, Ref};
    use windows_future::AsyncOperationProgressHandler;

    const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
    /// Asking wakes the Store's own errand, and its answer is the state from before it looked:
    /// at 15:06 it says nothing and at 15:07 it is installing. A person who pressed the button
    /// is owed the answer that comes out of that errand, not the one that preceded it.
    const AGAIN: std::time::Duration = std::time::Duration::from_secs(6);
    const UNTIL: std::time::Duration = std::time::Duration::from_secs(75);
    /// Long past the point where the process should already have been taken down by the install.
    const AT_LENGTH: std::time::Duration = std::time::Duration::from_secs(30 * 60);

    /// The Store answers on a thread of its own and has been known to never answer at all, which
    /// would otherwise leave the window waiting on it for the rest of the session.
    fn apart<T: Send + 'static>(
        how_long: std::time::Duration,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (tell, hear) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let _ = tell.send(work());
        });
        hear.recv_timeout(how_long).ok()
    }

    /// A copy the Store did not sell is one the Store will never have an update for, and its
    /// empty answer reads exactly like «you are on the newest one». Left as that, a package
    /// installed by hand would be told it was current for the rest of its life.
    fn sold_here() -> bool {
        Package::Current()
            .and_then(|one| one.SignatureKind())
            .is_ok_and(|kind| kind == PackageSignatureKind::Store)
    }

    pub fn asked(window: isize, forced: bool) -> Shelf {
        if !sold_here() {
            witness::warn(
                channel::WINDOW,
                "this package did not come from the Store, so the Store answers for nothing in it",
                &[],
            );
            return Shelf::Silent;
        }
        if let Some(version) = beside_us() {
            return Shelf::Landed(version);
        }
        match apart(PATIENCE, move || waiting(window)) {
            Some(Ok(Some(version))) => Shelf::Waiting(version),
            Some(Ok(None)) if forced => waited_out(window),
            Some(Ok(None)) => in_its_queue(window),
            Some(Err(why)) => {
                witness::warn(
                    channel::WINDOW,
                    "the Store was asked what it has and refused",
                    &[("why", Fact::Why(why.message()))],
                );
                Shelf::Silent
            }
            None => {
                witness::warn(
                    channel::WINDOW,
                    "the Store was asked what it has and never answered",
                    &[("waited", Fact::Count(PATIENCE.as_secs() as usize))],
                );
                Shelf::Silent
            }
        }
    }

    /// The Store was just asked and said nothing, which is what it says while its own errand is
    /// still running. Waiting it out turns «nothing for you» into what it actually found.
    fn waited_out(window: isize) -> Shelf {
        let since = std::time::Instant::now();
        while since.elapsed() < UNTIL {
            std::thread::sleep(AGAIN);
            if let Some(version) = beside_us() {
                witness::note(
                    channel::WINDOW,
                    "the Store brought the update down while the person waited on the answer",
                    &[("version", Fact::Id(version.clone()))],
                );
                return Shelf::Landed(version);
            }
            if let Some(Ok(Some(version))) = apart(PATIENCE, move || waiting(window)) {
                witness::note(
                    channel::WINDOW,
                    "the Store named an update only after its own errand had run",
                    &[
                        ("version", Fact::Id(version.clone())),
                        ("waited", Fact::Count(since.elapsed().as_secs() as usize)),
                    ],
                );
                return Shelf::Waiting(version);
            }
        }
        in_its_queue(window)
    }

    /// What the Store has taken on itself: bringing it down, paused, or waiting its turn. While
    /// an errand of its own is in the queue, the update it is about is no longer an update it
    /// offers — asking what it has is asking the one place that has stopped counting it.
    fn in_its_queue(window: isize) -> Shelf {
        let queued = apart(PATIENCE, move || queued(window));
        match queued {
            Some(Ok(true)) => {
                witness::note(
                    channel::WINDOW,
                    "the Store names no update because it already has one in its own queue",
                    &[],
                );
                Shelf::Queued
            }
            Some(Ok(false)) => Shelf::Current,
            Some(Err(why)) => {
                witness::warn(
                    channel::WINDOW,
                    "the Store was asked what it is already getting and refused",
                    &[("why", Fact::Why(why.message()))],
                );
                Shelf::Current
            }
            None => {
                witness::warn(
                    channel::WINDOW,
                    "the Store was asked what it is already getting and never answered",
                    &[("waited", Fact::Count(PATIENCE.as_secs() as usize))],
                );
                Shelf::Current
            }
        }
    }

    /// Anything of ours in the Store's queue that has not finished: it will land on its own, and
    /// what is left to say is that it is coming.
    fn queued(window: isize) -> windows::core::Result<bool> {
        let items = shop(window)?.GetAssociatedStoreQueueItemsAsync()?.join()?;
        for one in 0..items.Size()? {
            let item = items.GetAt(one)?;
            let state = item.GetCurrentStatus()?.PackageInstallState()?;
            let coming = matches!(
                state,
                StoreQueueItemState::Active | StoreQueueItemState::Paused
            );
            witness::note(
                channel::WINDOW,
                "the Store has something of ours in its queue",
                &[
                    ("id", Fact::Id(item.ProductId()?.to_string())),
                    ("state", Fact::Count(state.0 as usize)),
                ],
            );
            if coming {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// A newer package of our own family already registered for this user is one the Store has
    /// finished bringing down: it cannot replace a copy that is running, so it waits for the
    /// window to close. The Store's own answer never mentions it — there is nothing left to get.
    fn beside_us() -> Option<String> {
        let ours = Package::Current().ok()?;
        let family = ours.Id().ok()?.FamilyName().ok()?;
        let here = numbered(&ours.Id().ok()?.Version().ok()?);
        let shelf = windows::Management::Deployment::PackageManager::new().ok()?;
        let mut newest: Option<semver::Version> = None;
        let mine: semver::Version = here.parse().ok()?;
        for one in shelf.FindPackagesByPackageFamilyName(&family).ok()? {
            let Ok(id) = one.Id() else { continue };
            let Ok(version) = id.Version() else { continue };
            let Ok(said) = numbered(&version).parse::<semver::Version>() else {
                continue;
            };
            if said > mine && newest.as_ref().is_none_or(|had| said > *had) {
                newest = Some(said);
            }
        }
        newest.map(|one| one.to_string())
    }

    /// The errand covers optional packages too, and the version of one of those is not a version
    /// of Tisty to put in front of anybody.
    fn waiting(window: isize) -> windows::core::Result<Option<String>> {
        let updates = shop(window)?
            .GetAppAndOptionalStorePackageUpdatesAsync()?
            .join()?;
        let ours = Package::Current()?.Id()?.FamilyName()?;
        let mut offered: Vec<String> = Vec::new();
        for one in 0..updates.Size()? {
            let id = updates.GetAt(one)?.Package()?.Id()?;
            let name = id.FamilyName()?;
            if name == ours {
                return Ok(Some(numbered(&id.Version()?)));
            }
            offered.push(name.to_string());
        }
        // The Store answers from what it last knew unless it is due to ask again, so an empty
        // answer here is either «nothing for you» or «I did not look», and they read the same.
        witness::note(
            channel::WINDOW,
            "the Store was asked what it has and named nothing for this package",
            &[
                ("ours", Fact::Id(ours.to_string())),
                ("offered", Fact::Count(offered.len())),
                (
                    "names",
                    Fact::Why(if offered.is_empty() {
                        "nothing at all".to_string()
                    } else {
                        offered.join(", ")
                    }),
                ),
            ],
        );
        Ok(None)
    }

    pub fn take(
        window: isize,
        told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        apart(AT_LENGTH, move || taken(window, told)).unwrap_or_else(|| {
            Err(Trouble::Failed(format!(
                "the Store was left to it for {} minutes and never came back",
                AT_LENGTH.as_secs() / 60
            )))
        })
    }

    fn taken(
        window: isize,
        told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        let shop = shop(window).map_err(sour)?;
        let updates = shop
            .GetAppAndOptionalStorePackageUpdatesAsync()
            .and_then(|asking| asking.join())
            .map_err(sour)?;
        if updates.Size().map_err(sour)? == 0 {
            return Err(Trouble::Gone);
        }

        let asking = shop
            .RequestDownloadAndInstallStorePackageUpdatesAsync(&updates)
            .map_err(sour)?;
        let told = std::sync::Mutex::new(told);
        asking
            .SetProgress(&AsyncOperationProgressHandler::new(
                move |_, far: Ref<'_, StorePackageUpdateStatus>| {
                    let (stage, how) = super::step(far.ok()?.PackageDownloadProgress);
                    let mut told = told.lock().unwrap_or_else(|e| e.into_inner());
                    told(stage, how);
                    Ok(())
                },
            ))
            .map_err(sour)?;

        // Windows takes the process with it to put the new package in place, so on the ordinary
        // path nothing below this line is ever reached.
        let done = asking.join().map_err(sour)?;
        match done.OverallState().map_err(sour)? {
            StorePackageUpdateState::Completed => Ok(()),
            StorePackageUpdateState::Canceled => Err(Trouble::Stopped),
            other => Err(Trouble::Failed(format!("{other:?}"))),
        }
    }

    fn shop(window: isize) -> windows::core::Result<StoreContext> {
        apartment();
        let shop = StoreContext::GetDefault()?;
        owned(&shop, window)?;
        Ok(shop)
    }

    /// The Store raises dialogs of its own and refuses with ERROR_INVALID_WINDOW_HANDLE unless it
    /// is told which window owns them.
    #[allow(unsafe_code)]
    fn owned(shop: &StoreContext, window: isize) -> windows::core::Result<()> {
        let owner: IInitializeWithWindow = shop.cast()?;
        unsafe { owner.Initialize(HWND(window as *mut core::ffi::c_void)) }
    }

    /// A worker thread of the async runtime belongs to no apartment, and the activation fails there
    /// before it begins.
    #[allow(unsafe_code)]
    fn apartment() {
        static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        ONCE.get_or_init(|| unsafe {
            let _ = CoIncrementMTAUsage();
        });
    }

    fn numbered(version: &PackageVersion) -> String {
        format!("{}.{}.{}", version.Major, version.Minor, version.Build)
    }

    fn sour(why: windows::core::Error) -> Trouble {
        Trouble::Failed(why.message())
    }
}

#[cfg(not(windows))]
mod there {
    use super::{Shelf, Trouble};

    pub fn asked(_window: isize, _forced: bool) -> Shelf {
        Shelf::Silent
    }

    pub fn take(
        _window: isize,
        _told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        Err(Trouble::Gone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_download_fills_the_bar_on_its_own() {
        assert_eq!(step(0.0), ("getting", 0));
        assert_eq!(step(0.4), ("getting", 50));
        assert_eq!(step(0.79), ("getting", 98));
    }

    #[test]
    fn what_the_store_counts_as_the_install_is_shown_as_installing() {
        assert_eq!(step(0.8), ("installing", 100));
        assert_eq!(step(0.9), ("installing", 100));
        assert_eq!(step(1.0), ("installing", 100));
    }

    #[test]
    fn a_figure_that_makes_no_sense_still_leaves_a_bar_that_does() {
        assert_eq!(step(-1.0), ("getting", 0));
        assert_eq!(step(2.0), ("installing", 100));
    }
}
