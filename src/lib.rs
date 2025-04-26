//! # MetricsLogger
//!
//! `MetricsLogger` is a library for logging and recording metrics in a structured and efficient manner.
//! It integrates with the `metrics` crate and provides a custom implementation of the `Recorder` trait
//! to handle counters, gauges, and histograms. The library is designed to allow users to log metrics
//! periodically and handle errors gracefully.
//!
//! ## Features
//! - **Custom Recorder**: Implements the `Recorder` trait to register and describe metrics.
//! - **Threaded Logging**: Uses a background thread to periodically log metrics based on a configurable interval.
//! - **Error Handling**: Allows users to provide custom error handling callbacks.
//!
//! ## Usage
//! To use `MetricsLogger`, create an instance of the `MetricsLogger` struct by specifying the logging interval,
//! a logging callback, and an error callback. The library will handle the rest, including updating metrics
//! and invoking the provided callbacks. Note that the callbacks are required to avoid potential issues
//! related to the user's project using a different version of the `log` or `tracing` crate than this crate.
//!
//! ## Example
//! ```rust
//! use metrics_logger::MetricsLogger;
//!
//! let logger = MetricsLogger::new(
//!     10, // Log interval in seconds
//!     |logs| println!("Metrics: {}", logs), // Logging callback
//!     |err| eprintln!("Error: {}", err),    // Error callback
//! );
//! ```
//!
//! ## Modules
//! - `cmd`: Handles commands for updating metrics.
//! - `handles`: Contains implementations for metric handles (e.g., counters, gauges, histograms).
//! - `state`: Manages the internal state of metrics and generates logs.
//!
//! ## Dependencies
//! This library re-exports the `metrics` crate to ensure compatibility with the same version used internally.

// let the library user have the same version of metrics as this crate.
pub use metrics;

mod cmd;
mod handles;
mod state;

use cmd::*;
use handles::*;
use state::*;

use metrics::{Counter, Gauge, Histogram, Key, KeyName, Metadata, Recorder, SharedString, Unit};
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

pub struct MetricsLogger<F> {
    tx: Sender<MetricsCmd>,
    err_cb: F,
}

impl<F> MetricsLogger<F>
where
    F: Fn(&str) + Copy + Send + Sync + 'static,
{
    pub fn new<F2>(log_interval_secs: u64, log_cb: F2, err_cb: F) -> Self
    where
        F2: Fn(&str) + Copy + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let mut state = MetricsState::new();
        std::thread::spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_secs(log_interval_secs)) {
                    Ok(cmd) => {
                        state.update(cmd);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(logs) = state.output_logs() {
                            (log_cb)(&logs);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
        Self { tx, err_cb }
    }
}

impl<F> Recorder for MetricsLogger<F>
where
    F: Fn(&str) + Copy + Send + Sync + 'static,
{
    fn describe_counter(&self, _name: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_gauge(&self, _name: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_histogram(&self, _name: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn register_counter(&self, key: &Key, _meta: &Metadata<'_>) -> Counter {
        let name = key.name().to_string();
        let handle = CounterHandle {
            name,
            tx: self.tx.clone(),
            err_cb: self.err_cb,
        };
        Counter::from_arc(Arc::new(handle))
    }

    fn register_gauge(&self, key: &Key, _meta: &Metadata<'_>) -> Gauge {
        let name = key.name().to_string();

        let handle = GaugeHandle {
            name,
            tx: self.tx.clone(),
            err_cb: self.err_cb,
        };
        Gauge::from_arc(Arc::new(handle))
    }

    fn register_histogram(&self, key: &Key, _meta: &Metadata<'_>) -> Histogram {
        let name = key.name().to_string();
        let handle = HistogramHandle {
            name,
            tx: self.tx.clone(),
            err_cb: self.err_cb,
        };
        Histogram::from_arc(Arc::new(handle))
    }
}
