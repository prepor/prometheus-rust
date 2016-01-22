use ::implementation::{Implementation, Collected, CollectedInstance};

pub struct Gauge {
    value: f64,
}

impl Gauge {
    pub fn new() -> Gauge {
        Gauge{ value: 0.0 }
    }

    pub fn inc(&mut self) {
        self.add(1.0);
    }

    pub fn add(&mut self, value: f64) {
        self.value += value;
    }

    pub fn dec(&mut self) {
        self.sub(1.0);
    }

    pub fn sub(&mut self, value: f64) {
        self.value -= value;
    }

    pub fn set(&mut self, value: f64) {
        self.value = value;
    }
}


impl Implementation for Gauge {
    fn metric_type() -> &'static str {
        "gauge"
    }

    fn collect(&self) -> Collected {
        vec![CollectedInstance::value(self.value)]
    }
}
