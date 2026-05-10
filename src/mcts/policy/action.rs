use crate::{
	caminos::placement::Placement,
	mcts::graph::{Edge, Node},
};

/// A reachable node in the search graph,
/// consisting of an edge and its corresponding child node.
pub type ReachableNode<'a> = (&'a Edge, &'a Node);

/// Select the root child with the highest average reward (score over visits).
pub struct MaxChild;

/// Select the most visited root child.
pub struct RobustChild;

/// Select the root child with both the highest visit count
/// and the highest score.
pub struct MaxRobustChild;

/// Select the child which maximizes an upper confidence bound on the win rate.
pub struct SecureChild {
	/// Scales the confidence bonus. Higher values favor nodes with few visits.
	security_constant: f32,
}

/// Determines the best move based on the properties of the child nodes
/// of the root node.
pub trait ActionPolicy: Send + Sync {
	/// Selects the winning move from the given child nodes of the root node.
	/// Returns `None` if there are no child nodes satisfying the criteria.
	fn select(&self, nodes: &[ReachableNode]) -> Option<&'static Placement>;
}

impl Default for Box<dyn ActionPolicy> {
	fn default() -> Self {
		Box::new(RobustChild)
	}
}

impl ActionPolicy for MaxChild {
	fn select<'a>(&self, nodes: &[ReachableNode]) -> Option<&'static Placement> {
		nodes
			.iter()
			.max_by(|(_, a), (_, b)| {
				let q = |n: &Node| n.score / (n.visits + 1) as f32;
				q(a).total_cmp(&q(b))
			})
			.map(|(edge, _)| edge.placement)
	}
}

impl ActionPolicy for RobustChild {
	fn select<'a>(&self, nodes: &[ReachableNode]) -> Option<&'static Placement> {
		nodes
			.iter()
			.max_by_key(|(_, child)| child.visits)
			.map(|(edge, _)| edge.placement)
	}
}

impl ActionPolicy for MaxRobustChild {
	fn select<'a>(&self, nodes: &[ReachableNode]) -> Option<&'static Placement> {
		nodes
			.iter()
			.max_by(|(_, a), (_, b)| {
				a.visits
					.cmp(&b.visits)
					.then_with(|| a.score.total_cmp(&b.score))
			})
			.map(|(edge, _)| edge.placement)
	}
}

impl ActionPolicy for SecureChild {
	fn select<'a>(&self, nodes: &[ReachableNode]) -> Option<&'static Placement> {
		nodes
			.iter()
			.max_by(|(_, a), (_, b)| {
				let score = |n: &Node| {
					let q = n.score / (n.visits + 1) as f32;
					let bonus = self.security_constant / ((n.visits + 1) as f32).sqrt();
					q + bonus
				};

				score(a).total_cmp(&score(b))
			})
			.map(|(edge, _)| edge.placement)
	}
}
