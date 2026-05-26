use std::f32::consts::SQRT_2;

use crate::mcts::graph::{Edge, Node};

/// Defines how to compute the selection score of a child node based on
/// its parent and the edge connecting them.
pub trait SelectionPolicy: Send + Sync {
	/// Returns the selection score of the child node.
	fn score(&self, parent: &Node, edge: &Edge, child: &Node) -> f32;
}

/// Selects the child node with the highest UCB1 score for exploration.
pub struct Ucb1 {
	/// The exploration constant used in the UCB1 formula.
	/// Higher values encourage exploration, lower values exploitation.
	pub exploration_constant: f32,
}

/// Like [`Ucb1`] but with a variance-aware exploration bonus.
///
/// Incorporates an upper bound on the reward variance, making the
/// exploration bonus tighter when variance is low.
pub struct Ucb1Tuned {
	/// Cap on the variance estimate. For rewards in [0, 1] use 0.25,
	/// and for rewards in [-1, 1] use 1.0.
	pub variance_cap: f32,
}

/// Like [`Ucb1`] augmented with a Beta-distribution prior over rewards.
///
/// The prior injects `alpha` virtual successes and `beta` virtual failures,
/// smoothing the mean estimate when a node has few visits and biasing
/// uninformed nodes towards the prior mean.
pub struct BayesianUct {
	/// The exploration constant used in the UCB1 formula.
	/// Higher values encourage exploration, lower values exploitation.
	pub exploration_constant: f32,

	/// Prior success count (virtual wins).
	pub alpha: f32,

	/// Prior failure count (virtual losses).
	pub beta: f32,
}

impl Default for Box<dyn SelectionPolicy> {
	fn default() -> Self {
		Box::new(Ucb1 {
			exploration_constant: SQRT_2,
		})
	}
}

impl SelectionPolicy for Ucb1 {
	fn score(&self, parent: &Node, edge: &Edge, _child: &Node) -> f32 {
		if edge.visits == 0 {
			// Unvisited edges are always preferred
			return f32::INFINITY;
		}

		let exploitation = edge.score / (edge.visits as f32);
		let exploration =
			self.exploration_constant * ((parent.visits as f32).ln() / (edge.visits as f32)).sqrt();

		exploitation + exploration
	}
}

impl SelectionPolicy for Ucb1Tuned {
	fn score(&self, parent: &Node, edge: &Edge, _child: &Node) -> f32 {
		if edge.visits == 0 {
			return f32::INFINITY;
		}

		let n = parent.visits as f32;
		let n_i = edge.visits as f32;
		let mean = edge.score / n_i;
		let mean_sq = edge.score_squared / n_i;

		let variance = (mean_sq - mean * mean + (2.0 * n.ln() / n_i).sqrt()).min(self.variance_cap);

		(n.ln() / n_i * variance).sqrt() + mean
	}
}

impl SelectionPolicy for BayesianUct {
	fn score(&self, parent: &Node, edge: &Edge, _child: &Node) -> f32 {
		if edge.visits == 0 {
			return f32::INFINITY;
		}

		let effective_n = edge.visits as f32 + self.alpha + self.beta;
		let mean = (edge.score + self.alpha) / effective_n;
		let exploration =
			self.exploration_constant * ((parent.visits as f32).ln() / effective_n).sqrt();

		mean + exploration
	}
}
