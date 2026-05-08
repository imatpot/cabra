use std::io::{self, Write};

use crate::{
	caminos::{placement::Placement, state::GameState},
	mcts::{
		graph::{Edge, EdgeIndex, Graph, Node, NodeId},
		policy::{
			action::ActionPolicy,
			computation::ComputationalLimit,
			expansion::{ExpansionPolicy, ExpansionPredicate},
			parallelization::ParallelizationPolicy,
			reward::RewardPolicy,
			rollout::{RolloutPolicy, RolloutResult},
			selection::SelectionPolicy,
		},
	},
	util::ansi,
};

/// A Monte Carlo Tree Search (MCTS) agent.
pub struct MctsAgent {
	/// The (potentially prepolulated) search graph used by this agent.
	pub graph: Graph,

	/// The configuration of the agent.
	pub config: MctsAgentConfig,

	/// The number of threads to use for rollouts.
	rollout_threads: u8,

	/// TODO: i know what this is but dunno how to describe it
	root_threads: u8,
}

impl MctsAgent {
	/// Finds the best next placement for the given game state
	/// using Monte Carlo Tree Search.
	pub fn search_best_placement(&mut self, origin: &GameState) -> Option<&'static Placement> {
		let id = origin.as_node_id();

		if self
			.graph
			.nodes
			.entry(id)
			.or_insert_with(|| Node::new(origin.clone()))
			.is_terminal()
		{
			// No placement can be made from a terminal state
			return None;
		}

		print!("{}Iterating... ", ansi::DIM);
		io::stdout().flush().ok();

		let mut iterations = 0;
		let mut computational_limit_not_exhausted = self.config.computational_limit.predicate();
		while computational_limit_not_exhausted() {
			self.iterate(id);
			iterations += 1;
		}

		println!("iterated {iterations} times{}", ansi::RESET);

		// Return the placement that leads to the best child node according to the win policy
		self.config.action_policy.select(
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

	/// Performs a single MCTS iteration, starting from the given node ID.
	/// It consists of the following steps:
	///
	/// 1. Selection: Starting from the active game state as the root,
	///    recursively select attractive child nodes until reaching a leaf node.
	///
	/// 2. Expansion: If the leaf node is not terminal and should be expanded, expand it by adding a new
	///    child node corresponding to an unexplored move.
	///
	/// 3. Simulation: Roll out a full game from the new child node.
	///    This samples a possible future trajectory of the game and its result,
	///    which is used to evaluate the new child node.
	///
	/// 4. Backpropagation: Update the visit counts and scores of all nodes
	///    and edges along the path from the new child node back to the root
	///    based on the game result and the scoring policy.
	fn iterate(&mut self, origin_id: NodeId) {
		let (leaf_id, mut path) = self.select(origin_id);

		let (rollout_node_id, rollout) = match self.expand(&leaf_id) {
			Some((edge_index, child_id, child_state)) => {
				path.push((leaf_id, edge_index));
				(child_id, self.rollout(&child_state))
			}
			None => {
				// Terminal node; no rollout required
				let result = self.graph.nodes.get(&leaf_id).unwrap().result.unwrap();
				(leaf_id, RolloutResult { result, depth: 0 })
			}
		};

		self.backpropagate(&path, &rollout_node_id, &rollout);
	}

	/// Selects a leaf node to expand, starting from the given origin node ID.
	fn select(&self, origin_id: NodeId) -> (NodeId, Vec<MctsSelection>) {
		let mut path = Vec::<MctsSelection>::new();
		let mut current_id = origin_id;

		loop {
			let node = self.graph.nodes.get(&current_id).unwrap();

			if node.is_terminal() || self.config.expansion_predicate.should_expand(node) {
				return (current_id, path);
			}

			let (best_edge_to_child, best_child_id) = node
				.children
				.iter()
				.enumerate()
				.max_by(|(_, edge_to_a), (_, edge_to_b)| {
					let a = self.graph.nodes.get(&edge_to_a.child_id).unwrap();
					let b = self.graph.nodes.get(&edge_to_b.child_id).unwrap();
					let score_a = self.config.selection_policy.score(node, edge_to_a, a);
					let score_b = self.config.selection_policy.score(node, edge_to_b, b);
					score_a.total_cmp(&score_b)
				})
				.map(|(i, e)| (i, e.child_id))
				.unwrap();

			path.push((current_id, best_edge_to_child));
			current_id = best_child_id;
		}
	}

	/// Expands the given node by adding a new, unexplored child node.
	fn expand(&mut self, node_id: &NodeId) -> Option<MctsExpansion> {
		let node = self.graph.nodes.get_mut(&node_id).unwrap();

		if !self.config.expansion_predicate.should_expand(&node) {
			return None;
		}

		let placement = self
			.config
			.expansion_policy
			.expand(&mut node.unexplored_placements);

		let mut child_state = node.state;
		child_state.apply_placement(placement);
		let child_id = child_state.as_node_id();

		node.children.push(Edge::new(placement, child_id));
		let edge_index = node.children.len() - 1;

		let child = self
			.graph
			.nodes
			.entry(child_id)
			.or_insert(Node::new(child_state.clone()));

		child.parents.insert(*node_id);

		Some((edge_index, child_id, child_state))
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
		terminal_id: &NodeId,
		rollout: &RolloutResult,
	) {
		// Yes, I "back"propagate from the top down instead of bottom up.
		// This way, I don't need to reverse the path order though!

		for &(node_id, edge_index) in path.iter() {
			let node = self.graph.nodes.get_mut(&node_id).unwrap();

			let node_score = Self::node_score(&self.config.reward_policy, &node, &rollout);
			let edge_score = Self::edge_score(&self.config.reward_policy, &node, &rollout);

			node.visit(node_score);
			node.children[edge_index].score += edge_score;
		}

		let terminal = self.graph.nodes.get_mut(&terminal_id).unwrap();
		let terminal_score = Self::node_score(&self.config.reward_policy, &terminal, &rollout);
		terminal.visit(terminal_score);
	}

	/// Score from the perspective of the player who moved to this node.
	fn node_score(reward_policy: &RewardPolicy, node: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(&rollout.result, &rollout.depth, &node.state.last_player())
	}

	/// Score from the perspective of the player who made the edge's move.
	fn edge_score(reward_policy: &RewardPolicy, parent: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(&rollout.result, &rollout.depth, &parent.state.next_player())
	}

	pub fn new(config: MctsAgentConfig) -> Self {
		let rollout_threads = match config.parallelization_policy {
			ParallelizationPolicy::SingeNodeMultipleRollouts { threads } => threads,
			ParallelizationPolicy::MultipleNodesSingleRollout { .. } => 1,
		};

		let root_threads = match config.parallelization_policy {
			ParallelizationPolicy::SingeNodeMultipleRollouts { .. } => 1,
			ParallelizationPolicy::MultipleNodesSingleRollout { threads } => threads,
		};

		Self {
			graph: Graph::new(),
			config,
			rollout_threads,
			root_threads,
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
	pub rollout_policy: Box<dyn RolloutPolicy>,

	/// Determines the best move based on the properties of the child nodes.
	pub action_policy: Box<dyn ActionPolicy>,

	/// Determines how the MCTS iterations should be parallelized.
	pub parallelization_policy: ParallelizationPolicy,
}

/// Pairing of a [`NodeId`] and the used index in its [`Node::children`].
type MctsSelection = (NodeId, EdgeIndex);

/// The result of an expansion, containing
/// the [`EdgeIndex`] of the used edge,
/// the [`NodeId`] of the expanded child,
/// and the expanded node's [`GameState`].
type MctsExpansion = (EdgeIndex, NodeId, GameState);
