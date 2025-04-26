use crate::cmd::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct HistogramState {
    sum: f64,
    sum_sq: f64,
    num_samples: u64,
    min: f64,
    max: f64,
}

#[derive(Default)]
pub struct MetricsState {
    counter_state: HashMap<String, u64>,
    gauge_state: HashMap<String, i64>,
    histogram_state: HashMap<String, HistogramState>,

    counter_updates: HashSet<String>,
    gauge_updates: HashSet<String>,
    histogram_updates: HashSet<String>,
}

impl MetricsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, cmd: MetricsCmd) {
        match cmd {
            MetricsCmd::Counter(counter_cmd) => match counter_cmd {
                CounterCmd::Increment { name, value } => {
                    *self.counter_state.entry(name.clone()).or_insert(0) += value;
                    self.counter_updates.insert(name);
                }
                CounterCmd::Absolute { name, value } => {
                    self.counter_state.insert(name.clone(), value);
                    self.counter_updates.insert(name);
                }
            },
            MetricsCmd::Gauge(gauge_cmd) => match gauge_cmd {
                GaugeCmd::Increment { name, value } => {
                    *self.gauge_state.entry(name.clone()).or_insert(0) += value as i64;
                    self.gauge_updates.insert(name);
                }
                GaugeCmd::Decrement { name, value } => {
                    *self.gauge_state.entry(name.clone()).or_insert(0) -= value as i64;
                    self.gauge_updates.insert(name);
                }
                GaugeCmd::Set { name, value } => {
                    self.gauge_state.insert(name.clone(), value as i64);
                    self.gauge_updates.insert(name);
                }
            },
            MetricsCmd::Histogram(histogram_cmd) => match histogram_cmd {
                HistogramCmd::Record { name, value } => {
                    self.histogram_state
                        .entry(name.clone())
                        .and_modify(|x| x.update(value))
                        .or_insert_with(HistogramState::default);

                    self.histogram_updates.insert(name);
                }
            },
        }
    }

    pub fn output_logs(&mut self) {
        todo!()
    }
}

impl HistogramState {
    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, value: f64) {
        self.sum += value;
        self.sum_sq += value * value;
        if self.num_samples == 0 {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }
        self.num_samples += 1;
    }

    fn std_dev(&self) -> Option<f64> {
        if self.num_samples < 2 {
            None
        } else {
            Some((self.sum_sq / (self.num_samples - 1) as f64).sqrt())
        }
    }

    fn avg(&self) -> Option<f64> {
        if self.num_samples == 0 {
            None
        } else {
            Some(self.sum / self.num_samples as f64)
        }
    }
}
