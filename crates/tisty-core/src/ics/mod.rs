mod lines;
mod read;
mod rule;
mod taking;
mod when;
mod write;

pub use read::{
    Alarm, BYTES_AT_MOST, Calendar, ITEMS_AT_MOST, Item, Kind, Unread, read, read_bytes,
};
pub(crate) use taking::{EVE_AT, TIMED_BEFORE};
pub use taking::{Odd, SOURCE, Taken, taken};
pub use write::written;

#[cfg(test)]
#[path = "ics_test.rs"]
mod tests;
