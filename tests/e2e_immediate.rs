use metrics_logger::LogMode;
mod utils;

#[test]
fn metrics_logger_e2e_immediate() {
    println!("testing Immediate mode");
    utils::metrics_logger_test(LogMode::Immediate);
}
