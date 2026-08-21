pub mod astar_sample;
pub mod bfs;
pub mod dfs;

pub use astar_sample::{AStarNode, AStarSamplingSource, Heuristic};
pub use bfs::BFSSource;
pub use dfs::DFSSource;

pub trait PathSource {
    type Item;

    fn next_path(&mut self) -> Option<Self::Item>;
}
