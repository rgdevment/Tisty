// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The uninstaller calls this: it is the only moment left to take the PATH
    // entry back out, and by then there is no window to ask from.
    if std::env::args().any(|one| one == "--unreach") {
        let _ = tisty_gui_lib::unreach();
        return;
    }
    tisty_gui_lib::run()
}
