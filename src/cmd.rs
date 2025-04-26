pub enum CounterCmd {
    Increment { name: String, value: u64 },
    Absolute { name: String, value: u64 },
}

pub enum GaugeCmd {
    Increment { name: String, value: f64 },
    Decrement { name: String, value: f64 },
    Set { name: String, value: f64 },
}

pub enum HistogramCmd {
    Record { name: String, value: f64 },
}

pub enum MetricsCmd {
    Counter(CounterCmd),
    Gauge(GaugeCmd),
    Histogram(HistogramCmd),
}
