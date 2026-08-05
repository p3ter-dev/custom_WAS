use std::cmp::Ordering;
use std::collections::BinaryHeap;
use rand::Rng;
use super::PathSink;

/// Internal entry stored in the priority queue.
#[derive(Debug, Clone)]
struct SampleEntry {
    /// A-Res score key: k = u^(1/w)
    key: f64,
    path: Vec<u8>,
}

impl PartialEq for SampleEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for SampleEntry {}

// Reverse ordering to make BinaryHeap act as a Min-Heap (smallest key at root).
impl Ord for SampleEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.key.partial_cmp(&self.key).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SampleEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A sink that collects a weighted random sample of up to `max_samples` paths.
pub struct WeightedSampleSink {
    max_samples: usize,
    samples: BinaryHeap<SampleEntry>,
    rng: rand::rngs::ThreadRng,
}

impl WeightedSampleSink {
    /// Create a new sampling sink for a specific sample size limit `max_samples`.
    pub fn new(max_samples: usize) -> Self {
        assert!(max_samples > 0, "Sample size must be greater than zero");
        Self {
            max_samples,
            samples: BinaryHeap::with_capacity(max_samples + 1),
            rng: rand::thread_rng(),
        }
    }

    /// Process a path alongside an explicit weight.
    pub fn sink_weighted(&mut self, path: &[u8], weight: f64) {
        let weight = if weight <= 0.0 { 1.0 } else { weight };
        
        // Compute A-Res key: k = u^(1/w)
        let u: f64 = self.rng.gen::<f64>();
        let key = u.powf(1.0 / weight);

        if self.samples.len() < self.max_samples {
            self.samples.push(SampleEntry {
                key,
                path: path.to_vec(),
            });
        } else if let Some(min_entry) = self.samples.peek() {
            if key > min_entry.key {
                self.samples.pop(); // Evict smallest key
                self.samples.push(SampleEntry {
                    key,
                    path: path.to_vec(),
                });
            }
        }
    }

    /// Drain the selected sampled paths.
    pub fn into_samples(self) -> Vec<Vec<u8>> {
        self.samples.into_iter().map(|entry| entry.path).collect()
    }
}

impl PathSink<&[u8]> for WeightedSampleSink {
    fn sink(&mut self, path: &[u8]) {
        // Default weight of 1.0 when unweighted
        self.sink_weighted(path, 1.0);
    }

    fn finalize(&mut self) -> bool {
        !self.samples.is_empty()
    }
}
