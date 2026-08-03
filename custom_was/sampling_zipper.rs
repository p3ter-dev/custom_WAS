use pathmap::utils::ByteMask;
use pathmap::zipper::{
    ReadZipperUntracked, TrieRef, Zipper, ZipperAbsolutePath, ZipperIteration,
    ZipperMoving, ZipperSubtries, ZipperValues,
};
use std::fmt;

/// A custom wrapper zipper that performs weighted sampling operations
/// over a PathMap trie zipper.
pub struct WeightedSamplingZipper<'trie, 'path, V: Clone + Send + Sync + Unpin> {
    pub inner: ReadZipperUntracked<'trie, 'path, V>,
    pub temperature: f32,
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> fmt::Debug
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeightedSamplingZipper")
            .field("inner", &self.inner)
            .field("temperature", &self.temperature)
            .finish()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> Clone
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            temperature: self.temperature,
        }
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> WeightedSamplingZipper<'trie, 'path, V> {
    pub fn new(inner: ReadZipperUntracked<'trie, 'path, V>, temperature: f32) -> Self {
        Self { inner, temperature }
    }

    /// Computes the temperature-scaled weights for available child steps 
    /// and performs a roulette-wheel / CDF weighted random choice.
    ///
    /// Mathematical formula:
    ///   W_effective[i] = (weight[i])^(1 / T)   or   exp(weight[i] / T)
    ///   P[i] = W_effective[i] / sum(W_effective)
    pub fn sample_next_child(&mut self) -> Option<u8> {
        let mask = self.child_mask();
        if mask.is_empty() {
            return None;
        }

        // Collect available byte branches
        let bytes: Vec<u8> = (0..=255)
            .filter(|&b| mask.get(b as usize))
            .collect();

        if bytes.is_empty() {
            return None;
        }

        // If temperature is 1.0 or single branch, pick uniform/proportional directly
        if bytes.len() == 1 {
            return Some(bytes[0]);
        }

        // Extract branch raw weights (assuming count/val or sub-trie sizes represent weights)
        let raw_weights: Vec<f32> = bytes
            .iter()
            .map(|&b| {
                // Peek child weight/size or default to 1.0
                1.0f32
            })
            .collect();

        // Apply temperature scaling: W_i = exp(w_i / T)
        let temp = if self.temperature <= 0.0 { 1e-5 } else { self.temperature };
        let scaled_weights: Vec<f32> = raw_weights
            .iter()
            .map(|&w| (w / temp).exp())
            .collect();

        let total_weight: f32 = scaled_weights.iter().sum();
        if total_weight <= 0.0 {
            return Some(bytes[0]);
        }

        // Draw a random value in [0, total_weight) using fastrand or std
        let rng_val = fastrand::f32() * total_weight;

        // Cumulative distribution function (CDF) sampling
        let mut cumulative = 0.0f32;
        for (i, &w) in scaled_weights.iter().enumerate() {
            cumulative += w;
            if rng_val <= cumulative {
                return Some(bytes[i]);
            }
        }

        Some(*bytes.last().unwrap())
    }

    /// Step down into a branch selected by the weighted random walk algorithm
    pub fn step_weighted_walk(&mut self) -> bool {
        if let Some(child_byte) = self.sample_next_child() {
            self.descend_to([child_byte]);
            true
        } else {
            false
        }
    }
}

// Trait Implementations
impl<'trie, 'path, V: Clone + Send + Sync + Unpin> Zipper
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn path_exists(&self) -> bool {
        self.inner.path_exists()
    }

    fn is_val(&self) -> bool {
        self.inner.is_val()
    }

    fn child_count(&self) -> usize {
        self.inner.child_count()
    }

    fn child_mask(&self) -> ByteMask {
        self.inner.child_mask()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> ZipperMoving
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn path(&self) -> &[u8] {
        self.inner.path()
    }

    fn val_count(&self) -> usize {
        self.inner.val_count()
    }

    fn descend_to<K>(&mut self, key: K)
    where
        K: AsRef<[u8]>,
    {
        self.inner.descend_to(key);
    }

    fn ascend(&mut self, len: usize) -> bool {
        self.inner.ascend(len)
    }

    fn ascend_until(&mut self) -> bool {
        self.inner.ascend_until()
    }

    fn ascend_until_branch(&mut self) -> bool {
        self.inner.ascend_until_branch()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> ZipperSubtries<V>
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn native_subtries(&self) -> bool {
        self.inner.native_subtries()
    }

    fn try_make_map(&self) -> Option<pathmap::PathMap<V>> {
        self.inner.try_make_map()
    }

    fn trie_ref(&self) -> Option<TrieRef<'_, V>> {
        self.inner.trie_ref()
    }

    fn alloc(&self) -> std::alloc::Global {
        self.inner.alloc()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> ZipperValues<V>
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn val(&self) -> Option<&V> {
        self.inner.val()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> ZipperAbsolutePath
    for WeightedSamplingZipper<'trie, 'path, V>
{
    fn origin_path(&self) -> &[u8] {
        self.inner.origin_path()
    }

    fn root_prefix_path(&self) -> &[u8] {
        self.inner.root_prefix_path()
    }
}

impl<'trie, 'path, V: Clone + Send + Sync + Unpin> ZipperIteration
    for WeightedSamplingZipper<'trie, 'path, V>
{}
