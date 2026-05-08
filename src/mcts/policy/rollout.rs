use std::sync::atomic::{AtomicU64, Ordering};

use chacha20::ChaCha8Rng;
use rand::{Rng, SeedableRng, rng, seq::IndexedRandom};

use crate::caminos::state::{GameResult, GameState};

/// A rollout policy that simulates a random playout from the given game state
/// until a terminal state is reached.
pub struct RolloutRandomly {
	/// The random number generator seed used to select moves during rollout.
	rng_seed: u64,

	/// Atomic counter used to generate unique seeds for each rollout.
	rng_counter: AtomicU64,
}

impl RolloutRandomly {
	/// Creates a new `RolloutRandomly` rollout policy with the given
	/// random number generator.
	pub fn seeded(seed: u64) -> Self {
		Self {
			rng_seed: seed,
			rng_counter: AtomicU64::new(0),
		}
	}

	/// Creates a new `RolloutRandomly` rollout policy with a default random
	/// number generator.
	pub fn unseeded() -> Self {
		Self {
			rng_seed: rng().next_u64(),
			rng_counter: AtomicU64::new(0),
		}
	}
}

/// Defines how to perform a rollout (simulation) from a given game state.
pub trait RolloutPolicy {
	/// A function that simulates a playout from the given node and
	/// returns the resulting game outcome.
	fn rollout(&self, state: &GameState) -> RolloutResult;
}

impl Default for Box<dyn RolloutPolicy> {
	fn default() -> Self {
		Box::new(RolloutRandomly::unseeded())
	}
}

/// The simulation details and result of a rollout.
pub struct RolloutResult {
	/// The result of the game after the rollout.
	pub result: GameResult,

	/// The amount of placements made during the rollout, which can be used to
	/// scale the impact of the result. Limited to u8 as Caminos concludes after
	/// a maximum of 28 moves.
	pub depth: u8,
}

impl RolloutPolicy for RolloutRandomly {
	fn rollout(&self, state: &GameState) -> RolloutResult {
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
			if let Some(result) = simulation.determine_winner() {
				return RolloutResult { result, depth };
			}

			match simulation.legal_placements().choose(&mut rng) {
				None => {
					return RolloutResult {
						result: GameResult::Draw,
						depth,
					};
				}

				Some(placement) => {
					depth += 1;
					simulation.apply_placement(placement)
				}
			}
		}
	}
}
