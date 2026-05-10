use std::f32::consts::SQRT_2;

use crate::mcts::graph::{Edge, Node};

/// Selects the child node with the highest UCB1 score for exploration.
pub struct Ucb1 {
	/// The exploration constant used in the UCB1 formula.
	/// Higher values encourage exploration, lower values exploitation.
	pub exploration_constant: f32,
}

/// Defines how to compute the selection score of a child node based on
/// its parent and the edge connecting them.
pub trait SelectionPolicy: Send + Sync {
	/// Returns the selection score of the child node.
	fn score(&self, parent: &Node, edge: &Edge, child: &Node) -> f32;
}

impl Default for Box<dyn SelectionPolicy> {
	fn default() -> Self {
		Box::new(Ucb1 {
			exploration_constant: SQRT_2,
		})
	}
}

impl SelectionPolicy for Ucb1 {
	fn score(&self, parent: &Node, edge: &Edge, child: &Node) -> f32 {
		if edge.visits == 0 {
			// Unvisited edges are always preferred
			return f32::INFINITY;
		}

		let exploitation = child.score / (child.visits as f32);
		let exploration =
			self.exploration_constant * ((parent.visits as f32).ln() / (edge.visits as f32)).sqrt();

		exploitation + exploration
	}
}
