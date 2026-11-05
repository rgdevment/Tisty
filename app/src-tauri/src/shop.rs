#![cfg_attr(not(windows), allow(dead_code))]

use crate::update::{self, Ahead, Coming, Ready};

/// A shop that answers it has nothing is not a shop that never answered: the first means this copy
/// is current, the second that nothing is known and the old wording still stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    /// The offer carries no number: the package it names is the one installed here.
    Waiting,
    /// Already on this machine, staged beside the copy in use, and it takes over when this one
    /// closes. Nothing left to download, so nothing to offer but the closing.
    Landed(String),
    Queued,
    /// In its queue, stopped for a reason that quitting Tisty will not clear.
    Paused,
    Current,
    Silent,
    /// Installed by other means, so the Store will never have an update for it.
    NotSold,
}

/// What the last look wrote down, and whether the Store itself made the offer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub version: Option<String>,
    pub offered: Option<bool>,
}

/// What a look comes to: what the window is told and what stays written for the next one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub ready: Option<Ready>,
    pub memory: Memory,
    pub unanswered: bool,
}

impl Verdict {
    /// A look nobody pressed for has nobody to tell that the Store did not answer.
    pub fn refused(&self, pressed: bool) -> bool {
        pressed && self.unanswered
    }
}

/// Only a press is waited for, and not when the catalog says there is nothing to wait for.
pub fn waits(pressed: bool, ahead: &Ahead) -> bool {
    pressed && *ahead != Ahead::Nothing
}

/// An offer with no number is not kept: nothing would say when it stopped being true.
pub fn last_said(now: &str, was: &Memory) -> Option<Ready> {
    let version = was
        .version
        .as_deref()
        .filter(|one| update::later(now, one))?;
    Some(match was.offered? {
        true => update::from_the_shop(Some(version)),
        false => update::on_its_way(Some(version), Coming::NotYet),
    })
}

pub fn verdict(shelf: Shelf, ahead: Ahead, now: &str, was: Memory) -> Verdict {
    let number = match &ahead {
        Ahead::Is(version) => Some(version.clone()),
        Ahead::Nothing => None,
        Ahead::Unheard => was.version.clone().filter(|one| update::later(now, one)),
    };
    let told = |ready: Ready, version: Option<String>, offered: bool| Verdict {
        ready: Some(ready),
        memory: Memory {
            version,
            offered: Some(offered),
        },
        unanswered: false,
    };
    let none = |unanswered: bool| Verdict {
        ready: None,
        memory: Memory::default(),
        unanswered,
    };
    match (shelf, ahead) {
        (Shelf::Waiting, _) => told(update::from_the_shop(number.as_deref()), number, true),
        (Shelf::Landed(version), _) => {
            let landed = Ready {
                installs: false,
                ..update::from_the_shop(Some(&version))
            };
            told(landed, Some(version), true)
        }
        (Shelf::Queued, _) => told(
            update::on_its_way(number.as_deref(), Coming::Queued),
            number,
            false,
        ),
        (Shelf::Paused, _) => told(
            update::on_its_way(number.as_deref(), Coming::Paused),
            number,
            false,
        ),
        (Shelf::NotSold, _) => none(false),
        (_, Ahead::Is(version)) => told(
            update::on_its_way(Some(&version), Coming::NotYet),
            Some(version),
            false,
        ),
        (Shelf::Current, Ahead::Nothing) => none(false),
        // Nothing newer is listed, so of what was remembered only the Store's own offer can stand.
        (Shelf::Silent, Ahead::Nothing) if was.offered != Some(true) => none(true),
        // The Store just said it has no offer, so only a coming version it never spoke for stands.
        (Shelf::Current, Ahead::Unheard) if was.offered != Some(false) => none(false),
        (shelf, _) => Verdict {
            ready: last_said(now, &was),
            memory: was,
            unanswered: shelf == Shelf::Silent,
        },
    }
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

/// What can be told with no question asked of anyone, so nothing goes over the network for it.
pub fn settled() -> Option<Shelf> {
    there::settled()
}

pub fn sold() -> bool {
    there::sold_here()
}

pub fn asked(window: isize, waits: bool) -> Shelf {
    there::asked(window, waits)
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
        StoreContext, StorePackageUpdateState, StorePackageUpdateStatus,
        StoreQueueItemExtendedState, StoreQueueItemState,
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
    pub fn sold_here() -> bool {
        Package::Current()
            .and_then(|one| one.SignatureKind())
            .is_ok_and(|kind| kind == PackageSignatureKind::Store)
    }

    pub fn settled() -> Option<Shelf> {
        if !sold_here() {
            witness::warn(
                channel::UPDATE,
                "this package did not come from the Store, so the Store answers for nothing in it",
                &[],
            );
            return Some(Shelf::NotSold);
        }
        beside_us().map(Shelf::Landed)
    }

    pub fn asked(window: isize, waits: bool) -> Shelf {
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) if waits => waited_out(window),
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn offers(window: isize) -> Option<bool> {
        match apart(PATIENCE, move || waiting(window)) {
            Some(Ok(offers)) => Some(offers),
            Some(Err(why)) => {
                witness::warn(
                    channel::UPDATE,
                    "the Store was asked what it has and refused",
                    &[("why", Fact::Why(why.message()))],
                );
                None
            }
            None => {
                witness::warn(
                    channel::UPDATE,
                    "the Store was asked what it has and never answered",
                    &[("waited", Fact::Count(PATIENCE.as_secs() as usize))],
                );
                None
            }
        }
    }

    /// The Store rations its looks and then repeats itself, so the wait watches what costs none.
    fn waited_out(window: isize) -> Shelf {
        let since = std::time::Instant::now();
        while since.elapsed() < UNTIL {
            std::thread::sleep(AGAIN);
            if let Some(version) = beside_us() {
                witness::note(
                    channel::UPDATE,
                    "the Store brought the update down while the person waited on the answer",
                    &[("version", Fact::Id(version.clone()))],
                );
                return Shelf::Landed(version);
            }
            if let Some(Ok(Some(found))) = apart(PATIENCE, move || queued(window)) {
                return found;
            }
        }
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn in_its_queue(window: isize) -> Shelf {
        let queued = apart(PATIENCE, move || queued(window));
        match queued {
            Some(Ok(Some(found))) => {
                witness::note(
                    channel::UPDATE,
                    "the Store names no update because it already has one in its own queue",
                    &[],
                );
                found
            }
            Some(Ok(None)) => Shelf::Current,
            Some(Err(why)) => {
                witness::warn(
                    channel::UPDATE,
                    "the Store was asked what it is already getting and refused",
                    &[("why", Fact::Why(why.message()))],
                );
                Shelf::Silent
            }
            None => {
                witness::warn(
                    channel::UPDATE,
                    "the Store was asked what it is already getting and never answered",
                    &[("waited", Fact::Count(PATIENCE.as_secs() as usize))],
                );
                Shelf::Silent
            }
        }
    }

    fn queued(window: isize) -> windows::core::Result<Option<Shelf>> {
        let ours = Package::Current()?.Id()?.FamilyName()?;
        let items = shop(window)?.GetAssociatedStoreQueueItemsAsync()?.join()?;
        witness::standing(
            channel::UPDATE,
            "the Store's queue was read",
            &[("items", Fact::Count(items.Size()? as usize))],
        );
        for one in 0..items.Size()? {
            let item = items.GetAt(one)?;
            if item.PackageFamilyName().unwrap_or_default() != ours {
                continue;
            }
            let status = item.GetCurrentStatus()?;
            let state = status.PackageInstallState()?;
            if matches!(
                state,
                StoreQueueItemState::Active | StoreQueueItemState::Paused
            ) {
                let reason = status.PackageInstallExtendedState().unwrap_or_default();
                witness::note(
                    channel::UPDATE,
                    "the Store has one of ours in its own queue",
                    &[
                        ("state", Fact::Count(state.0 as usize)),
                        ("reason", Fact::Count(reason.0 as usize)),
                    ],
                );
                return Ok(Some(of_the_queue(state, reason)));
            }
        }
        Ok(None)
    }

    /// Two reasons only wait for Tisty to stop running, as an update on its way does.
    pub(super) fn of_the_queue(
        state: StoreQueueItemState,
        reason: StoreQueueItemExtendedState,
    ) -> Shelf {
        let waits_for_us = matches!(
            reason,
            StoreQueueItemExtendedState::PausedPackagesInUse
                | StoreQueueItemExtendedState::PausedReadyToInstall
        );
        if state == StoreQueueItemState::Paused && !waits_for_us {
            Shelf::Paused
        } else {
            Shelf::Queued
        }
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
        let mine_only = windows::core::HSTRING::new();
        for one in shelf
            .FindPackagesByUserSecurityIdPackageFamilyName(&mine_only, &family)
            .ok()?
        {
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

    /// The errand covers optional packages too, and an update to one of those is not one to Tisty.
    fn waiting(window: isize) -> windows::core::Result<bool> {
        let updates = shop(window)?
            .GetAppAndOptionalStorePackageUpdatesAsync()?
            .join()?;
        let ours = Package::Current()?.Id()?.FamilyName()?;
        let mut offered: Vec<String> = Vec::new();
        for one in 0..updates.Size()? {
            let id = updates.GetAt(one)?.Package()?.Id()?;
            let name = id.FamilyName()?;
            if name == ours {
                witness::note(
                    channel::UPDATE,
                    "the Store offers an update for this package",
                    &[("named", Fact::Id(numbered(&id.Version()?)))],
                );
                return Ok(true);
            }
            offered.push(name.to_string());
        }
        // The Store answers from what it last knew unless it is due to ask again, so an empty
        // answer here is either «nothing for you» or «I did not look», and they read the same.
        witness::note(
            channel::UPDATE,
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
        Ok(false)
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

    pub fn settled() -> Option<Shelf> {
        None
    }

    pub fn sold_here() -> bool {
        false
    }

    pub fn asked(_window: isize, _waits: bool) -> Shelf {
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
#[path = "shop_test.rs"]
mod tests;
