use std::sync::atomic::{AtomicU64, Ordering};

use chacha20::ChaCha8Rng;
use rand::{Rng, RngExt, SeedableRng, rng, seq::IteratorRandom};

use crate::caminos::{
	board::BitBoard,
	piece::Piece,
	placement::Placement,
	state::{GameResult, GameState, Player},
};

/// Defines how to perform a rollout (simulation) from a given game state.
pub struct RolloutPolicy {
	/// The random number generator seed used to select moves during rollout.
	rng_seed: u64,

	/// Atomic counter used to generate unique seeds for each rollout.
	rng_counter: AtomicU64,

	/// Which set of [`RolloutBias`]es to apply during rollouts.
	/// Adding more biases increases the quality of rollouts,
	/// but also increases the computational effort spent on each rollout.
	pub biases: Vec<PlacementBias>,
}

impl RolloutPolicy {
	pub fn rollout(&self, state: &GameState) -> RolloutResult {
		// This is really cool.
		//
		// By using a combination of a fixed seed and an atomic counter to set
		// the stream of the ChaCha8Rng, we can ensure that each rollout
		// produces a different sequence of random numbers, even when
		// parallelized, while keeping reproducibility when using the same
		// initial seed!
		//
		// Taken/inspired by this one:
		// https://github.com/rust-random/rand/blob/31cd3326034525acb06e1616487ef3c41c7acab0/examples/rayon-monte-carlo.rs

		let mut rng = ChaCha8Rng::seed_from_u64(self.rng_seed);
		rng.set_stream(self.rng_counter.fetch_add(1, Ordering::Relaxed));

		let mut simulation = *state;
		let mut depth = 0u8;

		loop {
			if let Some(result) = simulation.result {
				return RolloutResult { result, depth };
			}

			let mut legal_placements = simulation.next_legal_placements().peekable();

			if legal_placements.peek().is_none() {
				// No legal placements, so the game is a draw
				return RolloutResult {
					result: GameResult::Draw,
					depth,
				};
			}

			let random_placement = if self.biases.is_empty() {
				legal_placements.choose(&mut rng).unwrap()
			} else {
				let legal_placements_vec = legal_placements.collect::<Vec<_>>();

				let context = PlacementBiasContext::new(&simulation);
				let mut total = 0.0;
				let weights = legal_placements_vec
					.iter()
					.map(|p| {
						total += self
							.biases
							.iter()
							.fold(1.0, |acc, bias| acc * bias.get_weight(&context, p));
						total
					})
					.collect::<Vec<_>>();

				let threshold: f32 = rng.random_range(0.0..1.0) * total;
				let i = weights.partition_point(|&w| w < threshold);
				legal_placements_vec[i]
			};

			depth += 1;
			simulation.apply_placement(random_placement)
		}
	}

	/// Creates a new [`RolloutPolicy`] with the given RNG seed.
	pub fn seeded(seed: u64, biases: &[PlacementBias]) -> Self {
		Self {
			rng_seed: seed,
			rng_counter: AtomicU64::new(0),
			biases: biases.into(),
		}
	}

	/// Creates a new [`RolloutPolicy`] with a random RNG seed.
	pub fn unseeded(biases: &[PlacementBias]) -> Self {
		Self {
			rng_seed: rng().next_u64(),
			rng_counter: AtomicU64::new(0),
			biases: biases.into(),
		}
	}
}

impl Default for RolloutPolicy {
	fn default() -> Self {
		Self::unseeded(&[])
	}
}

/// Defines the intensity of rollouts to perform during a rollout phase, which
/// can be used to scale the computational effort spent on rollouts compared to
/// tree traversal and selection.
pub struct RolloutIntensity {
	/// The number of rollouts to perform per node during the rollout phase.
	pub rollouts_per_node: u8,

	/// The number of nodes to perform rollouts for during the rollout phase.
	/// This can be used to perform rollouts for multiple nodes in the tree.
	pub nodes_per_rollout: u8,
}

impl Default for RolloutIntensity {
	fn default() -> Self {
		Self {
			rollouts_per_node: 1,
			nodes_per_rollout: 1,
		}
	}
}

/// The simulation details and result of a rollout.
#[derive(Clone, Copy)]
pub struct RolloutResult {
	/// The result of the game after the rollout.
	pub result: GameResult,

	/// The amount of placements made during the rollout, which can be used to
	/// scale the impact of the result. Limited to u8 as Caminos concludes after
	/// a maximum of 28 moves.
	pub depth: u8,
}

/// Precomputed context for placement bias calculations.
pub struct PlacementBiasContext {
	/// The opponent's occupancy bitboard shifted up by one layer.
	opponent_shifted_up: BitBoard,

	/// The opponent's occupancy bitboard shifted in all orthogonal directions,
	/// i.e. north, south, east, west, up, down.
	opponent_shifted_orthogonally: BitBoard,

	/// The opponent's occupancy bitboard shifted in all cardinal directions,
	/// i.e. north, south, east, west, but NOT up or down.
	own_shifted_cardinally: BitBoard,

	/// The opponent's occupancy bitboard shifted in all orthogonal directions,
	/// i.e. north, south, east, west, up, down.
	own_shifted_orthogonally: BitBoard,
}

impl PlacementBiasContext {
	fn new(state: &GameState) -> Self {
		let (own, opponent) = match state.next_player() {
			Player::A => (state.players[0].occupancy, state.players[1].occupancy),
			Player::B => (state.players[1].occupancy, state.players[0].occupancy),
		};

		Self {
			opponent_shifted_up: opponent.shift_up(),
			opponent_shifted_orthogonally: opponent.shift_orthogonally(),
			own_shifted_cardinally: own.shift_cardinally(),
			own_shifted_orthogonally: own.shift_orthogonally(),
		}
	}
}

/// Defines the bias to apply during rollouts, which can be used to guide the
/// rollout policy towards more promising or "human" moves or strategies.
///
/// The base bias is `1`, higher biases increase the chances of a move being
/// selected. The biases are applied multiplicatively, so a bias of `2` means
/// that a move is twice as likely to be selected, while a bias of `0.5` means
/// that a move is half as likely to be selected.
///
/// When multiple biases apply to the same move, their effects are multiplied,
/// so a move that is both tall and touching opponent pieces with biases of `2`
/// and `0.5` respectively would have an overall bias of 1.
#[derive(Clone, Copy)]
pub enum PlacementBias {
	/// Prefer placements spanning two or more layers.$
	/// The taller the placement, the stronger the bias.
	Tall(f32),

	/// Prefer placements lying completely in the bottom layer.
	Flat(f32),

	/// Prefer placements that cover as many opponent cells as possible.
	CoverOpponent(f32),

	/// Prefer placements that cover as many own cells as possible.
	CoverOwn(f32),

	/// Prefer placements that touch as many opponent cells as possible.
	TouchingOpponent(f32),

	/// Prefer placements that touch as many own cells as possible.
	TouchingOwn(f32),

	/// Prefer placements that cover more cells in the north-south direction.
	/// The longer, the stronger the bias.
	NorthSouth(f32),

	/// Prefer placements that cover more cells in the east-west direction.
	/// The longer, the stronger the bias.
	EastWest(f32),

	/// Prefer a specific type of [`Piece`].
	Piece(Piece, f32),
}

impl PlacementBias {
	pub fn get_weight(&self, context: &PlacementBiasContext, placement: &Placement) -> f32 {
		match self {
			PlacementBias::Tall(bias) => {
				if placement.height() > 1 {
					*bias * placement.height() as f32
				} else {
					1.0
				}
			}

			PlacementBias::Flat(bias) => {
				if placement.height() == 1 {
					*bias
				} else {
					1.0
				}
			}

			PlacementBias::CoverOpponent(bias) => {
				let num_covered = (context.opponent_shifted_up & placement.board_mask).count_ones();
				if num_covered > 0 {
					*bias * num_covered as f32
				} else {
					1.0
				}
			}
			PlacementBias::CoverOwn(bias) => {
				let num_covered =
					(context.own_shifted_cardinally & placement.board_mask).count_ones();
				if num_covered > 0 {
					*bias * num_covered as f32
				} else {
					1.0
				}
			}
			PlacementBias::TouchingOpponent(bias) => {
				let num_covered =
					(context.opponent_shifted_orthogonally & placement.board_mask).count_ones();
				if num_covered > 0 {
					*bias * num_covered as f32
				} else {
					1.0
				}
			}
			PlacementBias::TouchingOwn(bias) => {
				let num_covered =
					(context.own_shifted_orthogonally & placement.board_mask).count_ones();
				if num_covered > 0 {
					*bias * num_covered as f32
				} else {
					1.0
				}
			}

			PlacementBias::NorthSouth(bias) => {
				if placement.north_south_extent() > 1 {
					*bias * placement.north_south_extent() as f32
				} else {
					1.0
				}
			}
			PlacementBias::EastWest(bias) => {
				if placement.east_west_extent() > 1 {
					*bias * placement.east_west_extent() as f32
				} else {
					1.0
				}
			}

			PlacementBias::Piece(piece, bias) => {
				if placement.piece == *piece {
					*bias
				} else {
					1.0
				}
			}
		}
	}
}

impl Dimension for &Placement {
	fn height(&self) -> u8 {
		let zs = self.occupied_positions.map(|(_, _, z)| z).into_iter();
		zs.max().unwrap() + 1
	}

	fn north_south_extent(&self) -> u8 {
		let xs = self.occupied_positions.map(|(x, _, _)| x);

		let (min, max) = xs
			.into_iter()
			.fold((u8::MAX, u8::MIN), |(min, max), x| (min.min(x), max.max(x)));

		max - min + 1
	}

	fn east_west_extent(&self) -> u8 {
		let ys = self.occupied_positions.map(|(_, y, _)| y);

		let (min, max) = ys
			.into_iter()
			.fold((u8::MAX, u8::MIN), |(min, max), y| (min.min(y), max.max(y)));

		max - min + 1
	}
}

trait Dimension {
	/// Returns the height of the placement.
	fn height(&self) -> u8;

	/// Returns the north-south extent of the placement,
	/// i.e. the distance it covers in the north-south direction.
	fn north_south_extent(&self) -> u8;

	/// Returns the east-west extent of the placement,
	/// i.e. the distance it covers in the east-west direction.
	fn east_west_extent(&self) -> u8;
}
