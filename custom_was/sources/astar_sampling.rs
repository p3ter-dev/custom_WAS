use super::PathSource;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Trait defining cost heuristics for A* traversal
pub trait Heuristic<Node> {
    /// Estimate remaining cost h(n) from the current node to the target
    fn estimate_cost(&self, node: &Node) -> f64;
    /// Calculate step transition cost g(n_parent -> n_child)
    fn step_cost(&self, parent: &Node, child: &Node) -> f64;
}

/// A node wrapper tracking path costs f(n) = g(n) + h(n)
#[derive(Debug, Clone)]
pub struct AStarNode<Node> {
    pub data: Node,
    pub g_cost: f64, // Cost from start to current node
    pub h_cost: f64, // Estimated cost from current node to goal
    pub f_cost: f64, // Total estimated path cost
}

impl<Node> AStarNode<Node> {
    pub fn new(data: Node, g_cost: f64, h_cost: f64) -> Self {
        Self {
            data,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
        }
    }
}

// Order nodes in reverse so BinaryHeap acts as a Min-Heap (lowest f_cost at root)
impl<Node> PartialEq for AStarNode<Node> {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl<Node> Eq for AStarNode<Node> {}

impl<Node> Ord for AStarNode<Node> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
            .reverse()
    }
}

impl<Node> PartialOrd for AStarNode<Node> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* Source with Sampling Zipper Branching
pub struct AStarSamplingSource<Node, H, F>
where
    H: Heuristic<Node>,
    F: Fn(&Node) -> Vec<Node>, // Zipper branch expansion function
{
    open_set: BinaryHeap<AStarNode<Node>>,
    heuristic: H,
    expand_children: F,
    sample_k: usize, // Max child branches to sample at each step
}

impl<Node, H, F> AStarSamplingSource<Node, H, F>
where
    H: Heuristic<Node>,
    F: Fn(&Node) -> Vec<Node>,
    Node: Clone,
{
    /// Create a new A* Sampling Source
    ///
    /// * `start` - The root node / starting zipper position
    /// * `heuristic` - Struct implementing Heuristic<Node>
    /// * `expand_children` - Closure expanding child branches under current node
    /// * `sample_k` - Maximum number of child branches to sample per node expansion
    pub fn new(start: Node, heuristic: H, expand_children: F, sample_k: usize) -> Self {
        assert!(sample_k > 0, "sample_k must be greater than 0");

        let h_cost = heuristic.estimate_cost(&start);
        let root_node = AStarNode::new(start, 0.0, h_cost);

        let mut open_set = BinaryHeap::new();
        open_set.push(root_node);

        Self {
            open_set,
            heuristic,
            expand_children,
            sample_k,
        }
    }
}

impl<Node, H, F> PathSource for AStarSamplingSource<Node, H, F>
where
    H: Heuristic<Node>,
    F: Fn(&Node) -> Vec<Node>,
    Node: Clone,
{
    type Item = AStarNode<Node>;

    fn next_path(&mut self) -> Option<Self::Item> {
        // Pop node with the lowest f_cost
        let current = self.open_set.pop()?;

        // 1. Zipper expansion: Get all valid child branches under current node
        let children = (self.expand_children)(&current.data);

        if !children.is_empty() {
            // 2. Score candidate children using g(n) + h(n)
            let mut scored_children: Vec<AStarNode<Node>> = children
                .into_iter()
                .map(|child| {
                    let step = self.heuristic.step_cost(&current.data, &child);
                    let g = current.g_cost + step;
                    let h = self.heuristic.estimate_cost(&child);
                    AStarNode::new(child, g, h)
                })
                .collect();

            // 3. SAMPLING ZIPPER STEP: Sort children by f_cost & sample top-k
            scored_children.sort_by(|a, b| {
                a.f_cost
                    .partial_cmp(&b.f_cost)
                    .unwrap_or(Ordering::Equal)
            });

            // Push only the best 'sample_k' branches back into the A* priority queue
            for child_node in scored_children.into_iter().take(self.sample_k) {
                self.open_set.push(child_node);
            }
        }

        Some(current)
    }
}
