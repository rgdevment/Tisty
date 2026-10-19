use tisty_carrier::uploads::Uploads;

use super::Drive;

pub(super) use tisty_carrier::uploads::key_of;

// Google keeps a resumable session for a week; one a day short of that is begun again instead.
const KEPT_FOR: u64 = 6 * 24 * 60 * 60;

impl Drive {
    fn uploads(&self) -> Option<Uploads> {
        Some(Uploads::under(self.owing.as_ref()?.parent()?, KEPT_FOR))
    }

    pub(super) fn session_for(&self, key: &str) -> Option<String> {
        self.uploads()?.session_for(key)
    }

    pub(super) fn session_opened(&self, key: &str, uri: &str) {
        if let Some(uploads) = self.uploads() {
            uploads.opened(key, uri);
        }
    }

    pub(super) fn session_closed(&self, key: &str) {
        if let Some(uploads) = self.uploads() {
            uploads.closed(key);
        }
    }
}
