use rustc_hash::{FxHashMap, FxHashSet};

use crate::caminos::{placement::Placement, state::GameState};

/// Identifies a node in the search graph.
pub type NodeIndex = usize;

/// Identifies the edge from a parent node to a child node.
pub type EdgeIndex = usize;

/// A directed, acyclic graph representing the explored game tree.
pub struct Graph {
	/// Collection of all the graph's nodes.
	pub nodes: Vec<Node>,

	/// Maps game states to their corresponding [`NodeIndex`].
	/// Used exclusively for de-duplication on insertion into [`Graph::nodes`].
	pub state_index: FxHashMap<GameState, NodeIndex>,
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
	pub parents: FxHashSet<NodeIndex>,

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

	/// The ID of the child node that this edge points to.
	pub child_index: NodeIndex,
}

impl Graph {
	/// Returns a reference to the node with the given [`NodeIndex`].
	pub fn node(&self, id: NodeIndex) -> &Node {
		&self.nodes[id]
	}

	/// Returns a mutable reference to the node with the given [`NodeIndex`].
	pub fn node_mut(&mut self, id: NodeIndex) -> &mut Node {
		&mut self.nodes[id]
	}

	/// Returns the [`NodeIndex`] of the node for the given state, if it exists.
	pub fn index_opt(&self, state: &GameState) -> Option<NodeIndex> {
		self.state_index.get(state).copied()
	}

	/// Returns the [`NodeIndex`] of the node for the given state,
	/// inserting a new [`Node`] if it doesn't already exist.
	pub fn index(&mut self, state: GameState) -> NodeIndex {
		if let Some(&id) = self.state_index.get(&state) {
			return id;
		}

		let id = self.nodes.len();
		self.nodes.push(Node::new(state));
		self.state_index.insert(state, id);

		id
	}

	/// Creates a new graph with a single root [`Node`]
	/// representing the empty game state.
	pub fn new() -> Self {
		let nodes = vec![Node::new(GameState::EMPTY)];

		let mut state_index = FxHashMap::default();
		state_index.insert(GameState::EMPTY, 0);

		Graph { nodes, state_index }
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
	pub fn new(placement: &'static Placement, child_index: NodeIndex) -> Self {
		Self {
			placement,
			visits: 0,
			score: 0.0,
			child_index,
		}
	}
}
