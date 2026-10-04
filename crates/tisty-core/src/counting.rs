use std::cell::Cell;

thread_local! {
    static OPENED: Cell<u64> = const { Cell::new(0) };
    static LOOKED: Cell<u64> = const { Cell::new(0) };
}

pub fn opened() {
    OPENED.with(|one| one.set(one.get() + 1));
}

pub fn opens() -> u64 {
    OPENED.with(Cell::get)
}

pub fn from_now() -> u64 {
    OPENED.with(|one| one.replace(0))
}

/// A question about a file that reads none of it: what a quiet round mostly spends.
pub fn looked() {
    LOOKED.with(|one| one.set(one.get() + 1));
}

pub fn looks_from_now() -> u64 {
    LOOKED.with(|one| one.replace(0))
}
