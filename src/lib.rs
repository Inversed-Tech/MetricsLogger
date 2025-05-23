//! # metrics-logger
//!
//! `metrics-logger` is a crate for logging metrics. It aids development and
//! testing by allowing developers to view metrics without setting up a network
//! endpoint.
//!
//! This is done by implementing the `Recorder` trait from the `metrics` crate.
//!
//! ## Notes
//! - This crate exports the `metrics` crate to avoid version mismatches. The
//!   version of `metrics-logger` matches that of the `metrics` crate.
//! - `MetricsLogger` requires callbacks to avoid issues with different versions
//!   of the `log` or `tracing` crates in the user's project.
//!
//! ## Example
//! ```rust
//! use metrics_logger::{metrics, MetricsLogger, LogMode};
//!
//! // MetricsLogger implements the Recorder trait
//! let recorder = MetricsLogger::new(
//!     LogMode::Periodic(10), // Logging interval in seconds
//!     |logs| println!("Metrics: {}", logs), // Logging callback
//!     |err| eprintln!("Error: {}", err),    // Error callback
//! );
//!
//! // This tells the metrics crate to use your Recorder implementation.
//! metrics::set_global_recorder(recorder).unwrap();
//! ```
//!
//! ## Modules
//! - `cmd`: Handles commands for updating metrics.
//! - `handles`: Implements metric handles (e.g., counters, gauges, histograms).
//! - `state`: Manages metric state and generates logs.
//!
//! ## Dependencies
//! This library re-exports the `metrics` crate to ensure compatibility with the
//! same version used internally.

pub use metrics;

mod cmd;
mod handles;
mod state;

use cmd::*;
use handles::*;
use state::*;

use metrics::{Counter, Gauge, Histogram, Key, KeyName, Metadata, Recorder, SharedString, Unit};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

pub struct MetricsLogger<F> {
    tx: Sender<MetricsCmd>,
    err_cb: F,
}

pub enum LogMode {
    /// Emit logs as soon as a metric is updated
    Immediate,
    /// Aggregate metrics for the specified duration, in seconds, before emitting a log
    Periodic(u64),
}

impl<F> MetricsLogger<F>
where
    F: Fn(&str) + Copy + Send + Sync + 'static,
{
    pub fn new<F2>(mode: LogMode, log_cb: F2, err_cb: F) -> Self
    where
        F2: Fn(&str) + Copy + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel();
        match mode {
            LogMode::Immediate => Self::launch_immediate_mode(rx, log_cb),
            LogMode::Periodic(log_interval_secs) => {
                Self::launch_periodic_mode(rx, log_cb, log_interval_secs)
            }
        }
        Self { tx, err_cb }
    }

    fn launch_immediate_mode<F2>(rx: Receiver<MetricsCmd>, log_cb: F2)
    where
        F2: Fn(&str) + Copy + Send + Sync + 'static,
    {
        std::thread::spawn(move || {
            let mut state = MetricsState::new();
            for cmd in rx.iter() {
                state.update(cmd);
                if let Some(logs) = state.output_logs() {
                    (log_cb)(&logs);
                }
            }
        });
    }

    fn launch_periodic_mode<F2>(rx: Receiver<MetricsCmd>, log_cb: F2, log_interval_secs: u64)
    where
        F2: Fn(&str) + Copy + Send + Sync + 'static,
    {
        std::thread::spawn(move || {
            let mut state = MetricsState::new();
            let interval = Duration::from_secs(log_interval_secs);
            let mut next_log_time = Instant::now() + interval;
            loop {
                match rx.recv_timeout(Duration::from_secs(log_interval_secs)) {
                    Ok(cmd) => {
                        state.update(cmd);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                let now = Instant::now();
                if now >= next_log_time {
                    if let Some(logs) = state.output_logs() {
                        (log_cb)(&logs);
                    }
                    next_log_time = now + interval;
                }
            }
        });
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
