use rand::{Rng, seq::IteratorRandom};

use crate::{
	caminos::{
		board::{BitBoard, Layer},
		piece::Piece,
		placement::{LEGAL_PLACEMENTS, Placement},
	},
	util::ansi,
};

/// The two players of a Caminos game.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
	A,
	B,
}

/// The result of a Caminos game.
/// A win can be strong (built a bridge) or weak (fewer pieces touching the
/// bottom perimeter; [`BitBoard::BOTTOM_PERIMETER`]).
/// A win for one player automatically results in a loss for the other.
/// The game may also end in a draw.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

	/// Returns whether the player has any pieces left.
	pub fn has_pieces(&self) -> bool {
		self.l_remaining > 0 || self.t_remaining > 0 || self.z_remaining > 0 || self.o_remaining > 0
	}

	/// Returns the piece types that this player can still place.
	pub fn remaining_piece_types(&self) -> impl Iterator<Item = &'static Piece> + 'static {
		[
			(self.l_remaining > 0).then_some(&Piece::L),
			(self.t_remaining > 0).then_some(&Piece::T),
			(self.z_remaining > 0).then_some(&Piece::Z),
			(self.o_remaining > 0).then_some(&Piece::O),
		]
		.into_iter()
		.flatten()
	}

	/// Returns a random piece that this player can still place.
	/// Returns `None` if the player has no pieces left.
	pub fn random_piece(&self, rng: &mut impl Rng) -> Option<Piece> {
		self.remaining_piece_types().choose(rng).copied()
	}
}

/// Game state of a Caminos game.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameState {
	/// The state of each player in the game.
	pub players: [PlayerState; 2],

	/// The result of the game, if it has concluded.
	/// This is `None` if the game is ongoing.
	pub result: Option<GameResult>,

	/// The player who is to play the next move.
	next_player: Player,
}

impl GameState {
	/// Creates a new game state with both players in the initial state and
	/// player 0 starting.
	pub const EMPTY: Self = Self {
		players: [PlayerState::EMPTY, PlayerState::EMPTY],
		result: None,
		next_player: Player::A,
	};

	/// Returns the player who is to play the next move.
	pub fn next_player(&self) -> Player {
		self.next_player
	}

	/// Returns the player who played the last move.
	pub fn last_player(&self) -> Player {
		!self.next_player()
	}

	/// Returns all legal placements for the current player.
	/// Returns an empty iterator if the game has concluded.
	pub fn next_legal_placements(&self) -> impl Iterator<Item = &'static Placement> + 'static {
		self.result
			.is_none()
			.then(|| {
				let next_player_state = match self.next_player {
					Player::A => &self.players[0],
					Player::B => &self.players[1],
				};

				LEGAL_PLACEMENTS.of_many_without_overlap_without_floating(
					next_player_state.remaining_piece_types(),
					self.occupancy(),
				)
			})
			.into_iter()
			.flatten()
	}

	/// Applies a placement to the game state.
	/// Assumes that the player has the required piece.
	pub fn apply_placement(&mut self, placement: &Placement) {
		let player = match self.next_player {
			Player::A => &mut self.players[0],
			Player::B => &mut self.players[1],
		};

		player.occupancy = player.occupancy | placement.board_mask;

		match placement.piece {
			Piece::L => player.l_remaining -= 1,
			Piece::T => player.t_remaining -= 1,
			Piece::Z => player.z_remaining -= 1,
			Piece::O => player.o_remaining -= 1,
		}

		if !(placement.board_mask & BitBoard::BOTTOM_PERIMETER).is_empty() {
			player.pieces_touching_bottom_edge += 1;
		}

		self.swap_players();
		self.result = self.determine_winner();
	}

	/// Returns a combined top-view representation of the game state.
	pub fn top_view(&self) -> [Layer; 2] {
		let a = &self.players[0].occupancy;
		let b = &self.players[1].occupancy;

		let a_top = a.layers[2];
		let b_top = b.layers[2];

		let a_mid = a.layers[1] & !b_top;
		let b_mid = b.layers[1] & !a_top;

		let a_bot = a.layers[0] & !b_mid & !b_top;
		let b_bot = b.layers[0] & !a_mid & !a_top;

		[a_bot | a_mid | a_top, b_bot | b_mid | b_top]
	}

	/// Returns a bitboard representing all occupied cells regardless of player.
	pub fn occupancy(&self) -> BitBoard {
		self.players[0].occupancy | self.players[1].occupancy
	}

	/// Determines whether the last move concluded the game.
	/// Returns [`Some`] if the game has concluded,
	/// or [`None`] if the game has not yet concluded.
	fn determine_winner(&self) -> Option<GameResult> {
		let a = &self.players[0];
		let b = &self.players[1];

		if self.next_player == Player::B && a.occupancy.has_bridge(&b.occupancy) {
			return Some(GameResult::StrongWin(Player::A));
		}

		if self.next_player == Player::A && b.occupancy.has_bridge(&a.occupancy) {
			return Some(GameResult::StrongWin(Player::B));
		}

		let next_player_state = match self.next_player {
			Player::A => a,
			Player::B => b,
		};

		if LEGAL_PLACEMENTS
			.of_many_without_overlap_without_floating(
				next_player_state.remaining_piece_types(),
				self.occupancy(),
			)
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
			return Some(GameResult::WeakWin(Player::B));
		}

		Some(GameResult::Draw)
	}

	/// Swaps the current player, changing the turn to the other player.
	fn swap_players(&mut self) {
		self.next_player = !self.next_player;
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

impl std::fmt::Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Player::A => write!(f, "A"),
			Player::B => write!(f, "B"),
		}
	}
}

impl std::fmt::Display for GameState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let mut color = ansi::DIM;

					let char = if self.players[0].occupancy.is_xyz_occupied(x, y, z) {
						color = ansi::MAGENTA;
						'█'
					} else if self.players[1].occupancy.is_xyz_occupied(x, y, z) {
						color = ansi::RESET;
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{}{}{} ", color, char, ansi::RESET)?;
				}

				if z < 2 {
					write!(f, " ")?;
				}
			}

			writeln!(f)?;
		}

		Ok(())
	}
}

impl From<&[&'static Placement]> for GameState {
	fn from(placements: &[&'static Placement]) -> Self {
		let mut state = GameState::EMPTY;

		for placement in placements {
			state.apply_placement(placement);
		}

		state
	}
}
