use super::PathSink;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Scored path container for A* Sinks
#[derive(Debug, Clone)]
pub struct ScoredPath<P> {
    pub path: P,
    pub f_cost: f64,
}

impl<P> PartialEq for ScoredPath<P> {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl<P> Eq for ScoredPath<P> {}

// Default ordering: Higher f_cost has higher priority (Max-Heap)
impl<P> Ord for ScoredPath<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_cost
            .partial_cmp(&other.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl<P> PartialOrd for ScoredPath<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// 1. Best-K A* Sink (Top-K Lowest-Cost Paths)

/// A sink that retains only the top-K lowest cost paths discovered by an A* traversal.
pub struct BestKSink<P> {
    k: usize,
    /// Max-Heap storing top-K lowest cost paths.
    /// The path with the LARGEST f_cost among the top-K resides at root (peek()).
    buffer: BinaryHeap<ScoredPath<P>>,
}

impl<P> BestKSink<P> {
    /// Create a new BestKSink with a maximum capacity of `k` paths.
    pub fn new(k: usize) -> Self {
        assert!(k > 0, "Capacity k must be greater than 0");
        Self {
            k,
            buffer: BinaryHeap::with_capacity(k + 1),
        }
    }

    /// Process a path with an explicit A* cost f(n) = g(n) + h(n).
    pub fn sink_scored(&mut self, path: P, f_cost: f64) {
        let entry = ScoredPath { path, f_cost };

        if self.buffer.len() < self.k {
            self.buffer.push(entry);
        } else if let Some(max_entry) = self.buffer.peek() {
            // If new path has a lower cost than our current worst top-k entry, replace it
            if entry.f_cost < max_entry.f_cost {
                self.buffer.pop();
                self.buffer.push(entry);
            }
        }
    }

    /// Drain and return the winning top-K paths ordered from lowest cost to highest cost.
    pub fn into_sorted_paths(self) -> Vec<ScoredPath<P>> {
        let mut vec = self.buffer.into_vec();
        vec.sort_by(|a, b| {
            a.f_cost
                .partial_cmp(&b.f_cost)
                .unwrap_or(Ordering::Equal)
        });
        vec
    }
}

impl<P> PathSink<ScoredPath<P>> for BestKSink<P> {
    fn sink(&mut self, item: ScoredPath<P>) {
        self.sink_scored(item.path, item.f_cost);
    }

    fn finalize(&mut self) -> bool {
        !self.buffer.is_empty()
    }
}

// 2. Cost-Threshold Pruning Sink
/// A sink that accepts paths only if their cost f(n) is within a maximum threshold.
pub struct ThresholdSink<P> {
    max_cost_threshold: f64,
    accepted_paths: Vec<ScoredPath<P>>,
}

impl<P> ThresholdSink<P> {
    /// Create a ThresholdSink with a maximum allowed path cost threshold.
    pub fn new(max_cost_threshold: f64) -> Self {
        Self {
            max_cost_threshold,
            accepted_paths: Vec::new(),
        }
    }

    /// Access all collected paths within cost limit.
    pub fn accepted_paths(&self) -> &[ScoredPath<P>] {
        &self.accepted_paths
    }

    /// Consume sink and retrieve collected paths.
    pub fn into_paths(self) -> Vec<ScoredPath<P>> {
        self.accepted_paths
    }
}

impl<P> PathSink<ScoredPath<P>> for ThresholdSink<P> {
    fn sink(&mut self, item: ScoredPath<P>) {
        if item.f_cost <= self.max_cost_threshold {
            self.accepted_paths.push(item);
        }
    }

    fn finalize(&mut self) -> bool {
        !self.accepted_paths.is_empty()
    }
}
