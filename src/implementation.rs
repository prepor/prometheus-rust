pub trait Implementation {
    fn metric_type() -> &'static str;
    fn collect(&self) -> Collected;
}

pub type Collected = Vec<CollectedInstance>;

pub struct CollectedInstance {
    pub suffix: Option<String>,
    pub addition_labels: Option<Vec<(String, String)>>,
    pub value: f64
}

impl CollectedInstance {
    pub fn new(v: f64, suffix: Option<String>, addition_labels: Option<Vec<(String, String)>>)
           -> CollectedInstance {
        CollectedInstance {
            suffix: suffix,
            addition_labels: addition_labels,
            value: v
        }
    }
    pub fn value(v: f64) -> CollectedInstance {
        Self::new(v, None, None)
    }
}
