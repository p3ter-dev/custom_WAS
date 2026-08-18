# Custom WAS: Weighted Atom Sampling & Path Processing Engine

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

**Custom WAS** is a high-performance Rust library providing Weighted Atom Sampling (WAS), temperature-scaled random walk trie traversals, path generation sources, and stream aggregation sinks. It is designed to seamlessly integrate with the [MORK](https://github.com/) atom sweep engine and [`pathmap`](https://crates.io/crates/pathmap) prefix trie data structures.

---

## 🌟 Key Features

- **Temperature-Scaled Weighted Sampling (`WeightedSamplingZipper`)**
  - Performs temperature-controlled probabilistic branch selection over `PathMap` trie zippers.
  - Implements Roulette-Wheel / Cumulative Distribution Function (CDF) sampling for dynamic random walks.
  - Full wrapper implementation of `pathmap::zipper` traits (`Zipper`, `ZipperMoving`, `ZipperSubtries`, `ZipperValues`, `ZipperAbsolutePath`, `ZipperIteration`).

- **WAS Source Integration (`WASSource`)**
  - Parses MORK pattern expressions (e.g., `("WAS" _pattern {t: f32})`) to dynamically set sampling temperatures.
  - Wraps untracked trie zippers with `PrefixZipper` for prefix-matched pattern evaluation.

- **Path Sources (`custom_was::sources`)**
  - `BFSSource<N>`: Queue-based Breadth-First Search path generator.
  - `DFSSource<N>`: Stack-based Depth-First Search path generator.
  - Extensible via the unified `PathSource` trait.

- **Path Sinks (`custom_was::sinks`)**
  - `PathSink<P>`: Lifecycle interface (`sink`, `finalize`) for path processors.
  - `CountSink`: Zero-allocation aggregator tracking total emitted items from path sources.
  - `WeightedSampleSink`: Implements **Algorithm A-Res** (Weighted Reservoir Sampling with key $k = u^{1/w}$) using a min-heap priority queue to maintain a bounded weighted sample stream.

---

## 📐 Mathematical Formulation

### 1. Temperature-Scaled Branch Selection
For a given trie node with child branches $i \in \{1, \dots, n\}$ and raw branch weights $w_i$, the effective weight $W_{\text{effective}}[i]$ under temperature $T > 0$ is computed as:

$$W_{\text{effective}}[i] = \exp\left(\frac{w_i}{T}\right)$$

The selection probability $P[i]$ for choosing branch $i$ during a random walk step is:

$$P[i] = \frac{W_{\text{effective}}[i]}{\sum_{j=1}^{n} W_{\text{effective}}[j]}$$

- **High Temperature ($T \to \infty$)**: Uniform random selection across available branches.
- **Low Temperature ($T \to 0^+$)**: Greedy selection favoring the highest-weighted branch.

---

### 2. Weighted Reservoir Sampling (Algorithm A-Res)
When collecting samples with `WeightedSampleSink`, each candidate path $x_i$ with item weight $w_i > 0$ is assigned a key $k_i$:

$$k_i = u_i^{1/w_i}, \quad \text{where } u_i \sim U(0, 1)$$

The sink maintains a min-heap priority queue of size $K$ (`max_samples`). Paths with the largest $K$ keys are retained in the sample reservoir, guaranteeing exact weighted random selection without replacement.

---

## 📁 Repository Structure

```text
custom_WAS/
└── custom_was/
    ├── mod.rs                # Core module entrypoint exposing submodules
    ├── was_source.rs         # WASSource pattern engine integration
    ├── sampling_zipper.rs    # WeightedSamplingZipper trie wrapper & CDF sampler
    ├── sinks/                # Path consumption & aggregation sinks
    │   ├── mod.rs            # PathSink trait definition
    │   ├── count.rs          # CountSink path counter
    │   └── weighted_sample.rs# WeightedSampleSink (Algorithm A-Res)
    └── sources/              # Path generation strategies
        ├── bfs.rs            # BFSSource (Breadth-First Search)
        └── dfs.rs            # DFSSource (Depth-First Search)
```

---

## 🚀 Getting Started

### Prerequisites

Ensure your project includes dependencies for `mork_expr`, `pathmap`, `fastrand`, and `rand`.

```toml
[dependencies]
fastrand = "2.0"
rand = "0.8"
pathmap = { path = "../pathmap" }
mork_expr = { path = "../mork_expr" }
```

---

## 💡 Usage Examples

### 1. Weighted Random Walk on Trie Zippers

```rust
use pathmap::zipper::ReadZipperUntracked;
use custom_was::sampling_zipper::WeightedSamplingZipper;

// Wrap an untracked PathMap reader with a temperature of 0.7
let temperature = 0.7f32;
let mut sampler = WeightedSamplingZipper::new(untracked_zipper, temperature);

// Perform step-by-step weighted random walk traversal
while sampler.step_weighted_walk() {
    println!("Traversed byte step: {:?}", sampler.path());
}
```

---

### 2. Collecting Weighted Samples with `WeightedSampleSink`

```rust
use custom_was::sinks::{PathSink, WeightedSampleSink};

// Create a sink collecting up to 10 weighted samples
let mut sink = WeightedSampleSink::new(10);

// Stream paths with respective weights
sink.sink_weighted(b"atom/path/A", 2.5);
sink.sink_weighted(b"atom/path/B", 0.8);
sink.sink_weighted(b"atom/path/C", 5.0);

// Finalize and extract the reservoir samples
if sink.finalize() {
    let samples: Vec<Vec<u8>> = sink.into_samples();
    for sample in samples {
        println!("Sampled path: {:?}", String::from_utf8_lossy(&sample));
    }
}
```

---

### 3. Path Generation with BFS / DFS Sources

```rust
use custom_was::sources::{BFSSource, DFSSource, PathSource};

// Breadth-First Search traversal
let mut bfs = BFSSource::new("root_node");
while let Some(node) = bfs.next_path() {
    println!("BFS visited: {}", node);
}

// Depth-First Search traversal
let mut dfs = DFSSource::new("root_node");
while let Some(node) = dfs.next_path() {
    println!("DFS visited: {}", node);
}
```

---

## 🛠️ API Reference

### Core Types

| Component | Description |
| :--- | :--- |
| `WASSource` | Evaluates MORK `WAS` expressions and creates prefix-bound weighted sampling zipper sources. |
| `WeightedSamplingZipper` | Trie zipper wrapper performing temperature-scaled CDF sampling over branch byte masks. |
| `PathSource` | Trait for generating sequential path nodes (`next_path`). |
| `BFSSource<N>` | Queue-driven Breadth-First Search path source. |
| `DFSSource<N>` | Stack-driven Depth-First Search path source. |
| `PathSink<P>` | Trait defining path stream consumption (`sink`) and lifecycle completion (`finalize`). |
| `CountSink` | Stream sink counting emitted items without storing path payloads. |
| `WeightedSampleSink` | Reservoir sampling sink maintaining top-$K$ weighted samples using Algorithm A-Res. |

---

## 📜 License

This project is licensed under the MIT License.
