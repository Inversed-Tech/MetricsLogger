use crate::cmd::*;
use metrics::{CounterFn, GaugeFn, HistogramFn};
use std::sync::mpsc::Sender;

pub(crate) struct CounterHandle<F> {
    pub(crate) name: String,
    pub(crate) tx: Sender<MetricsCmd>,
    pub(crate) err_cb: F,
}

impl<F> CounterFn for CounterHandle<F>
where
    F: Fn(&str),
{
    fn increment(&self, value: u64) {
        if let Err(e) = self.tx.send(MetricsCmd::Counter(CounterCmd::Increment {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!(
                "Failed to send counter metrics for increment: {:?}",
                e
            ));
        }
    }
    fn absolute(&self, value: u64) {
        if let Err(e) = self.tx.send(MetricsCmd::Counter(CounterCmd::Absolute {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!(
                "Failed to send counter metrics for absolute: {:?}",
                e
            ));
        }
    }
}
pub(crate) struct GaugeHandle<F> {
    pub(crate) name: String,
    pub(crate) tx: Sender<MetricsCmd>,
    pub(crate) err_cb: F,
}

impl<F> GaugeFn for GaugeHandle<F>
where
    F: Fn(&str),
{
    fn increment(&self, value: f64) {
        if let Err(e) = self.tx.send(MetricsCmd::Gauge(GaugeCmd::Increment {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!(
                "Failed to send gauge metrics for increment: {:?}",
                e
            ));
        }
    }

    fn decrement(&self, value: f64) {
        if let Err(e) = self.tx.send(MetricsCmd::Gauge(GaugeCmd::Decrement {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!(
                "Failed to send gauge metrics for decrement: {:?}",
                e
            ));
        }
    }

    fn set(&self, value: f64) {
        if let Err(e) = self.tx.send(MetricsCmd::Gauge(GaugeCmd::Set {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!("Failed to send gauge metrics for set: {:?}", e));
        }
    }
}

pub(crate) struct HistogramHandle<F> {
    pub(crate) name: String,
    pub(crate) tx: Sender<MetricsCmd>,
    pub(crate) err_cb: F,
}

impl<F> HistogramFn for HistogramHandle<F>
where
    F: Fn(&str),
{
    fn record(&self, value: f64) {
        if let Err(e) = self.tx.send(MetricsCmd::Histogram(HistogramCmd::Record {
            name: self.name.clone(),
            value,
        })) {
            (self.err_cb)(&format!("Failed to send histogram metrics: {:?}", e));
        }
    }
}
