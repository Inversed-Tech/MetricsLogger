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

pub struct MetricsLogger<F> {
    tx: Sender<MetricsCmd>,
    err_cb: F,
}

impl<F> MetricsLogger<F>
where
    F: Fn(&str) + Copy + Send + Sync + 'static,
{
    pub fn new(log_interval_secs: usize, log_cb: F, err_cb: F) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut state = MetricsState::new();
        std::thread::spawn(move || {
            for cmd in rx.iter() {
                state.update(cmd);
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
