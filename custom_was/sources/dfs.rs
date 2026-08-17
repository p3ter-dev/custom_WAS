use super::PathSource;
use std::collections::VecDeque;

/// Depth-First Search Path Source
pub struct DFSSource<N> {
    stack: Vec<N>,
}

impl<N> DFSSource<N> {
    pub fn new(root: N) -> Self {
        Self { stack: vec![root] }
    }
}

impl<N> PathSource for DFSSource<N>
where
    N: Clone,
{
    type Item = N;

    fn next_path(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
