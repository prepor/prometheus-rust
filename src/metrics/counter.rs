use ::implementation::{Implementation, Collected, CollectedInstance};

pub struct Counter {
    sum: f64,
}

impl Counter {
    pub fn new() -> Counter {
        Counter { sum: 0.0 }
    }

    pub fn inc(&mut self) {
        self.add(1.0);
    }

    pub fn add(&mut self, value: f64) {
        assert!(value >= 0.0);
        self.sum += value;;
    }
}


impl Implementation for Counter {
    fn metric_type() -> &'static str {
        "counter"
    }

    fn collect(&self) -> Collected {
        vec![CollectedInstance::value(self.sum)]
    }
}
