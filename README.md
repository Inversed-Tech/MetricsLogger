# metrics-logger

`metrics-logger` is a crate for logging metrics. It aids development and testing by allowing developers to view metrics without setting up a network endpoint.

This is achieved by implementing the `Recorder` trait from the `metrics` crate.

## Notes
- This crate exports the `metrics` crate to avoid version mismatches. The version of `metrics-logger` matches that of the `metrics` crate.
- `MetricsLogger` requires callbacks to avoid issues with different versions of the `log` or `tracing` crates in the user's project.

## Example

```rust
use metrics_logger::{metrics, MetricsLogger, LogMode};

// MetricsLogger implements the Recorder trait
let recorder = MetricsLogger::new(
    LogMode::Periodic(10), // Logging interval in seconds
    |logs| println!("Metrics: {}", logs), // Logging callback
    |err| eprintln!("Error: {}", err),    // Error callback
);

// This tells the metrics crate to use your Recorder implementation.
metrics::set_global_recorder(recorder).unwrap();
```

## Modules
- `cmd`: Handles commands for updating metrics.
- `handles`: Implements metric handles (e.g., counters, gauges, histograms).
- `state`: Manages metric state and generates logs.

## Dependencies
This library re-exports the `metrics` crate to ensure compatibility with the same version used internally.
