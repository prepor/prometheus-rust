use ::implementation::{Implementation, Collected, CollectedInstance};
use std::rc::Rc;

pub struct Histogram {
    sum: f64,
    counts: Box<[(f64, f64)]>,
    labels: Rc<Vec<String>>
}

impl Histogram {
    pub fn new(counts: Box<[(f64, f64)]>, labels: Rc<Vec<String>>) -> Histogram {
        Histogram { sum: 0.0, counts: counts, labels: labels }
    }

    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        for &mut (upper_bound, ref mut v) in self.counts.iter_mut() {
            if value < upper_bound {
                *v += 1.0;
            }
        }
    }
}

impl Implementation for Histogram {
    fn metric_type() -> &'static str {
        "histogram"
    }

    fn collect(&self) -> Collected {
        let mut res = Vec::with_capacity(self.counts.len() + 2);
        let &(_, count) = self.counts.last().unwrap();
        for (i, &(_, v)) in self.counts.iter().take(self.counts.len() - 1).enumerate() {
            res.push(CollectedInstance::new(v, None, Some(vec![("le".to_string(), self.labels[i].clone())])));
        }
        res.push(CollectedInstance::new(self.sum, Some("sum".to_string()), None));
        res.push(CollectedInstance::new(count, Some("count".to_string()), None));
        res
    }
}
