use std::sync::atomic::{AtomicU64, Ordering};

use chacha20::ChaCha8Rng;
use rand::{Rng, SeedableRng, rng};

use crate::{
	caminos::placement::{Placement, PlacementRefs},
	mcts::graph::Node,
};

/// Expands the node in a fixed order, always taking the last unexplored move.
pub struct ExpandInOrder;

/// Expands the node by randomly selecting one of the unexplored moves.
pub struct ExpandRandomly {
	/// The random number generator used to select moves during expansion.
	rng_seed: u64,

	/// Atomic counter used to generate unique seeds for each expansion.
	rng_counter: AtomicU64,
}

impl ExpandRandomly {
	/// Creates a new `ExpandRandomly` expansion policy with the given
	/// random number generator.
	pub fn seeded(seed: u64) -> Self {
		Self {
			rng_seed: seed,
			rng_counter: AtomicU64::new(0),
		}
	}

	/// Creates a new `ExpandRandomly` expansion policy with a default random
	/// number generator.
	pub fn unseeded() -> Self {
		Self {
			rng_seed: rng().next_u64(),
			rng_counter: AtomicU64::new(0),
		}
	}
}

/// Determines how a node should be expanded.
pub trait ExpansionPolicy: Send + Sync {
	/// Expands the given node and returns the placement
	/// that led to the new child node.
	fn expand(&self, moves: &mut PlacementRefs) -> &'static Placement;
}

impl Default for Box<dyn ExpansionPolicy> {
	fn default() -> Self {
		Box::new(ExpandRandomly::unseeded())
	}
}

impl ExpansionPolicy for ExpandInOrder {
	fn expand(&self, nodes: &mut PlacementRefs) -> &'static Placement {
		nodes.pop().unwrap()
	}
}

impl ExpansionPolicy for ExpandRandomly {
	fn expand(&self, nodes: &mut PlacementRefs) -> &'static Placement {
		let mut rng = ChaCha8Rng::seed_from_u64(self.rng_seed);
		rng.set_stream(self.rng_counter.fetch_add(1, Ordering::Relaxed));

		let i = rng.next_u64() as usize % nodes.len();
		nodes.swap_remove(i)
	}
}

/// Always expand the node when it has unexplored children,
/// regardless of its properties.
pub struct ExpandAlways;

/// Expand the node whenever it has been visited at least `n` times.
pub struct ExpandWhenVisited {
	pub times: u32,
}

/// Determines whether a node should be expanded based on its properties.
pub trait ExpansionPredicate: Send + Sync {
	/// Returns `true` if the node should be expanded.
	fn should_expand(&self, node: &Node) -> bool;
}

impl Default for Box<dyn ExpansionPredicate> {
	fn default() -> Self {
		Box::new(ExpandAlways)
	}
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
