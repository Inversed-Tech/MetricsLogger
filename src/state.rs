use crate::PeriodicMode;
use crate::cmd::*;
use std::collections::{HashMap, HashSet};

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
                        .or_default();

                    self.histogram_updates.insert(name);
                }
            },
        }
    }

    pub fn output_logs(&mut self, mode: PeriodicMode) -> Option<String> {
        match mode {
            PeriodicMode::Diff => self.output_diff(),
            PeriodicMode::Full => self.output_full(),
        }
    }
    fn output_diff(&mut self) -> Option<String> {
        let mut logs = String::new();

        // Process counter updates
        for name in self.counter_updates.drain() {
            if let Some(value) = self.counter_state.get(&name) {
                logs.push_str(&format!(r#"{{"{}": {}}},"#, name, value));
            }
        }

        // Process gauge updates
        for name in self.gauge_updates.drain() {
            if let Some(value) = self.gauge_state.get(&name) {
                logs.push_str(&format!(r#"{{"{}": {}}},"#, name, value));
            }
        }

        // Process histogram updates
        for name in self.histogram_updates.drain() {
            if let Some(histogram) = self.histogram_state.get(&name) {
                let avg = histogram.avg().unwrap_or(0.0);
                let std_dev = histogram.std_dev().unwrap_or(0.0);
                logs.push_str(&format!(
                    r#"{{"{}": {{"avg": {:.2}, "std_dev": {:.2}, "min": {:.2}, "max": {:.2}, "samples": {}}}}},"#,
                    name, avg, std_dev, histogram.min, histogram.max, histogram.num_samples
                ));
            }
        }

        if logs.is_empty() { None } else { Some(logs) }
    }

    fn output_full(&mut self) -> Option<String> {
        let mut logs = String::new();
        // Print all counter states as JSON
        for (name, value) in &self.counter_state {
            logs.push_str(&format!(r#"{{"{}": {}}},"#, name, value));
        }

        // Print all gauge states as JSON
        for (name, value) in &self.gauge_state {
            logs.push_str(&format!(r#"{{"{}": {}}},"#, name, value));
        }

        // Print all histogram states as JSON
        for (name, histogram) in &self.histogram_state {
            let avg = histogram.avg().unwrap_or(0.0);
            let std_dev = histogram.std_dev().unwrap_or(0.0);
            logs.push_str(&format!(
                r#"{{"{}": {{"avg": {:.2}, "std_dev": {:.2}, "min": {:.2}, "max": {:.2}, "samples": {}}}}},"#,
                name, avg, std_dev, histogram.min, histogram.max, histogram.num_samples
            ));
        }

        if logs.is_empty() { None } else { Some(logs) }
    }
}

#[derive(Default)]
struct HistogramState {
    sum: f64,
    sum_sq: f64,
    num_samples: u64,
    min: f64,
    max: f64,
}

impl HistogramState {
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
        if self.num_samples == 0 {
            None
        } else {
            let avg_sq = self.avg().unwrap() * self.avg().unwrap();
            let avg_ss = self.sum_sq / self.num_samples as f64;
            Some((avg_ss - avg_sq).sqrt())
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_state_avg() {
        let mut histogram = HistogramState::default();
        histogram.update(10.0);
        histogram.update(20.0);
        histogram.update(30.0);

        assert_eq!(histogram.avg(), Some(20.0));
    }

    #[test]
    fn test_histogram_state_std_dev() {
        let mut histogram = HistogramState::default();
        histogram.update(10.0);
        histogram.update(20.0);
        histogram.update(30.0);

        // calculate using moments: sqrt( (100 + 400 + 900)/3 - (60/3)^2 )
        let expected_std_dev: f64 = 8.16;
        let rounded_std_dev = (histogram.std_dev().unwrap() * 100.0).round() / 100.0;
        assert_eq!(rounded_std_dev, expected_std_dev);
    }

    #[test]
    fn test_histogram_state_empty_avg() {
        let histogram = HistogramState::default();
        assert_eq!(histogram.avg(), None);
    }

    #[test]
    fn test_histogram_state_empty_std_dev() {
        let histogram = HistogramState::default();
        assert_eq!(histogram.std_dev(), None);
    }

    #[test]
    fn test_histogram_state_min_max() {
        let mut histogram = HistogramState::default();
        histogram.update(15.0);
        histogram.update(5.0);
        histogram.update(25.0);

        assert_eq!(histogram.min, 5.0);
        assert_eq!(histogram.max, 25.0);
    }
}
