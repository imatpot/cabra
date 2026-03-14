use rand::Rng;

use crate::caminos::{board::BitBoard, piece::Piece};

/// Game state of a single Caminos player.
pub struct PlayerState {
	/// A bitboard representing the cells occupied by this player's pieces.
	pub occupancy: BitBoard,

	/// The number of this player's pieces that are touching the bottom edge of the board.
	pub pieces_touching_bottom_edge: u8,

	/// The number of L-pieces remaining for this player.
	pub l_remaining: u8,

	/// The number of T-pieces remaining for this player.
	pub t_remaining: u8,

	/// The number of Z-pieces remaining for this player.
	pub z_remaining: u8,

	/// The number of O-pieces remaining for this player.
	pub o_remaining: u8,
}

impl PlayerState {
	/// Creates a new player state with no occupied cells and the default number of pieces.
	pub const EMPTY: Self = Self {
		occupancy: BitBoard::EMPTY,
		pieces_touching_bottom_edge: 0,
		l_remaining: 4,
		t_remaining: 4,
		z_remaining: 4,
		o_remaining: 2,
	};

	/// Returns a random piece that this player can still place.
	/// If the player has no pieces left, returns `None`.
	pub fn random_piece(&self, rng: &mut impl Rng) -> Option<Piece> {
		Piece::random_of(
			rng,
			&[
				if self.l_remaining > 0 {
					Some(Piece::L)
				} else {
					None
				},
				if self.t_remaining > 0 {
					Some(Piece::T)
				} else {
					None
				},
				if self.z_remaining > 0 {
					Some(Piece::Z)
				} else {
					None
				},
				if self.o_remaining > 0 {
					Some(Piece::O)
				} else {
					None
				},
			]
			.into_iter()
			.flatten()
			.collect::<Vec<_>>(),
		)
	}
}

/// Game state of a Caminos game.
pub struct GameState {
	/// The state of each player in the game.
	pub players: [PlayerState; 2],

	/// The index of the current player (0 or 1).
	pub current_player: u8,
}

impl GameState {
	/// Creates a new game state with both players in the initial state and player 0 starting.
	pub const EMPTY: Self = Self {
		players: [PlayerState::EMPTY, PlayerState::EMPTY],
		current_player: 0,
	};

	/// Swaps the current player, changing the turn to the other player.
	pub fn swap_players(&mut self) {
		self.current_player = 1 - self.current_player;
	}
}
