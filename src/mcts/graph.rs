use std::hash::{DefaultHasher, Hash, Hasher};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::caminos::{
	placement::{Placement, PlacementRefs},
	state::{GameResult, GameState},
};

/// Identifies a node in the DAG.
/// Calculated as [`GameState::hash`] from [`Node::state`].
pub type NodeId = u64;

/// Identifies the edge from a parent node to a child node.
pub type EdgeIndex = usize;

/// A directed, acyclic graph representing the explored game tree.
pub struct Graph {
	/// Maps node IDs to their corresponding nodes.
	pub nodes: FxHashMap<NodeId, Node>,
}

/// A node in the search graph,
/// representing a game state and its associated metadata.
pub struct Node {
	/// The game state represented by this node.
	pub state: GameState,

	/// The result of the game if this node is terminal,
	/// or `None` if it's not terminal.
	pub result: Option<GameResult>,

	/// The number of times this node has been visited during the search.
	pub visits: u32,

	/// The cumulative score of this node.
	pub score: f32,

	/// The edges to the child nodes.
	pub children: Vec<Edge>,

	/// The IDs of the parent nodes.
	pub parents: FxHashSet<NodeId>,

	/// The placements that have not yet been explored from this node.
	/// Shouldn't overlap with an [`Edge::placement`] from [`Node::children`].
	pub unexplored_placements: PlacementRefs,
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
	pub child_id: NodeId,
}

impl Graph {
	/// Creates a new graph with a single root node
	/// representing the empty game state.
	pub fn new() -> Self {
		let mut nodes = FxHashMap::default();
		nodes.insert(Node::root_id(), Node::new(GameState::EMPTY));

		Graph { nodes }
	}
}

impl Node {
	/// Returns the ID of the root node, which represents the empty game state.
	pub fn root_id() -> NodeId {
		GameState::EMPTY.as_node_id()
	}

	/// Returns the ID of this node, calculated from its game state.
	pub fn id(&self) -> NodeId {
		self.state.as_node_id()
	}

	/// Returns `true` if this node is terminal (i.e. it has a game result).
	pub fn is_terminal(&self) -> bool {
		self.result.is_some()
	}

	/// Updates the visit count and cumulative score of this node
	/// based on the given score.
	pub fn visit(&mut self, score: f32) {
		self.visits += 1;
		self.score += score;
	}

	/// Creates a new node with the given game state and parent IDs.
	pub fn new(state: GameState) -> Self {
		let result = state.determine_winner();

		let unexplored_placements = if result.is_some() {
			Vec::new()
		} else {
			state.legal_placements()
		};

		Self {
			state,
			result,

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
	pub fn visit(&mut self, score: f32) {
		self.visits += 1;
		self.score += score;
	}

	/// Creates a new edge with the given placement and child node ID.
	pub fn new(placement: &'static Placement, child: NodeId) -> Self {
		Self {
			placement,
			visits: 0,
			score: 0.0,
			child_id: child,
		}
	}
}

impl GameState {
	/// Calculates a unique node ID for this game state by hashing its contents.
	pub fn as_node_id(&self) -> NodeId {
		let mut hasher = DefaultHasher::new();
		self.hash(&mut hasher);
		hasher.finish()
	}
}
