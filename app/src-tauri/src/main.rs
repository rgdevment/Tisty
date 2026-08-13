#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|one| one == "--unreach") {
        let _ = tisty_gui_lib::unreach();
        return;
    }
    tisty_gui_lib::run()
}
