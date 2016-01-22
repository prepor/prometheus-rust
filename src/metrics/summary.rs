use ::implementation::{Implementation, Collected, CollectedInstance};

pub struct Summary {
    sum: f64,
    count: f64,
}

impl Summary {
    pub fn new() -> Summary {
        Summary{ sum: 0.0, count: 0.0}
    }

    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1.0;
    }
}

impl Implementation for Summary {
    fn metric_type() -> &'static str {
        "summary"
    }

    fn collect(&self) -> Collected {
        vec![CollectedInstance::new(self.sum, Some("sum".to_string()), None),
             CollectedInstance::new(self.count, Some("count".to_string()), None)]
    }
}
