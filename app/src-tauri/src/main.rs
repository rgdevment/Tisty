#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if tisty_gui_lib::asked_to_unreach() {
        let _ = tisty_gui_lib::unreach();
        return;
    }
    tisty_gui_lib::run()
}
