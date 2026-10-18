mod lines;
mod read;
mod rule;
mod taking;
mod when;

pub use read::{
    Alarm, BYTES_AT_MOST, Calendar, ITEMS_AT_MOST, Item, Kind, Unread, read, read_bytes,
};
pub use taking::{Odd, SOURCE, Taken, taken};

#[cfg(test)]
#[path = "ics_test.rs"]
mod tests;
