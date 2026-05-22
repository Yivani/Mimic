// Always run as a GUI app on Windows — never spawn/attach a console window.
#![windows_subsystem = "windows"]

fn main() {
    mimic_lib::run()
}
