fn main() {
    if let Err(error) = runhaven_core::harness::pins::check_pins() {
        eprintln!("runhaven-check-pins: {error}");
        std::process::exit(1);
    }
}
