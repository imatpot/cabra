use crate::mcts::graph::{Edge, Node};

/// Selects the child node with the highest UCB1 score for exploration.
pub struct Ucb1 {
	/// The exploration constant used in the UCB1 formula.
	/// Higher values encourage exploration, lower values exploitation.
	pub exploration_constant: f64,
}

/// Defines how to compute the selection score of a child node based on
/// its parent and the edge connecting them.
pub trait SelectionPolicy {
	/// Returns the selection score of the child node.
	fn score(&self, parent: &Node, edge: &Edge, child: &Node) -> f64;
}

impl SelectionPolicy for Ucb1 {
	fn score(&self, parent: &Node, edge: &Edge, child: &Node) -> f64 {
		if edge.visits == 0 {
			// Unvisited edges are always preferred
			return f64::INFINITY;
		}

		let exploitation = child.score / (child.visits as f64);
		let exploration =
			self.exploration_constant * ((parent.visits as f64).ln() / (edge.visits as f64)).sqrt();

		exploitation + exploration
	}
}
