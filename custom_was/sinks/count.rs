use super::PathSink;

/// An aggregating sink that counts incoming paths without storing them.
#[derive(Debug, Default, Clone)]
pub struct CountSink {
    count: usize,
}

impl CountSink {
    /// Create a new, zeroed CountSink.
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Returns the total number of paths processed so far.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Resets the internal counter back to zero.
    pub fn reset(&mut self) {
        self.count = 0;
    }
}

impl<P> PathSink<P> for CountSink {
    fn sink(&mut self, _path: P) {
        self.count += 1;
    }

    fn finalize(&mut self) -> bool {
        self.count > 0
    }
}
