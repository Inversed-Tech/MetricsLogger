use metrics_logger::{MetricsLogger, metrics};
use std::time::Duration;

#[test]
fn test_metrics_logger_integration() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let recorder = MetricsLogger::new(
        1,
        |logs| log::debug!("\n{}", logs),
        |err| log::error!("MetricsLogger error: {}", err),
    );
    metrics::set_global_recorder(recorder).unwrap();

    let counter = metrics::counter!("test_counter");
    let gauge = metrics::gauge!("test_gauge");
    let histogram = metrics::histogram!("test_histogram");

    let handle = std::thread::spawn(move || {
        println!("generating logs");
        for idx in 0..3 {
            counter.increment(1);
            gauge.increment(idx);
            histogram.record(idx as f64);
            std::thread::sleep(Duration::from_secs(1));
        }

        println!("sleeping. no logs should be generated");
        std::thread::sleep(Duration::from_secs(3));

        println!(
            "logging again. testing that metrics with the same name are counted in the same bucket."
        );
        for idx in 4..6 {
            metrics::counter!("test_counter").increment(1);
            metrics::gauge!("test_gauge").decrement(idx);
            metrics::histogram!("test_histogram").record(idx as f64);
            std::thread::sleep(Duration::from_secs(1));
        }
    });
    handle.join().expect("Thread panicked");
}
