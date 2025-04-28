use metrics_logger::LogMode;
mod utils;

#[test]
fn metrics_logger_e2e_periodic() {
    println!("testing Periodic mode");
    utils::metrics_logger_test(LogMode::Periodic(2));
}
