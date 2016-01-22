use ::Labels;
use ::Error;

use std::hash::SipHasher;
use std::hash::Hasher;

pub fn hash_labels(required: &Vec<String>, labels: Labels) -> Result<u64, Error> {
    if labels.len() != required.len() {
        return Err(Error::InconsistentLabels(format!("different length: {} vs {}",
                                                     labels.len(), required.len())));
    }
    let mut hash = SipHasher::new();
    for label in required {
        let &(_, ref v) = try!(labels.iter().find(|&&(ref k, _)| label == k)
                               .ok_or_else(|| Error::InconsistentLabels(format!("Missing {} label", label))));
        hash.write(v.as_bytes());
    }
    Ok(hash.finish())
}

pub mod histogram {
    use std::f64;
    use std::rc::Rc;

    pub fn buckets_to_counts(buckets: Vec<f64>) -> (Box<[(f64, f64)]>, Rc<Vec<String>>) {
        // FIXME: validate buckets
        let mut counts = Vec::new();
        let mut labels = Vec::new();
        for bucket in buckets {
            counts.push((bucket, 0.0));
            labels.push(bucket.to_string());
        }
        counts.push((f64::MAX, 0.0));
        labels.push("+Inf".to_string());
        (counts.into_boxed_slice(), Rc::new(labels))
    }
}

pub mod expose {
    use ::Registry;
    use ::expose::*;
    pub fn text<'a>(registry: &mut Registry) -> Formatted<'a> {
        let mut buf = String::new();
        let mut to_forget = Vec::new();
        for (k, metric) in &registry.metrics {
            match metric.upgrade() {
                Some(metric) => {
                    let metric = metric.borrow();
                    let values = metric.values();
                    if values.len() > 0 {
                        let descr = metric.descr();
                        match descr.help {
                            &Some(ref help) =>
                                buf.push_str(&format!("# HELP {} {}\n", descr.name, help)),
                            &None => ()
                        }
                        buf.push_str(&format!("# TYPE {} {}\n", descr.name, descr.metric_type));
                        for v in values {
                            for collected in v.collected {
                                buf.push_str(descr.name);
                                match collected.suffix {
                                    Some(ref v) => {
                                        buf.push('_');
                                        buf.push_str(v);
                                    },
                                    None => ()
                                };
                                let adds_number = collected.addition_labels.clone().map_or(0, |v| v.len());
                                if v.labels.len() > 0 || adds_number > 0 {
                                    buf.push_str(&labels_to_text(v.labels, &collected.addition_labels));
                                }
                                buf.push(' ');
                                buf.push_str(&collected.value.to_string());
                                buf.push('\n');
                            }
                        }
                    }
                }
                None => to_forget.push(k)
            }

        }

        Formatted {
            content_type: "text/plain; version=0.0.4",
            body: buf.into_bytes()
        }
    }

    fn join_labels(buf: &mut String, labels: &Vec<(String, String)>) {
        let last_i = labels.len() - 1;
        for (i, &(ref label, ref value)) in labels.iter().enumerate() {
            buf.push_str(&label);
            buf.push('=');
            buf.push('"');
            buf.push_str(&value);
            buf.push('"');
            if i != last_i { buf.push(',') }
        }
    }

    fn labels_to_text(labels: &Vec<(String, String)>, addition_labels: &Option<Vec<(String, String)>>) -> String {
        let mut buf = String::new();
        buf.push('{');
        join_labels(&mut buf, labels);
        match addition_labels {
            &Some(ref v) => {
                if labels.len() > 0 { buf.push(',') };
                join_labels(&mut buf, v)
            },
            &None => ()
        }
        buf.push('}');
        buf
    }

}
