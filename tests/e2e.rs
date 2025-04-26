use metrics_logger::{MetricsLogger, metrics};
use std::time::Duration;

#[test]
fn test_metrics_logger_integration() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let recorder = MetricsLogger::new(
        1,
        |logs| log::debug!("{}", logs),
        |err| log::error!("MetricsLogger error: {}", err),
    );

    metrics::set_global_recorder(recorder).unwrap();

    // Register a counter
    let counter = metrics::counter!("test_counter");

    // Spawn a thread to increment the counter
    let handle = std::thread::spawn(move || {
        for _ in 0..5 {
            counter.increment(1);
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    // Wait for the thread to finish
    handle.join().expect("Thread panicked");
}
