//! Sinks for consuming, aggregating, and filtering query result paths.

pub mod astar_sink;
pub mod count;
pub mod weighted_sample;

pub use astar_sink::{BestKSink, ScoredPath, ThresholdSink};
pub use count::CountSink;
pub use weighted_sample::WeightedSampleSink;

/// Core trait defining the lifecycle of a Path Sink.
pub trait PathSink<P> {
    /// Process and accept a single path item from the execution stream.
    fn sink(&mut self, path: P);

    /// Finalize execution, flushing accumulated state or metrics.
    /// Returns `true` if the state was mutated or written successfully.
    fn finalize(&mut self) -> bool;
}
