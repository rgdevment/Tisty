use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};
use tisty_core::herald::{Due, Happening};
use tisty_core::witness::{self, Fact, channel};

// Windows lets a booked toast go once the machine was off five minutes past its hour.
pub const FRESH: jiff::SignedDuration = jiff::SignedDuration::from_mins(5);

// The system can drop what it held, so a settled plan is checked against it again after a while.
const RECHECK: jiff::SignedDuration = jiff::SignedDuration::from_mins(10);

const MARK: &str = "tisty-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ring {
    pub task: String,
    pub at: jiff::Timestamp,
    pub heading: String,
    pub title: String,
    pub heard: bool,
    pub urgent: bool,
}

impl Ring {
    pub fn key(&self) -> String {
        use sha2::Digest;
        let digest = sha2::Sha256::new()
            .chain_update(self.at.as_second().to_le_bytes())
            .chain_update(self.heading.as_bytes())
            .chain_update([0])
            .chain_update(self.title.as_bytes())
            .chain_update([u8::from(self.heard), u8::from(self.urgent)])
            .finalize();
        let short: String = digest
            .iter()
            .take(12)
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!("{MARK}{short}-{}", self.task)
    }
}

pub fn ours(key: &str) -> bool {
    key.starts_with(MARK)
}

#[cfg(any(target_os = "macos", test))]
pub fn task_of(key: &str) -> Option<&str> {
    let (_, task) = key.strip_prefix(MARK)?.split_once('-')?;
    (!task.is_empty()).then_some(task)
}

pub fn rings(coming: Vec<Due>, heading: &str, heard: bool, urgent: bool) -> Vec<Ring> {
    let mut seen = HashSet::new();
    coming
        .into_iter()
        .filter(|one| match &one.what {
            Happening::Due { task, .. } => seen.insert((task.clone(), one.at)),
            _ => true,
        })
        .filter_map(|one| match one.what {
            Happening::Due { title, task } => Some(Ring {
                task,
                at: one.at,
                heading: heading.to_string(),
                title,
                heard,
                urgent,
            }),
            _ => None,
        })
        .collect()
}

pub trait Book: Send + Sync {
    fn shows(&self) -> bool;

    fn pending(&self) -> Option<Vec<String>>;

    fn cancel(&self, keys: &[String]);

    fn add(&self, ring: &Ring) -> bool;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Reconciled {
    pub held: Vec<Ring>,
    pub whole: bool,
    pub refused: usize,
}

// Asking whether the system shows anything is what makes macOS ask the person, so it waits for a first reminder.
pub fn reconciled(book: &dyn Book, wanted: &[Ring]) -> Reconciled {
    if !wanted.is_empty() && !book.shows() {
        return Reconciled::default();
    }
    let Some(pending) = book.pending() else {
        return Reconciled::default();
    };
    let pending: HashSet<String> = pending.into_iter().filter(|key| ours(key)).collect();
    let keys: Vec<String> = wanted.iter().map(Ring::key).collect();
    let stale: Vec<String> = pending
        .iter()
        .filter(|key| !keys.contains(key))
        .cloned()
        .collect();
    if !stale.is_empty() {
        book.cancel(&stale);
    }
    let mut done = Reconciled {
        held: Vec::new(),
        whole: true,
        refused: 0,
    };
    for (ring, key) in wanted.iter().zip(&keys) {
        if pending.contains(key) || book.add(ring) {
            done.held.push(ring.clone());
        } else {
            done.whole = false;
            done.refused += 1;
        }
    }
    done
}

#[derive(Default)]
struct Held {
    settled: Option<(Vec<String>, jiff::Timestamp)>,
    warned: Option<Vec<String>>,
    rings: Vec<Ring>,
}

pub struct Booking {
    book: Option<Box<dyn Book>>,
    most: usize,
    urgent: bool,
    held: Mutex<Held>,
}

impl Booking {
    pub fn new(book: Option<Box<dyn Book>>, most: usize) -> Self {
        Self {
            book,
            most,
            urgent: false,
            held: Mutex::new(Held::default()),
        }
    }

    pub fn here(app: &tauri::AppHandle) -> Self {
        let book = match crate::waking::off_the_machine() {
            true => None,
            false => there::book(),
        };
        if book.is_some() {
            there::listen(app);
        }
        Self {
            urgent: there::urgent(),
            ..Self::new(book, there::MOST)
        }
    }

    /// Whether this system can let a reminder through «Do not disturb» at all.
    pub fn urgent(&self) -> bool {
        self.urgent
    }

    pub fn books(&self) -> bool {
        self.book.is_some()
    }

    pub fn most(&self) -> usize {
        self.most
    }

    pub fn keep(&self, wanted: &[Ring], now: jiff::Timestamp) {
        let Some(book) = self.book.as_deref() else {
            return;
        };
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let keys: Vec<String> = wanted.iter().map(Ring::key).collect();
        let fresh = |(settled, at): &(Vec<String>, jiff::Timestamp)| {
            *settled == keys && now.duration_since(*at) < RECHECK
        };
        if held.settled.as_ref().is_some_and(fresh) {
            return;
        }
        let done = reconciled(book, wanted);
        if done.refused > 0 && held.warned.as_ref() != Some(&keys) {
            witness::warn(
                channel::HERALD,
                "the system did not take every reminder, so the window rings the rest",
                &[
                    ("asked", Fact::Count(wanted.len())),
                    ("refused", Fact::Count(done.refused)),
                ],
            );
            held.warned = Some(keys.clone());
        }
        let mut rings: Vec<Ring> = std::mem::take(&mut held.rings)
            .into_iter()
            .filter(|one| one.at <= now && now.duration_since(one.at) <= FRESH)
            .collect();
        rings.extend(done.held);
        held.rings = rings;
        held.settled = done.whole.then_some((keys, now));
    }

    /// After `forget` emptied the system, nothing is held, and a Tisty still open books afresh.
    pub fn forgotten(&self) {
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        held.settled = None;
        held.rings.clear();
    }

    pub fn holds(&self, due: &Due, now: jiff::Timestamp) -> bool {
        let Happening::Due { task, .. } = &due.what else {
            return false;
        };
        if now.duration_since(due.at) > FRESH {
            return false;
        }
        self.held
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .rings
            .iter()
            .any(|one| one.task == *task && one.at == due.at)
    }
}

pub fn settings_page() -> Option<&'static str> {
    there::PAGE
}

pub fn forget() {
    if crate::waking::off_the_machine() {
        return;
    }
    let Some(book) = there::book() else {
        return;
    };
    if let Some(keys) = book.pending() {
        book.cancel(&keys);
    }
}

#[cfg(windows)]
mod there {
    use super::{Book, Ring};
    use tisty_core::witness::{self, Fact, channel};
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::DateTime;
    use windows::UI::Notifications::{
        NotificationSetting, ScheduledToastNotification, ToastNotificationManager, ToastNotifier,
    };
    use windows::core::HSTRING;

    pub const MOST: usize = 256;

    const GROUP: &str = "reminders";

    pub const NAMED: &str = "dev.rgdevment.tisty";

    pub const PAGE: Option<&str> = Some("ms-settings:notifications");

    // Microsoft lets an important toast through «Do not disturb» from this build on.
    const URGENT_SINCE: u32 = 22_546;

    pub fn urgent() -> bool {
        build().is_some_and(urgent_on)
    }

    pub fn urgent_on(build: u32) -> bool {
        build >= URGENT_SINCE
    }

    fn build() -> Option<u32> {
        winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            .ok()?
            .get_value::<String, _>("CurrentBuildNumber")
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    pub fn book() -> Option<Box<dyn Book>> {
        joined();
        Some(Box::new(Toasts {
            packaged: crate::update::route().route == crate::update::Route::Store,
        }))
    }

    pub fn listen(_app: &tauri::AppHandle) {}

    // A thread that never joined COM still reaches WinRT while the process keeps its shared apartment alive.
    #[allow(unsafe_code)]
    pub fn joined() {
        let _ = unsafe { windows::Win32::System::Com::CoIncrementMTAUsage() };
    }

    struct Toasts {
        packaged: bool,
    }

    type Scheduled = Vec<(ScheduledToastNotification, String)>;

    impl Toasts {
        fn notifier(&self) -> windows::core::Result<ToastNotifier> {
            if self.packaged {
                return ToastNotificationManager::CreateToastNotifier();
            }
            ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(NAMED))
        }

        fn ours(&self) -> windows::core::Result<(ToastNotifier, Scheduled)> {
            let notifier = self.notifier()?;
            let ours = notifier
                .GetScheduledToastNotifications()?
                .into_iter()
                .filter(|one| one.Group().is_ok_and(|group| group == GROUP))
                .filter_map(|one| {
                    let tag = one.Tag().ok()?.to_string();
                    Some((one, tag))
                })
                .collect();
            Ok((notifier, ours))
        }
    }

    impl Book for Toasts {
        fn shows(&self) -> bool {
            self.notifier()
                .and_then(|notifier| notifier.Setting())
                .is_ok_and(|setting| setting == NotificationSetting::Enabled)
        }

        fn pending(&self) -> Option<Vec<String>> {
            let (_, ours) = self.ours().ok()?;
            Some(ours.into_iter().map(|(_, tag)| tag).collect())
        }

        fn cancel(&self, keys: &[String]) {
            let Ok((notifier, ours)) = self.ours() else {
                return;
            };
            for (one, tag) in ours {
                if keys.contains(&tag) {
                    let _ = notifier.RemoveFromSchedule(&one);
                }
            }
        }

        fn add(&self, ring: &Ring) -> bool {
            self.notifier()
                .and_then(|notifier| booked(&notifier, ring))
                .inspect_err(|why| {
                    witness::warn(
                        channel::HERALD,
                        "Windows would not book a reminder",
                        &[("why", Fact::Why(why.message()))],
                    );
                })
                .is_ok()
        }
    }

    fn booked(notifier: &ToastNotifier, ring: &Ring) -> windows::core::Result<()> {
        let one = ScheduledToastNotification::CreateScheduledToastNotification(
            &page(ring)?,
            ticks(ring.at),
        )?;
        one.SetTag(&HSTRING::from(ring.key()))?;
        one.SetGroup(&HSTRING::from(GROUP))?;
        notifier.AddToSchedule(&one)
    }

    pub fn page(ring: &Ring) -> windows::core::Result<XmlDocument> {
        let page = XmlDocument::new()?;
        page.LoadXml(&HSTRING::from(toast(ring)))?;
        Ok(page)
    }

    pub fn toast(ring: &Ring) -> String {
        let quiet = match ring.heard {
            true => "",
            false => r#"<audio silent="true"/>"#,
        };
        let scenario = match ring.urgent {
            true => r#" scenario="urgent""#,
            false => "",
        };
        format!(
            r#"<toast launch="{}" activationType="protocol"{scenario}><visual><binding template="ToastGeneric"><text>{}</text><text>{}</text></binding></visual>{quiet}</toast>"#,
            escaped(&crate::viewing::task_link(&ring.task)),
            escaped(&ring.heading),
            escaped(&ring.title),
        )
    }

    fn escaped(text: &str) -> String {
        text.chars()
            .filter(|one| !one.is_control() || matches!(one, '\t' | '\n'))
            .fold(String::with_capacity(text.len()), |mut out, one| {
                match one {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    '\'' => out.push_str("&apos;"),
                    other => out.push(other),
                }
                out
            })
    }

    pub fn ticks(at: jiff::Timestamp) -> DateTime {
        const SINCE_1601: i64 = 11_644_473_600;
        DateTime {
            UniversalTime: (at.as_second() + SINCE_1601) * 10_000_000
                + i64::from(at.subsec_nanosecond()) / 100,
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod there {
    use super::{Book, Ring};
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject};
    use objc2::{AllocAnyThread, DefinedClass, define_class, msg_send};
    use objc2_foundation::{NSArray, NSDateComponents, NSError, NSString};
    use objc2_user_notifications::{
        UNAuthorizationOptions, UNAuthorizationStatus, UNCalendarNotificationTrigger,
        UNMutableNotificationContent, UNNotification, UNNotificationInterruptionLevel,
        UNNotificationPresentationOptions, UNNotificationRequest, UNNotificationResponse,
        UNNotificationSettings, UNNotificationSound, UNNotificationTrigger,
        UNUserNotificationCenter, UNUserNotificationCenterDelegate,
    };
    use std::ptr::NonNull;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    pub const MOST: usize = 64;

    pub const PAGE: Option<&str> =
        Some("x-apple.systempreferences:com.apple.Focus-Settings.extension");

    const WAIT: Duration = Duration::from_secs(2);

    // A copy signed to let reminders through carries the profile that grants it; without it macOS refuses the mark.
    pub fn urgent() -> bool {
        std::env::current_exe()
            .ok()
            .and_then(|exe| crate::waking::bundle_of(&exe))
            .is_some_and(|bundle| provisioned(&bundle))
    }

    pub fn provisioned(bundle: &std::path::Path) -> bool {
        bundle
            .join("Contents")
            .join("embedded.provisionprofile")
            .is_file()
    }

    // Outside a bundle the centre throws instead of answering.
    pub fn book() -> Option<Box<dyn Book>> {
        let exe = std::env::current_exe().ok()?;
        crate::waking::bundle_of(&exe)?;
        Some(Box::new(Center {
            asked: AtomicBool::new(false),
        }))
    }

    fn center() -> Retained<UNUserNotificationCenter> {
        UNUserNotificationCenter::currentNotificationCenter()
    }

    struct Center {
        asked: AtomicBool,
    }

    impl Book for Center {
        fn shows(&self) -> bool {
            let (told, heard) = mpsc::channel();
            let answer = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
                let settings = unsafe { settings.as_ref() };
                let _ = told.send(settings.authorizationStatus());
            });
            center().getNotificationSettingsWithCompletionHandler(&answer);
            match heard.recv_timeout(WAIT) {
                Ok(status) if status == UNAuthorizationStatus::NotDetermined => {
                    if !self.asked.swap(true, Ordering::Relaxed) {
                        ask();
                    }
                    false
                }
                Ok(status) => {
                    status == UNAuthorizationStatus::Authorized
                        || status == UNAuthorizationStatus::Provisional
                }
                Err(_) => false,
            }
        }

        fn pending(&self) -> Option<Vec<String>> {
            let (told, heard) = mpsc::channel();
            let answer = RcBlock::new(move |all: NonNull<NSArray<UNNotificationRequest>>| {
                let all = unsafe { all.as_ref() };
                let keys: Vec<String> =
                    all.iter().map(|one| one.identifier().to_string()).collect();
                let _ = told.send(keys);
            });
            center().getPendingNotificationRequestsWithCompletionHandler(&answer);
            heard.recv_timeout(WAIT).ok()
        }

        fn cancel(&self, keys: &[String]) {
            let keys: Vec<Retained<NSString>> =
                keys.iter().map(|one| NSString::from_str(one)).collect();
            center().removePendingNotificationRequestsWithIdentifiers(
                &NSArray::from_retained_slice(&keys),
            );
        }

        fn add(&self, ring: &Ring) -> bool {
            let content = UNMutableNotificationContent::new();
            content.setTitle(&NSString::from_str(&ring.heading));
            content.setBody(&NSString::from_str(&ring.title));
            if ring.heard {
                content.setSound(Some(&UNNotificationSound::defaultSound()));
            }
            if ring.urgent {
                content.setInterruptionLevel(UNNotificationInterruptionLevel::TimeSensitive);
            }
            let trigger = UNCalendarNotificationTrigger::triggerWithDateMatchingComponents_repeats(
                &parts(ring.at),
                false,
            );
            let trigger: &UNNotificationTrigger = &trigger;
            let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
                &NSString::from_str(&ring.key()),
                &content,
                Some(trigger),
            );
            let (told, heard) = mpsc::channel();
            let answer = RcBlock::new(move |trouble: *mut NSError| {
                let _ = told.send(trouble.is_null());
            });
            center().addNotificationRequest_withCompletionHandler(&request, Some(&answer));
            heard.recv_timeout(WAIT).unwrap_or(false)
        }
    }

    fn parts(at: jiff::Timestamp) -> Retained<NSDateComponents> {
        let civil = at.to_zoned(jiff::tz::TimeZone::system()).datetime();
        let parts = NSDateComponents::new();
        parts.setYear(civil.year().into());
        parts.setMonth(civil.month().into());
        parts.setDay(civil.day().into());
        parts.setHour(civil.hour().into());
        parts.setMinute(civil.minute().into());
        parts.setSecond(civil.second().into());
        parts
    }

    fn ask() {
        let answer = RcBlock::new(|_granted: Bool, _trouble: *mut NSError| {});
        center().requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &answer,
        );
    }

    type Opened = Box<dyn Fn(String) + Send + Sync>;

    define_class!(
        #[unsafe(super(NSObject))]
        #[name = "TistyReminderListener"]
        #[ivars = Opened]
        struct Listener;

        unsafe impl NSObjectProtocol for Listener {}

        unsafe impl UNUserNotificationCenterDelegate for Listener {
            #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
            fn shown(
                &self,
                _center: &UNUserNotificationCenter,
                _notification: &UNNotification,
                show: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
            ) {
                show.call((UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::List
                    | UNNotificationPresentationOptions::Sound,));
            }

            #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
            fn pressed(
                &self,
                _center: &UNUserNotificationCenter,
                response: &UNNotificationResponse,
                done: &block2::DynBlock<dyn Fn()>,
            ) {
                let key = response.notification().request().identifier().to_string();
                if let Some(task) = super::task_of(&key) {
                    (self.ivars())(task.to_string());
                }
                done.call(());
            }
        }
    );

    impl Listener {
        fn new(opened: Opened) -> Retained<Self> {
            let this = Self::alloc().set_ivars(opened);
            unsafe { msg_send![super(this), init] }
        }
    }

    // The centre holds its delegate weakly, and this one answers for as long as Tisty runs.
    pub fn listen(app: &tauri::AppHandle) {
        let app = app.clone();
        let listener = Listener::new(Box::new(move |task| crate::viewing::to_task(&app, task)));
        center().setDelegate(Some(ProtocolObject::from_ref(&*listener)));
        std::mem::forget(listener);
    }
}

#[cfg(all(not(windows), not(target_os = "macos")))]
mod there {
    use super::Book;

    pub const MOST: usize = 0;

    pub const PAGE: Option<&str> = None;

    pub fn urgent() -> bool {
        false
    }

    pub fn book() -> Option<Box<dyn Book>> {
        None
    }

    pub fn listen(_app: &tauri::AppHandle) {}
}

#[cfg(test)]
#[path = "booking_test.rs"]
mod tests;
