#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod desktop;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    desktop::run()
}
