use rand::{Rng, seq::IndexedRandom};

use crate::caminos::{
	board::BitBoard,
	piece::Piece,
	placement::{LEGAL_PLACEMENTS, Placement},
};

/// The two players of a Caminos game.
#[derive(Clone, Copy, Debug)]
pub enum Player {
	A,
	B,
}

/// The result of a Caminos game.
/// A win can be strong (built a bridge) or weak (fewer pieces touching the
/// bottom perimeter; [`BitBoard::BOTTOM_PERIMETER`]).
/// A win for one player automatically results in a loss for the other.
/// The game may also end in a draw.
#[derive(Debug)]
pub enum GameResult {
	/// The player has built a bridge.
	StrongWin(Player),
	/// Neither player built a bridge but the player has fewer pieces touching
	/// the bottom perimeter ([`BitBoard::BOTTOM_PERIMETER`]).
	WeakWin(Player),
	/// Neither player built a bridge and both players have the same amount of
	/// pieces touching the bottom perimeter ([`BitBoard::BOTTOM_PERIMETER`]).
	Draw,
}

/// Game state of a single Caminos player.
#[derive(Clone, Copy)]
pub struct PlayerState {
	/// A bitboard representing the cells occupied by this player's pieces.
	pub occupancy: BitBoard,

	/// The number of this player's pieces that are touching the bottom edge
	/// of the board.
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
	/// Creates a new player state with no occupied cells and the default
	/// number of pieces.
	pub const EMPTY: Self = Self {
		occupancy: BitBoard::EMPTY,
		pieces_touching_bottom_edge: 0,
		l_remaining: 4,
		t_remaining: 4,
		z_remaining: 4,
		o_remaining: 2,
	};

	/// Returns a random piece that this player can still place and decrements
	/// the count of that piece type.
	/// Returns `None` if the player has no pieces left.
	pub fn random_piece(&mut self, rng: &mut impl Rng) -> Option<Piece> {
		let mut pieces = Vec::new();

		if self.l_remaining > 0 {
			pieces.push(Piece::L);
		}

		if self.t_remaining > 0 {
			pieces.push(Piece::T);
		}

		if self.z_remaining > 0 {
			pieces.push(Piece::Z);
		}

		if self.o_remaining > 0 {
			pieces.push(Piece::O);
		}

		let piece = pieces.choose(rng);

		if let Some(piece) = piece {
			match piece {
				Piece::L => self.l_remaining -= 1,
				Piece::T => self.t_remaining -= 1,
				Piece::Z => self.z_remaining -= 1,
				Piece::O => self.o_remaining -= 1,
			}
		}

		piece.copied()
	}

	/// Returns whether the player has any pieces left.
	pub fn has_pieces(&self) -> bool {
		self.l_remaining > 0 || self.t_remaining > 0 || self.z_remaining > 0 || self.o_remaining > 0
	}

	/// Returns a list of the piece types that this player can still place.
	pub fn remaining_piece_types(&self) -> Vec<Piece> {
		let mut pieces = Vec::new();

		if self.l_remaining > 0 {
			pieces.push(Piece::L);
		}

		if self.t_remaining > 0 {
			pieces.push(Piece::T);
		}

		if self.z_remaining > 0 {
			pieces.push(Piece::Z);
		}

		if self.o_remaining > 0 {
			pieces.push(Piece::O);
		}

		pieces
	}
}

/// Game state of a Caminos game.
#[derive(Clone)]
pub struct GameState {
	/// The state of each player in the game.
	pub players: [PlayerState; 2],

	/// The current player.
	pub current_player: Player,

	/// The ordered sequence of placements made in the game.
	pub moves: Vec<Placement>,
}

impl GameState {
	/// Creates a new game state with both players in the initial state and
	/// player 0 starting.
	pub const EMPTY: Self = Self {
		players: [PlayerState::EMPTY, PlayerState::EMPTY],
		current_player: Player::A,
		moves: Vec::new(),
	};

	/// Swaps the current player, changing the turn to the other player.
	pub fn swap_players(&mut self) {
		self.current_player = !self.current_player;
	}

	/// Determines the winner of the game, if any.
	/// Returns [`Some`] if the game state can be determined,
	/// or [`None`] if the game has not yet concluded.
	pub fn determine_winner(&self) -> Option<GameResult> {
		let a = self.players[0];
		let b = self.players[1];

		if a.occupancy.has_bridge(&b.occupancy) {
			return Some(GameResult::StrongWin(Player::A));
		}

		if b.occupancy.has_bridge(&a.occupancy) {
			return Some(GameResult::StrongWin(Player::B));
		}

		let used = a.occupancy | b.occupancy;
		let current = match self.current_player {
			Player::A => a,
			Player::B => b,
		};

		if LEGAL_PLACEMENTS
			.of_many_no_overlap_no_floating(&current.remaining_piece_types(), &used)
			.peekable()
			.peek()
			.is_some()
		{
			// Current player can still place pieces, so the game is not over
			return None;
		}

		if a.pieces_touching_bottom_edge < b.pieces_touching_bottom_edge {
			return Some(GameResult::WeakWin(Player::A));
		}

		if b.pieces_touching_bottom_edge < a.pieces_touching_bottom_edge {
			return Some(GameResult::StrongWin(Player::B));
		}

		Some(GameResult::Draw)
	}
}

// -------------------------------------------------------------------------- //
// UTILITY IMPLS                                                              //
// -------------------------------------------------------------------------- //

impl std::ops::Not for Player {
	type Output = Player;

	fn not(self) -> Self::Output {
		match self {
			Player::A => Player::B,
			Player::B => Player::A,
		}
	}
}

impl Into<usize> for Player {
	fn into(self) -> usize {
		match self {
			Player::A => 0,
			Player::B => 1,
		}
	}
}
