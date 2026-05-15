use std::{
	io::{self, Write},
	time::Instant,
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
	caminos::{placement::Placement, state::GameState},
	mcts::{
		graph::{Edge, EdgeIndex, Graph, Node},
		policy::{
			action::ActionPolicy,
			computation::{ComputationalIntensity, ComputationalLimit},
			expansion::{ExpansionPolicy, ExpansionPredicate},
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
}

impl MctsAgent {
	/// Finds the best next placement for the given game state
	/// using Monte Carlo Tree Search.
	pub fn search_best_placement(&mut self, origin: GameState) -> Option<&'static Placement> {
		if self
			.graph
			.nodes
			.entry(origin)
			.or_insert_with(|| Node::new(origin))
			.is_terminal()
		{
			// No placement can be made from a terminal state
			return None;
		}

		print!("{}Iterating... ", ansi::DIM);
		io::stdout().flush().ok();

		let start = Instant::now();
		let mut iterations = 0;
		let mut computational_limit_not_exhausted = self.config.computational_limit.predicate();

		while computational_limit_not_exhausted() {
			self.iterate(origin);
			iterations += 1;
		}

		let elapsed = Instant::now().duration_since(start);

		println!(
			"iterated {iterations} times in {} ms{}",
			elapsed.as_millis(),
			ansi::RESET
		);

		// Return the placement that leads to the best child node according to the win policy
		self.config.action_policy.select(
			&self
				.graph
				.nodes
				.get(&origin)
				.unwrap()
				.children
				.iter()
				.map(|edge| (edge, self.graph.nodes.get(&edge.child_state).unwrap()))
				.collect::<Vec<_>>(),
		)
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
	pub fn iterate(&mut self, origin: GameState) {
		let (leaf_id, mut path) = self.select(origin);

		if let Some((edge_index, child_state)) = self.expand(&leaf_id) {
			path.push((leaf_id, edge_index));

			let rollouts: Vec<RolloutResult> =
				(0..self.config.computational_intensity.rollouts_per_node)
					.into_par_iter()
					.map(|_| self.rollout(&child_state))
					.collect();

			self.backpropagate(&path, &child_state, &rollouts);
		} else {
			let node = self.graph.nodes.get(&leaf_id).unwrap();

			if let Some(result) = node.state.result {
				// Node is terminal -> backpropagate immediately
				self.backpropagate(&path, &leaf_id, &[RolloutResult { result, depth: 0 }]);
			} else {
				// Node not terminal but can't (yet) be expanded
				let state = node.state.clone();

				let rollouts: Vec<RolloutResult> =
					(0..self.config.computational_intensity.rollouts_per_node)
						.into_par_iter()
						.map(|_| self.rollout(&state))
						.collect();

				self.backpropagate(&path, &leaf_id, &rollouts);
			}
		}
	}

	/// Selects a leaf node to expand, starting from the given origin node ID.
	fn select(&self, origin: GameState) -> (GameState, Vec<MctsSelection>) {
		let mut path = Vec::<MctsSelection>::new();
		let mut current_id = origin;

		loop {
			let node = self.graph.nodes.get(&current_id).unwrap();

			if node.is_terminal()
				|| node.children.is_empty()
				|| (!node.unexplored_placements.is_empty()
					&& self.config.expansion_predicate.should_expand(node))
			{
				return (current_id, path);
			}

			let (best_edge_to_child, best_child_id) = node
				.children
				.iter()
				.enumerate()
				.map(|(i, edge)| {
					let score = self.config.selection_policy.score(
						node,
						edge,
						self.graph.nodes.get(&edge.child_state).unwrap(),
					);

					(i, edge.child_state, score)
				})
				.max_by(|(_, _, score_a), (_ib, _state_b, score_b)| score_a.total_cmp(score_b))
				.map(|(i, child_state, _)| (i, child_state))
				.unwrap();

			path.push((current_id, best_edge_to_child));
			current_id = best_child_id;
		}
	}

	/// Expands the given node by adding a new, unexplored child node.
	fn expand(&mut self, node_state: &GameState) -> Option<MctsExpansion> {
		let node = self.graph.nodes.get_mut(&node_state).unwrap();

		if node.unexplored_placements.is_empty()
			|| !self.config.expansion_predicate.should_expand(&node)
		{
			return None;
		}

		let placement = self
			.config
			.expansion_policy
			.expand(&mut node.unexplored_placements);

		let mut child_state = node.state;
		child_state.apply_placement(placement);

		node.children.push(Edge::new(placement, child_state));
		let edge_index = node.children.len() - 1;

		let child = self
			.graph
			.nodes
			.entry(child_state)
			.or_insert_with(|| Node::new(child_state));

		child.parents.insert(*node_state);

		Some((edge_index, child_state))
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
		terminal: &GameState,
		rollouts: &[RolloutResult],
	) {
		// Yes, I "back"propagate from the top down instead of bottom up.
		// This way, I don't need to reverse the path order though!

		let num_rollouts = rollouts.len() as u32;

		for &(node_id, edge_index) in path.iter() {
			let node = self.graph.nodes.get_mut(&node_id).unwrap();

			let (node_score, edge_score) =
				rollouts
					.iter()
					.fold((0.0, 0.0), |(node_acc, edge_acc), rollout| {
						(
							node_acc + Self::node_score(&self.config.reward_policy, node, rollout),
							edge_acc + Self::edge_score(&self.config.reward_policy, node, rollout),
						)
					});

			node.visit(num_rollouts, node_score);
			node.children[edge_index].visit(num_rollouts, edge_score);
		}

		let terminal = self.graph.nodes.get_mut(&terminal).unwrap();
		let terminal_score = rollouts
			.par_iter()
			.map(|rollout| Self::node_score(&self.config.reward_policy, &terminal, rollout))
			.sum::<f32>();

		terminal.visit(num_rollouts, terminal_score);
	}

	/// Score from the perspective of the player who moved to this node.
	fn node_score(reward_policy: &RewardPolicy, node: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(&rollout.result, &rollout.depth, &node.state.last_player())
	}

	/// Score from the perspective of the player who made the edge's move.
	fn edge_score(reward_policy: &RewardPolicy, parent: &Node, rollout: &RolloutResult) -> f32 {
		reward_policy.score(&rollout.result, &rollout.depth, &parent.state.next_player())
	}

	/// Reroots the search graph to the node with the given ID,
	/// making it the new root. All nodes that are not reachable
	/// from the new root will be removed.
	pub fn prune(&mut self, new_root: &GameState) {
		self.graph.reroot(new_root);
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

/// Pairing of a [`NodeId`] and the used index in its [`Node::children`].
type MctsSelection = (GameState, EdgeIndex);

/// The result of an expansion, containing
/// the [`EdgeIndex`] of the used edge,
/// the [`NodeId`] of the expanded child,
/// and the expanded node's [`GameState`].
type MctsExpansion = (EdgeIndex, GameState);
