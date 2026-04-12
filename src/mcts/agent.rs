use std::{
	io::{self, Write},
	time::{Duration, Instant},
};

use crate::{
	caminos::{
		placement::Placement,
		state::{GameResult, GameState},
	},
	mcts::{
		graph::{Edge, Graph, Node, NodeId},
		policy::{
			expansion::{ExpansionPolicy, ExpansionPredicate},
			rollout::RolloutPolicy,
			scoring::ScoringPolicy,
			selection::SelectionPolicy,
			win::WinPolicy,
		},
	},
	util::ansi,
};

/// A Monte Carlo Tree Search (MCTS) agent.
pub struct MctsAgent {
	/// The (potentially prepolulated) search graph used by this agent.
	pub graph: Graph,

	/// The computational limit for this agent.
	pub computational_limit: Box<dyn ComputationalLimit>,

	/// A mapping from [`GameResult`]s to their corresponding scores
	/// used during backpropagation.
	pub scoring_policy: ScoringPolicy,

	/// Determines the score of a node during the selection phase.
	pub selection_policy: Box<dyn SelectionPolicy>,

	/// Determines whether a node should be expanded.
	pub expansion_predicate: Box<dyn ExpansionPredicate>,

	/// Determines how a node should be expanded,
	/// i.e. which unexplored move should be taken.
	pub expansion_policy: Box<dyn ExpansionPolicy>,

	/// Simulates a full playout from the given node.
	pub rollout_policy: Box<dyn RolloutPolicy>,

	/// Determines the best move based on the properties of the child nodes.
	pub win_policy: Box<dyn WinPolicy>,
}

impl MctsAgent {
	/// Finds the best next placement for the given game state
	/// using Monte Carlo Tree Search.
	pub fn find_best_placement(&mut self, origin: &GameState) -> Option<&'static Placement> {
		let id = origin.as_node_id();

		if self
			.graph
			.nodes
			.entry(id)
			.or_insert_with(|| Node::new(origin.clone(), &[]))
			.is_terminal()
		{
			// No placement can be made from a terminal state
			return None;
		}

		print!("{}Iterating... ", ansi::DIM);
		io::stdout().flush().ok();

		let mut iterations = 0;
		let mut computational_limit_not_exhausted = self.computational_limit.predicate();
		while computational_limit_not_exhausted() {
			self.iterate(&id);
			iterations += 1;
		}

		println!("iterated {iterations} times{}", ansi::RESET);

		// Return the placement that leads to the best child node according to the win policy
		self.win_policy.select_winner(
			&self
				.graph
				.nodes
				.get(&id)
				.unwrap()
				.children
				.iter()
				.map(|edge| (edge, self.graph.nodes.get(&edge.child_id).unwrap()))
				.collect::<Vec<_>>(),
		)
	}

	/// Performs one iteration of the Monte Carlo Tree Search algorithm,
	/// starting from the node with the given ID.
	/// It consists of the following steps:
	///
	/// 1. Selection: Starting from the root, recursively select child nodes
	///    according to the selection policy until a leaf node is reached.
	///
	/// 2. Expansion: If the leaf node is not terminal and should be expanded
	///    according to the expansion predicate, expand it by adding a new
	///    child node corresponding to an unexplored move.
	///
	/// 3. Simulation: Simulate a full playout from the new child node using
	///    the rollout policy to obtain a game result.
	///
	/// 4. Backpropagation: Update the visit counts and scores of all nodes
	///    and edges along the path from the new child node back to the root
	///    based on the game result and the scoring policy.
	fn iterate(&mut self, node_id: &NodeId) -> GameResult {
		// This function re-fetches the node multiple times.
		// This was done to avoid mut and non-mut re-borrowing issues.

		let node_ref = self.graph.nodes.get(&node_id).unwrap();

		if let Some(result) = node_ref.result {
			// The node is terminal,
			// so we can backpropagate the result without further exploration

			let node = self.graph.nodes.get_mut(&node_id).unwrap();
			let score = self.scoring_policy.score(&result, !node.state.next_player);
			node.visit(score);

			return result;
		}

		if self.expansion_predicate.should_expand(node_ref) {
			// The node should be expanded,
			// so we create a new child node by taking an unexplored placement

			let node_mut = self.graph.nodes.get_mut(&node_id).unwrap();
			let expanded_placement = self
				.expansion_policy
				.expand(&mut node_mut.unexplored_placements);

			// Get the child state by applying the expanded placement

			let mut child_state = node_mut.state.clone();
			child_state.apply_placement(expanded_placement);

			let child_id = child_state.as_node_id();

			let mut edge = Edge::new(expanded_placement, child_id);

			// Simulate a playout from the child state to get a game result

			let result = self.rollout_policy.rollout(&child_state);

			node_mut.visit(
				self.scoring_policy
					.score(&result, !node_mut.state.next_player),
			);

			// Calculate the score for the child and update the graph

			let child_score = self.scoring_policy.score(&result, !child_state.next_player);
			edge.visit(child_score);

			node_mut.children.push(edge);

			let child = self
				.graph
				.nodes
				.entry(child_id)
				.or_insert_with(|| Node::new(child_state, &[]));

			child.visit(child_score);

			if !child.parents.contains(node_id) {
				// New, unseen parent-child connection
				child.parents.push(*node_id);
			}

			return result;
		} else {
			// The node should not be expanded,
			// so we select the best child node according to the selection policy

			let (best_edge_index, best_child_id) = node_ref
				.children
				.iter()
				.enumerate()
				.max_by(|(_, edge_a), (_, edge_b)| {
					let a = self.graph.nodes.get(&edge_a.child_id).unwrap();
					let b = self.graph.nodes.get(&edge_b.child_id).unwrap();
					let score_a = self.selection_policy.score(node_ref, edge_a, a);
					let score_b = self.selection_policy.score(node_ref, edge_b, b);
					score_a.total_cmp(&score_b)
				})
				.map(|(index, edge)| (index, edge.child_id))
				.unwrap();

			// Recursively iterate on the best child node to get a game result

			let result = self.iterate(&best_child_id);
			let node_mut = self.graph.nodes.get_mut(&node_id).unwrap();

			// Backpropagate the result

			node_mut.visit(
				self.scoring_policy
					.score(&result, !node_mut.state.next_player),
			);

			node_mut.children[best_edge_index].visit(
				self.scoring_policy
					.score(&result, node_mut.state.next_player),
			);

			return result;
		}
	}
}

/// A computational limit based on a fixed number of iterations.
pub struct IterativeComputationalLimit {
	pub iterations: u32,
}

/// A computational limit based on a fixed duration of time.
pub struct TemporalComputationalLimit {
	pub duration: Duration,
}

/// A trait representing a computational limit, allowing execution of a callback
/// until the limit is exhausted.
pub trait ComputationalLimit {
	/// Returns a predicate that returns `true` until the limit is exhausted.
	fn predicate(&self) -> Box<dyn FnMut() -> bool>;
}

impl ComputationalLimit for IterativeComputationalLimit {
	fn predicate(&self) -> Box<dyn FnMut() -> bool> {
		let mut remaining = self.iterations;

		Box::new(move || {
			if remaining == 0 {
				return false;
			}
			remaining -= 1;
			true
		})
	}
}

impl ComputationalLimit for TemporalComputationalLimit {
	fn predicate(&self) -> Box<dyn FnMut() -> bool> {
		let deadline = Instant::now() + self.duration;
		Box::new(move || Instant::now() < deadline)
	}
}
