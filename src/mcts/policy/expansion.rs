use rand::Rng;

use crate::{caminos::placement::{Placement, PlacementRefs}, mcts::graph::Node};

/// Expands the node in a fixed order, always taking the last unexplored move.
pub struct ExpandInOrder;

/// Expands the node by randomly selecting one of the unexplored moves.
pub struct ExpandRandomly {
	/// The random number generator used to select moves during expansion.
	rng: Box<dyn Rng>,
}

impl ExpandRandomly {
	/// Creates a new `ExpandRandomly` expansion policy with the given
	/// random number generator.
	pub fn seeded(rng: Box<dyn Rng>) -> Self {
		Self { rng }
	}

	/// Creates a new `ExpandRandomly` expansion policy with a default random
	/// number generator.
	pub fn unseeded() -> Self {
		Self {
			rng: Box::new(rand::rng()),
		}
	}
}

/// Determines how a node should be expanded.
pub trait ExpansionPolicy {
	/// Expands the given node and returns the placement
	/// that led to the new child node.
	fn expand(&mut self, moves: &mut PlacementRefs) -> &'static Placement;
}

impl ExpansionPolicy for ExpandInOrder {
	fn expand(&mut self, nodes: &mut PlacementRefs) -> &'static Placement {
		nodes.pop().unwrap()
	}
}

impl ExpansionPolicy for ExpandRandomly {
	fn expand(&mut self, nodes: &mut PlacementRefs) -> &'static Placement {
		let i = self.rng.next_u64() as usize % nodes.len();
		nodes.swap_remove(i)
	}
}

/// Always expand the node when it has unexplored children,
/// regardless of its properties.
pub struct ExpandAlways;

/// Expand the node whenever it has been visited at least `n` times.
pub struct ExpandWhenVisited {
	times: u32,
}

/// Determines whether a node should be expanded based on its properties.
pub trait ExpansionPredicate {
	/// Returns `true` if the node should be expanded.
	fn should_expand(&self, node: &Node) -> bool;
}

impl ExpansionPredicate for ExpandAlways {
	fn should_expand(&self, node: &Node) -> bool {
		!node.unexplored_placements.is_empty()
	}
}

impl ExpansionPredicate for ExpandWhenVisited {
	fn should_expand(&self, node: &Node) -> bool {
		node.visits >= self.times
	}
}
