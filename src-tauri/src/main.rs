// A menu bar app must not open a console window on Windows. macOS ignores this,
// but the attribute belongs here for when the Windows build arrives (D-001).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chandra_lib::run()
}
