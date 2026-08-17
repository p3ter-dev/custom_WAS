use super::PathSource;
use std::collections::VecDeque;

/// Breadth-First Search Path Source
pub struct BFSSource<N> {
    queue: VecDeque<N>,
}

impl<N> BFSSource<N> {
    pub fn new(root: N) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(root);
        Self { queue }
    }
}

impl<N> PathSource for BFSSource<N>
where
    N: Clone,
{
    type Item = N;

    fn next_path(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}
