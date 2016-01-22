pub mod expose;
pub mod implementation;
pub mod metrics;

mod helpers;

use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::fmt;
use std::error;

use implementation::Implementation;

pub struct Registry<'a> {
    metrics: HashMap<String, Weak<RefCell<Collectable + 'a>>>
}

impl <'a>Registry<'a> {
    pub fn new() -> Registry<'a> {
        Registry { metrics: HashMap::new() }
    }

    pub fn counter(&mut self, name: String, help: Option<String>, labels: Vec<String>)
                   -> Result<Rc<RefCell<Metric<metrics::Counter>>>, Error> {
        let m = Metric::new(Box::new(|| metrics::Counter::new()), name.clone(), help, labels);
        self.register(name, m)
    }

    pub fn summary(&mut self, name: String, help: Option<String>, labels: Vec<String>)
                   -> Result<Rc<RefCell<Metric<metrics::Summary>>>, Error> {
        let m = Metric::new(Box::new(|| metrics::Summary::new()), name.clone(), help, labels);
        self.register(name, m)
    }

    pub fn gauge(&mut self, name: String, help: Option<String>, labels: Vec<String>)
                 -> Result<Rc<RefCell<Metric<metrics::Gauge>>>, Error> {
        let m = Metric::new(Box::new(|| metrics::Gauge::new()), name.clone(), help, labels);
        self.register(name, m)
    }

    pub fn histogram(&mut self, name: String, help: Option<String>, labels: Vec<String>, buckets: Vec<f64>)
                     -> Result<Rc<RefCell<Metric<metrics::Histogram>>>, Error> {
        let (buckets, bucket_labels) = helpers::histogram::buckets_to_counts(buckets);
        let constructor = move|| metrics::Histogram::new(buckets.clone(), bucket_labels.clone());
        let m = Metric::new(Box::new(constructor), name.clone(), help, labels);
        self.register(name, m)
    }

    pub fn register<T: Implementation + 'a>(&mut self, name: String, metric: Metric<T>)
                                            -> Result<Rc<RefCell<Metric<T>>>, Error> {
        let m = Rc::new(RefCell::new(metric));
        if self.metrics.contains_key(&name) {
            return Err(Error::MetricAlreadyExists);
        }
        self.metrics.insert(name, Rc::downgrade(&m) as Weak<RefCell<Collectable>>);
        Ok(m)
    }

    pub fn expose(&mut self, format: expose::Format) -> expose::Formatted {
        match format {
            expose::Format::Text => helpers::expose::text(self)
        }
    }
}


pub struct Metric<T> {
    name: String,
    help: Option<String>,
    labels: Vec<String>,
    constructor: Box<Fn() -> T>,
    storage: HashMap<u64, WithLabels<T>>,
}

impl <T: Implementation>Metric<T> {
    fn new(constructor: Box<Fn () -> T>, name: String, help: Option<String>, labels: Vec<String>) -> Metric<T> {
        Metric {
            name: name,
            help: help,
            labels: labels,
            constructor: constructor,
            storage: HashMap::new(),
        }
    }

    pub fn labels(&mut self, labels: Labels) -> Result<&mut T, Error> {
        let hash = try!(helpers::hash_labels(&self.labels, labels));
        let constructor = &self.constructor;
        let res = &mut self.storage.entry(hash)
            .or_insert_with(|| {
                let labels = labels.iter()
                    .map(|&(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<(String, String)>>();
                WithLabels::new(constructor(), labels)
            }).implementation;
        Ok(res)
    }
}

struct WithLabels<T> {
    implementation: T,
    labels: Vec<(String, String)>
}

impl <T>WithLabels<T> {
    fn new(implementation: T, labels: Vec<(String, String)>) -> WithLabels<T> {
        WithLabels { implementation: implementation, labels: labels }
    }
}

struct Descr<'a> {
    name: &'a String,
    help: &'a Option<String>,
    metric_type: &'a str,
}

struct Value<'a> {
    labels: &'a Vec<(String, String)>,
    collected: implementation::Collected,
}

trait Collectable {
    fn descr(&self) -> Descr;
    fn values(&self) -> Vec<Value>;
}

impl <T: Implementation>Collectable for Metric<T> {
    fn descr(&self) -> Descr {
        Descr {
            name: &self.name,
            help: &self.help,
            metric_type: T::metric_type(),
        }
    }
    fn values(&self) -> Vec<Value> {
        self.storage.values().map(|v| {
            Value {
                labels: &v.labels,
                collected: v.implementation.collect()
            }
        }).collect::<Vec<Value>>()
    }
}

pub type Labels<'a> = &'a[(&'a str, &'a str)];

#[derive(Debug)]
pub enum Error {
    InconsistentLabels(String),
    MetricAlreadyExists
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InconsistentLabels(ref reason) => write!(f, "Passed labels don't consistent with metric's declared metrics: {}", reason),
            Error::MetricAlreadyExists => write!(f, "Metric with this name already exists")
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InconsistentLabels(_) => "InconsistentLabels",
            Error::MetricAlreadyExists => "Already exists"
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn counter() {
        let mut registry = Registry::new();
        let counter = registry.counter("main".to_string(), None, vec!["path".to_string()]).unwrap();
        {
            let mut counter = counter.borrow_mut();
            let mut c1 = counter.labels(&[("path", "root")]).unwrap();
            c1.inc();
            c1.inc();
        }
        let res = String::from_utf8(registry.expose(expose::Format::Text).body).unwrap();
        assert_eq!("# TYPE main counter\n\
                    main{path=\"root\"} 2\n", res)
    }

    #[test]
    fn histogram() {
        let mut registry = Registry::new();
        let histogram = registry.histogram("main".to_string(), None, vec!["path".to_string()], vec![1.0, 5.0, 10.0, 50.0, 100.0, 300.0]).unwrap();
        {
            let mut histogram = histogram.borrow_mut();
            let mut c1 = histogram.labels(&[("path", "root")]).unwrap();
            c1.observe(30.0);
            c1.observe(250.0);
        }
        let res = String::from_utf8(registry.expose(expose::Format::Text).body).unwrap();
        assert_eq!("# TYPE main histogram\n\
                    main{path=\"root\",le=\"1\"} 0\n\
                    main{path=\"root\",le=\"5\"} 0\n\
                    main{path=\"root\",le=\"10\"} 0\n\
                    main{path=\"root\",le=\"50\"} 1\n\
                    main{path=\"root\",le=\"100\"} 1\n\
                    main{path=\"root\",le=\"300\"} 2\n\
                    main_sum{path=\"root\"} 280\n\
                    main_count{path=\"root\"} 2\n", res);
    }

    #[test]
    fn summary() {
        let mut registry = Registry::new();
        let summary = registry.summary("main".to_string(), None, vec!["path".to_string()]).unwrap();
        {
            let mut summary = summary.borrow_mut();
            let mut c1 = summary.labels(&[("path", "root")]).unwrap();
            c1.observe(30.0);
            c1.observe(250.0);
        }
        let res = String::from_utf8(registry.expose(expose::Format::Text).body).unwrap();
        assert_eq!("# TYPE main summary\n\
                    main_sum{path=\"root\"} 280\n\
                    main_count{path=\"root\"} 2\n", res);
    }
}
