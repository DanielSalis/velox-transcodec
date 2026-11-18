mod desktop;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    desktop::run()
}
