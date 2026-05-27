use std::{
	io::{self, Write},
	time::{Duration, Instant},
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
	caminos::{placement::Placement, state::GameState},
	mcts::{
		graph::{Edge, EdgeIndex, Graph, Node, NodeIndex},
		policy::{
			action::ActionPolicy,
			computation::{ComputationalIntensity, ComputationalLimit},
			expansion::{ExpansionPolicy, ExpansionPredicate},
			reward::RewardPolicy,
			rollout::{RolloutPolicy, RolloutResult},
			selection::SelectionPolicy,
		},
	},
};

/// A Monte Carlo Tree Search (MCTS) agent.
pub struct MctsAgent {
	/// The (potentially prepolulated) search graph used by this agent.
	pub graph: Graph,

	/// The configuration of the agent.
	pub config: MctsAgentConfig,
}

/// The result of a Monte Carlo Tree Search (MCTS) search.
pub struct MctsResult {
	/// The best placement found by the search, if any.
	pub placement: Option<&'static Placement>,

	/// The number of iterations performed during the search.
	pub iterations: u32,

	/// The duration of the search.
	pub duration: Duration,
}

impl MctsAgent {
	/// Finds the best next placement for the given game state
	/// using Monte Carlo Tree Search.
	pub fn search_best_placement(&mut self, origin: GameState) -> MctsResult {
		let start = Instant::now();
		let origin_index = self.graph.index(origin);

		if self.graph.node(origin_index).is_terminal() {
			// No placement can be made from a terminal state
			return MctsResult {
				placement: None,
				iterations: 0,
				duration: start.elapsed(),
			};
		}

		io::stdout().flush().ok();

		let mut iterations = 0;
		let mut computational_limit_not_exhausted = self.config.computational_limit.predicate();

		while computational_limit_not_exhausted() {
			self.iterate(origin_index);
			iterations += 1;
		}

		// Return the placement that leads to the best child node according to the win policy
		let children = self
			.graph
			.node(origin_index)
			.children
			.iter()
			.map(|edge| (edge, self.graph.node(edge.child_index)))
			.collect::<Vec<_>>();

		let placement = self.config.action_policy.select(&children);

		MctsResult {
			placement,
			iterations,
			duration: start.elapsed(),
		}
	}

	/// Performs a single MCTS iteration, starting from the given node ID.
	/// It consists of the following steps:
	///
	/// 1. Selection: Starting from the active game state as the root,
	///    recursively select attractive child nodes until reaching a leaf node.
	///
	/// 2. Expansion: If the leaf node is not terminal and should be expanded,
	///    expand it by adding a new unexplored child node.
	///
	/// 3. Simulation: Roll out a full game from the new child node.
	///    This samples a possible future trajectory of the game and its result,
	///    which is used to evaluate the new child node.
	///
	/// 4. Backpropagation: Update the visit counts and scores of all nodes
	///    and edges along the path from the new child node back to the root
	///    based on the game result and the scoring policy.

	pub fn iterate(&mut self, origin_index: NodeIndex) {
		let (leaf_index, mut path) = self.select(origin_index);

		if let Some((edge_index, child_index)) = self.expand(leaf_index) {
			path.push((leaf_index, edge_index));

			let child_state = self.graph.node(child_index).state;
			let rollouts: Vec<RolloutResult> =
				(0..self.config.computational_intensity.rollouts_per_node)
					.into_par_iter()
					.map(|_| self.rollout(&child_state))
					.collect();

			self.backpropagate(&path, child_index, &rollouts);
		} else {
			let leaf = self.graph.node(leaf_index);

			if let Some(result) = leaf.state.result {
				// Node is terminal -> backpropagate immediately
				self.backpropagate(
					&path,
					leaf_index,
					&[RolloutResult {
						result,
						depth: 0,
						num_biased_moves: 0,
					}],
				);
			} else {
				// Node was not terminal but couldn't (yet) be expanded
				let state = leaf.state;

				let rollouts: Vec<RolloutResult> =
					(0..self.config.computational_intensity.rollouts_per_node)
						.into_par_iter()
						.map(|_| self.rollout(&state))
						.collect();

				self.backpropagate(&path, leaf_index, &rollouts);
			}
		}
	}

	/// Selects a leaf node to expand, starting from the given origin node ID.
	fn select(&self, origin_index: NodeIndex) -> (NodeIndex, Vec<MctsSelection>) {
		let mut path = Vec::<MctsSelection>::new();
		let mut current_index = origin_index;

		loop {
			let node = self.graph.node(current_index);

			if node.is_terminal()
				|| node.children.is_empty()
				|| (self.config.expansion_predicate.should_expand(node)
					&& !node.unexplored_placements.is_empty())
			{
				return (current_index, path);
			}

			let (best_edge_index, best_child_index) = node
				.children
				.iter()
				.enumerate()
				.map(|(i, edge)| {
					let score = self.config.selection_policy.score(
						node,
						edge,
						self.graph.node(edge.child_index),
					);

					(i, edge.child_index, score)
				})
				.max_by(|(_, _, score_a), (_, _, score_b)| score_a.total_cmp(score_b))
				.map(|(i, child_index, _)| (i, child_index))
				.unwrap();

			path.push((current_index, best_edge_index));
			current_index = best_child_index;
		}
	}

	/// Expands the given node by adding a new, unexplored child node.
	fn expand(&mut self, node_index: NodeIndex) -> Option<MctsExpansion> {
		let node = self.graph.node(node_index);

		if node.unexplored_placements.is_empty()
			|| !self.config.expansion_predicate.should_expand(node)
		{
			return None;
		}

		let placement = self
			.config
			.expansion_policy
			.expand(&mut self.graph.node_mut(node_index).unexplored_placements);

		let mut child_state = self.graph.node(node_index).state;
		child_state.apply_placement(placement);

		let child_index = self.graph.index(child_state);

		self.graph
			.node_mut(node_index)
			.children
			.push(Edge::new(placement, child_index));

		let edge_index = self.graph.node(node_index).children.len() - 1;
		self.graph.node_mut(child_index).parents.insert(node_index);

		Some((edge_index, child_index))
	}

	/// Run a single rollout.
	fn rollout(&self, state: &GameState) -> RolloutResult {
		self.config.rollout_policy.rollout(state)
	}

	/// Backpropagates the result of a rollout through the path from the given
	/// terminal node ID to the root, updating the visit counts and scores of
	/// all nodes and edges along the path.
	fn backpropagate(
		&mut self,
		path: &[MctsSelection],
		terminal_index: NodeIndex,
		rollouts: &[RolloutResult],
	) {
		// Yes, I "back"propagate from the top down instead of bottom up.
		// This way, I don't need to reverse the path order though!

		let num_rollouts = rollouts.len() as u32;

		for &(node_id, edge_index) in path.iter() {
			let node = self.graph.node_mut(node_id);

			let (node_score, edge_score, edge_score_squared) = rollouts.iter().fold(
				(0.0, 0.0, 0.0),
				|(node_acc, edge_acc, edge_sq_acc), rollout| {
					let edge_score = Self::edge_score(&self.config.reward_policy, node, rollout);
					(
						node_acc + Self::node_score(&self.config.reward_policy, node, rollout),
						edge_acc + edge_score,
						edge_sq_acc + edge_score * edge_score,
					)
				},
			);

			node.visit(num_rollouts, node_score);
			node.children[edge_index].visit(num_rollouts, edge_score, edge_score_squared);
		}

		let terminal = self.graph.node_mut(terminal_index);
		let terminal_score = rollouts
			.par_iter()
			.map(|rollout| Self::node_score(&self.config.reward_policy, terminal, rollout))
			.sum::<f32>();

		terminal.visit(num_rollouts, terminal_score);
	}

	/// Score from the perspective of the player who moved to this node.
	fn node_score(reward_policy: &RewardPolicy, node: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(
			&rollout.result,
			&rollout.depth,
			rollout.num_biased_moves,
			&node.state.last_player(),
		)
	}

	/// Score from the perspective of the player who made the edge's move.
	fn edge_score(reward_policy: &RewardPolicy, parent: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(
			&rollout.result,
			&rollout.depth,
			rollout.num_biased_moves,
			&parent.state.next_player(),
		)
	}

	pub fn new(config: MctsAgentConfig) -> Self {
		Self {
			graph: Graph::new(),
			config,
		}
	}
}

/// The configuration of a Monte Carlo Tree Search (MCTS) agent,
/// which determines its behavior and performance characteristics.
#[derive(Default)]
pub struct MctsAgentConfig {
	/// The computational limit for this agent.
	pub computational_limit: Box<dyn ComputationalLimit>,

	/// A mapping from [`GameResult`]s to their corresponding scores
	/// used during backpropagation.
	pub reward_policy: RewardPolicy,

	/// Determines the score of a node during the selection phase.
	pub selection_policy: Box<dyn SelectionPolicy>,

	/// Determines whether a node should be expanded.
	pub expansion_predicate: Box<dyn ExpansionPredicate>,

	/// Determines how a node should be expanded,
	/// i.e. which unexplored move should be taken.
	pub expansion_policy: Box<dyn ExpansionPolicy>,

	/// Simulates a full playout from the given node.
	pub rollout_policy: RolloutPolicy,

	/// Determines how many rollouts to perform during each iteration.
	pub computational_intensity: ComputationalIntensity,

	/// Determines the best move based on the properties of the child nodes.
	pub action_policy: Box<dyn ActionPolicy>,
}

/// Pairing of a [`NodeIndex`] and the used index in its [`Node::children`].
type MctsSelection = (NodeIndex, EdgeIndex);

/// The result of an expansion: the [`EdgeIndex`] of the used edge
/// and the [`NodeIndex`] of the newly created child.
type MctsExpansion = (EdgeIndex, NodeIndex);
