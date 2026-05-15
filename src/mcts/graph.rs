use rustc_hash::{FxHashMap, FxHashSet};

use crate::caminos::{placement::Placement, state::GameState};

/// Identifies the edge from a parent node to a child node.
pub type EdgeIndex = usize;

/// A directed, acyclic graph representing the explored game tree.
pub struct Graph {
	/// Maps node IDs to their corresponding nodes.
	pub nodes: FxHashMap<GameState, Node>,
}

/// A node in the search graph,
/// representing a game state and its associated metadata.
pub struct Node {
	/// The game state represented by this node.
	pub state: GameState,

	/// The number of times this node has been visited during the search.
	pub visits: u32,

	/// The cumulative score of this node.
	pub score: f32,

	/// The edges to the child nodes.
	pub children: Vec<Edge>,

	/// The IDs of the parent nodes.
	pub parents: FxHashSet<GameState>,

	/// The placements that have not yet been explored from this node.
	/// Shouldn't overlap with an [`Edge::placement`] from [`Node::children`].
	pub unexplored_placements: Vec<&'static Placement>,
}

/// An edge in the search graph,
/// representing a move from a parent node to a child node.
pub struct Edge {
	/// The placement that leads from the parent node to the child node.
	pub placement: &'static Placement,

	/// The number of times this edge has been visited during the search.
	pub visits: u32,

	/// The cumulative score of this edge.
	pub score: f32,

	/// The state of the child node that this edge points to.
	pub child_state: GameState,
}

impl Graph {
	/// Creates a new graph with a single root node
	/// representing the empty game state.
	pub fn new() -> Self {
		let mut nodes = FxHashMap::default();
		nodes.insert(GameState::EMPTY, Node::new(GameState::EMPTY));

		Graph { nodes }
	}

	/// Reroots the graph to the node with the given ID, making it the new root.
	/// All nodes that are not reachable from the new root will be removed
	/// from the graph.
	pub fn reroot(&mut self, root: &GameState) {
		let mut visited = FxHashSet::default();
		let mut stack = vec![root.clone()];

		while let Some(state) = stack.pop() {
			if visited.insert(state) {
				if let Some(node) = self.nodes.get(&state) {
					for edge in &node.children {
						stack.push(edge.child_state);
					}
				}
			}
		}

		self.nodes.retain(|node_id, _| visited.contains(node_id));
	}
}

impl Node {
	/// Returns `true` if this node is terminal (i.e. it has a game result).
	pub fn is_terminal(&self) -> bool {
		self.state.result.is_some()
	}

	/// Updates the visit count and cumulative score of this node
	/// based on the given score.
	pub fn visit(&mut self, visits: u32, score: f32) {
		self.visits += visits;
		self.score += score;
	}

	/// Creates a new node with the given game state and parent IDs.
	pub fn new(state: GameState) -> Self {
		let unexplored_placements = if state.result.is_some() {
			Vec::new()
		} else {
			state.next_legal_placements().collect()
		};

		Self {
			state,

			visits: 0,
			score: 0.0,

			children: Vec::new(),
			parents: FxHashSet::default(),

			unexplored_placements,
		}
	}
}

impl Edge {
	/// Updates the visit count and cumulative score of this edge
	/// based on the given score.
	pub fn visit(&mut self, visits: u32, score: f32) {
		self.visits += visits;
		self.score += score;
	}

	/// Creates a new edge with the given placement and child node ID.
	pub fn new(placement: &'static Placement, child_state: GameState) -> Self {
		Self {
			placement,
			visits: 0,
			score: 0.0,
			child_state,
		}
	}
}
